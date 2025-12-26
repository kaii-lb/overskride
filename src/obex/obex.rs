use dbus::{blocking::{Connection,
    stdintf::org_freedesktop_dbus::{ObjectManagerInterfacesAdded, PropertiesPropertiesChanged, Properties}},
    Path, arg::{PropMap, RefArg, Variant}, MethodErr};

use dbus_crossroads::Crossroads;
use std::{time::Duration, collections::HashMap, sync::Mutex};
use dbus::channel::MatchingReceiver;
use std::str::FromStr;
use adw::glib;
use async_channel::Sender;
use futures::executor::block_on;
use crate::{message::Message, obex_utils::{ObexAgentManager1, ObexTransfer1, ObexClient1, ObexObjectPush1},
            window::OVERSKRIDE_PROPS,
            agent::wait_for_dialog_exit};

const SESSION_INTERFACE: &str = "org.bluez.obex.Session1";
const TRANSFER_INTERFACE: &str = "org.bluez.obex.Transfer1";

static mut SESSION_BUS: Mutex<Option<Connection>> = Mutex::new(None);
static mut CURRENT_SESSION: String = String::new();
static mut CURRENT_TRANSFER: String = String::new();
static mut CURRENT_FILE_SIZE: u64 = 0;
static mut CURRENT_FILE_NAME: String = String::new();
static mut CURRENT_SENDER: Option<Sender<Message>> = None;
static mut OUTBOUND: bool = false;
static mut LAST_BYTES: u64 = 0;
pub static mut BREAKING: bool = false;
pub static mut CANCEL: bool = false;
pub static mut AUTO_ACCEPT_AFTER_FIRST: bool = true;

// fn approx_equal(a: f32, b: f32, decimal_places: u8) -> bool {
//     let factor = 10.0f32.powi(decimal_places as i32);
//     let a = (a * factor).trunc();
//     let b = (b * factor).trunc();
//     a == b
// }

/// Checks if the properties match the [transfer interface](TRANSFER_INTERFACE) and updates various UI elements accordingly
async fn handle_properties_updated(interface: String, changed_properties: PropMap, transfer: String) {
    if interface == TRANSFER_INTERFACE {
        let sender = unsafe {
            CURRENT_SENDER.clone().unwrap()
        };
        let status = if let Some(status_holder) = &changed_properties.get_key_value("Status") {
            let dummy_status = status_holder.1.0.as_str().unwrap();

            // self-explanatory, but it tells the user about what's happening with the transfer
            match dummy_status {
                "active" => {
                	if unsafe { OUTBOUND } {
                    	sender.send(Message::PopupError("obex-transfer-active-outbound".to_string(), adw::ToastPriority::Normal)).await.expect("cannot send message");
                    	unsafe { BREAKING = false; }
                	}
                	else {
                    	sender.send(Message::PopupError("obex-transfer-active-inbound".to_string(), adw::ToastPriority::Normal)).await.expect("cannot send message");
                	}
                },
                "complete" => {
                	if unsafe { OUTBOUND } {
                    	sender.send(Message::PopupError("obex-transfer-complete-outbound".to_string(), adw::ToastPriority::Normal)).await.expect("cannot send message");
                    	unsafe { BREAKING = true; }
                	}
                	else {
                    	sender.send(Message::PopupError("obex-transfer-complete-inbound".to_string(), adw::ToastPriority::Normal)).await.expect("cannot send message");
                        move_to_store_folder(&sender).await;
                	}
                },
                "error" => {
                	if unsafe { OUTBOUND } {
                    	sender.send(Message::PopupError("obex-transfer-error-outbound".to_string(), adw::ToastPriority::Normal)).await.expect("cannot send message");
                    	unsafe { BREAKING = true; }
                	}
                	else {
                    	sender.send(Message::PopupError("obex-transfer-error-inbound".to_string(), adw::ToastPriority::Normal)).await.expect("cannot send message");
                	}
                },
                message => {
                    sender.send(Message::PopupError(message.to_string(), adw::ToastPriority::Normal)).await.expect("cannot send message");
                  	unsafe { BREAKING = false; }
                }
            }

            // println!("status: {:?}", dummy_status);

            dummy_status
        }
        else {
            ""
        };

        // convert from bytes to megabytes
        let (mb, kb) = if let Some(val) = changed_properties.get_key_value("Transferred") {
            let transferred = val.1.0.as_u64();

            // calculate the transfer speed by subtracting the current amount from the last
            let value_kb = unsafe {
                // println!("transferred: {}, last: {}", transferred.unwrap(), LAST_BYTES);
                (transferred.unwrap_or(0) / 1000).saturating_sub(LAST_BYTES / 1000)
            };

            let value_mb = match transferred {
                Some(val) => {
                    ((val as f32 / 1000000.0) * 1000.0).round() / 1000.0
                },
                None => {
                    0.0
                }
            };
            // println!("transferred: {}", value_mb);

            unsafe {
                LAST_BYTES = transferred.unwrap_or(0);
            }

            (value_mb, value_kb)
        }
        else {
            (0.0, 0)
        };
        // updates the transfer with the specified values
        sender.send(Message::UpdateTransfer(transfer, unsafe { CURRENT_FILE_NAME.clone() }, mb, kb, status.to_string())).await.expect("cannot send message");
    }
}

/// Is run when a new interface gets added, i.e. a new connection to dbus on the specified path or a new session
fn handle_interface_added(path: &Path, interfaces: &HashMap<String, PropMap>) {
    for interface in interfaces {

        // if interface is a session interface then set those variables accordingly
        if interface.0 == SESSION_INTERFACE && path.contains("server") {
            println!("started session: {:?}", interface.0);
            unsafe {
                let session_name = path.clone().to_string();
                let holder = session_name;
                CURRENT_SESSION = holder.clone();
            }
        }
        // if the interface is a transfer then handle the properties updated signal
        else if interface.0 == TRANSFER_INTERFACE && path.contains("server") && path.contains("transfer") {
            let conn: &mut Connection;
            unsafe {
                conn = SESSION_BUS.get_mut().unwrap().as_mut().unwrap();
				if CURRENT_TRANSFER != path.to_string()	{
                    OVERSKRIDE_PROPS.lock().unwrap().confirm_authorization = false
				}

                CURRENT_TRANSFER = path.to_string();
                println!("path is {}", path);
            }
            let proxy = conn.with_proxy("org.bluez.obex", path, Duration::from_millis(1000));
            proxy.match_signal(|signal: PropertiesPropertiesChanged, _: &Connection, message: &dbus::Message| {
                let transfer = if let Some(path) = message.path() {
                    path.to_string()
                }
                else {
                    "".to_string()
                };

                glib::spawn_future_local(async {
                    handle_properties_updated(signal.interface_name, signal.changed_properties, transfer).await;
                });
                true
            }).expect("can't match signal");

            if let Some(session) = &interface.1.get_key_value("Session").unwrap().1.0.as_str() {
                println!("transfer started at {:?}", session);
            }
        }
    }
}

/// Register a new obex agent to dbus, allowing files to be received
pub fn register_obex_agent(sender: Sender<Message>) -> Result<(), dbus::Error> {
    let conn: &mut Connection;
    unsafe {
        SESSION_BUS = Mutex::new(Some(Connection::new_session()?));
        conn = SESSION_BUS.get_mut().unwrap().as_mut().unwrap();
        CURRENT_SENDER = Some(sender.clone());
    }

    let proxy = conn.with_proxy("org.bluez.obex", "/", Duration::from_millis(5000));

    // matches the signal of a new object getting added to the dbus interface (ie an agent)
    proxy.match_signal(|signal: ObjectManagerInterfacesAdded, _: &Connection, _: &dbus::Message| {
        handle_interface_added(&signal.object, &signal.interfaces);
        // println!("caught signal! {:?}", signal);
        true
    }).expect("cannot match signal");

    drop(proxy);
    let proxy2 = conn.with_proxy("org.bluez.obex", "/org/bluez/obex", Duration::from_millis(5000));

    let mut cr = Crossroads::new();

    create_agent(&mut cr, sender.clone());
    proxy2.register_agent(Path::from_slice("/overskride/agent").unwrap()).expect("can't create agent");

    block_on(serve(conn, Some(cr))).expect("Cannot serve obex");

    Ok(())
}

/// Infinitely processes dbus requests until canceled
async fn serve(conn: &mut Connection, cr: Option<Crossroads>) -> Result<(), dbus::Error> {
	if let Some(mut crossroads) = cr {
	    conn.start_receive(dbus::message::MatchRule::new_method_call(), Box::new(move |msg, conn| {
	        crossroads.handle_message(msg, conn).unwrap();
	        true
	    }));
	}

    // Serve clients forever.
    unsafe {
        BREAKING = false;
        CANCEL = false;

        while !CANCEL && !BREAKING {
            // println!("serving");
            conn.process(Duration::from_millis(1000))?;
        }

        let sender = CURRENT_SENDER.clone().unwrap();

        let proxy2 = conn.with_proxy("org.bluez.obex", CURRENT_TRANSFER.clone(), Duration::from_millis(5000));

        if CANCEL {
            if let Err(err) = proxy2.cancel() {
                println!("error while canceling transfer {:?}", err.message());
            }
            println!("canceled");
            CANCEL = false;
        }

        BREAKING = false;

        // update transfer UI with the filename and transferred amount
        let filename = proxy2.name().unwrap_or("Unknown File".to_string());
        let transferred = (proxy2.transferred().unwrap_or(9999) as f32 / 1000000.0).round() / 100.0;
        sender.send(Message::UpdateTransfer(proxy2.path.to_string(), filename.clone(), transferred, 0, "error".to_string())).await.expect("cannot send message");

        // remove the transfer from the list after 1 minute
        std::thread::spawn(async move || {
            tokio::time::sleep(Duration::from_secs(60)).await;
            sender.send(Message::RemoveTransfer(CURRENT_TRANSFER.clone(), filename)).await.expect("cannot send message");
        });
    }
    Ok(())
}

/// This functions describes the methods an agent has, creates an object of that agent, and inserts it into a crossroads instance
fn create_agent(cr: &mut Crossroads, sender: Sender<Message>) {
    let agent = cr.register("org.bluez.obex.Agent1", |b| {
        b.method("AuthorizePush", ("transfer",), ("filename",), move |_, _, (transfer,): (Path,)| {
            println!("authorizing...");
            let conn = Connection::new_session().expect("cannot create connection.");
            let props = conn.with_proxy("org.bluez.obex", transfer.clone(), Duration::from_secs(5)).get_all(TRANSFER_INTERFACE);

            if let Ok(all_props) = props {
                // lots of fuckery but its self-explanatory
                let filename = all_props.get("Name").expect("cannot get name of file.").0.as_str().unwrap().to_owned();
                let filesize_holder = &*all_props.get("Size").expect("cannot get file size.").0;
                let filesize = filesize_holder.as_u64().unwrap_or(9999);
				let session = all_props.get("Session").expect("cannot get session for receive").0.as_str().unwrap_or("");

                // println!("all props is: {:?}", all_props);

                unsafe {
                    CURRENT_FILE_NAME = filename.clone();
                    CURRENT_FILE_SIZE = filesize;
               		OUTBOUND = false;
                }
                let mb = ((filesize as f32 / 1000000.0) * 100.0).round() / 100.0; // to megabytes

				// get the target device, if it doesn't exist, panic ensues
				let sender_props = conn.with_proxy("org.bluez.obex", session, Duration::from_secs(5)).get_all(SESSION_INTERFACE).unwrap();
				let device = sender_props.get("Destination").expect("cannot get sender device").0.as_str().unwrap_or("00:00:00:00:00:00");

				let (device_name, device_trusted) = if let Ok(props) = get_device_props(device) {
					props
				}
				else {
					return Err(MethodErr::from(("org.bluez.obex.Error.Canceled", "Request Canceled")));
				};

				// if user sets auto accept from trusted, immediately accept the transfer without confirmation
				if OVERSKRIDE_PROPS.lock().unwrap().auto_accept_from_trusted && device_trusted {
                    println!("transfer is: {:?}", transfer);
                    block_on(sender.send(Message::StartTransfer(transfer.to_string(), filename.clone(), 0.0, 0.0, mb, false))).expect("cannot send message");

					return Ok((filename,));
				}

				// if the ~/.cache directory doesn't exist, return as we have nowhere to store the file
				if !glib::user_cache_dir().exists() {
                    block_on(sender.clone().send(Message::PopupError("file-storage-cache-invalid".to_string(), adw::ToastPriority::High))).expect("cannot send message");

  					return Err(MethodErr::from(("org.bluez.obex.Error.Canceled", "Request Canceled")));
                }

                // spawn a dialog returning the accepted bool, no accepted => reject transfer
                if OVERSKRIDE_PROPS.lock().unwrap().confirm_authorization || spawn_dialog(filename.clone(), &sender, device_name) {
                    println!("transfer is: {:?}", transfer);
                    block_on(sender.send(Message::StartTransfer(transfer.to_string(), filename.clone(), 0.0, 0.0, mb, false))).expect("cannot send message");

                    unsafe {
                    	if !AUTO_ACCEPT_AFTER_FIRST {
                            OVERSKRIDE_PROPS.lock().unwrap().confirm_authorization = false;
                    	}
                    }

                    Ok((filename,))
                }
                else {
                    println!("rejected push");
                    let error = MethodErr::from(("org.bluez.obex.Error.Rejected", "Not Authorized"));
                    OVERSKRIDE_PROPS.lock().unwrap().confirm_authorization = false;
                    Err(error)
                }
            }
            else {
                OVERSKRIDE_PROPS.lock().unwrap().confirm_authorization = false;
                println!("failed to authorize push");
                Err(MethodErr::from(("org.bluez.obex.Error.Canceled", "Request Canceled")))
            }
        });

        // these are never called, not sure why they exist
        b.method("Cancel", (), (), move |_, _, _: ()| {
            println!("Cancelling...");
            Ok(())
        });

        b.method("Release", (), (), move |_, _, _: ()| {
            println!("Releasing...");
            Ok(())
        });
    });
    cr.insert("/overskride/agent", &[agent], ());

    println!("created obex agent");
}

/// Spawns a new dialog asking the user to allow or reject a file transfer from a device
#[tokio::main]
async fn spawn_dialog(filename: String, sender: &Sender<Message>, device_name: String) -> bool {
    println!("file receive request incoming!");

    let title = "File Transfer Incoming".to_string();
    let subtitle = "Accept <span font_weight='bold' color='#78aeed'>".to_string() + &filename + "</span> from <span font_weight='bold'>" + &device_name + "?</span>";
    let confirm = "Accept".to_string();
    let response_type = adw::ResponseAppearance::Suggested;

    OVERSKRIDE_PROPS.lock().unwrap().displaying_dialog = true;
    sender.send(Message::RequestYesNo(title, subtitle, confirm, response_type)).await.expect("cannot send message");

    wait_for_dialog_exit().await;

    tokio::time::sleep(Duration::from_millis(500)).await;
    OVERSKRIDE_PROPS.lock().unwrap().confirm_authorization
}

/// Wrapper function handling the adapter and target device, looping over all the files needing to be sent and sending them on by one
#[tokio::main]
pub async fn start_send_file(destination: bluer::Address, source: bluer::Address, sender: Sender<Message>) {
    OVERSKRIDE_PROPS.lock().unwrap().displaying_dialog = true;
    sender.send(Message::GetFile(gtk::FileChooserAction::Open)).await.expect("cannot send message");

    wait_for_dialog_exit().await;

    let file_paths = OVERSKRIDE_PROPS.lock().unwrap().send_files_path.clone();

    // if calling on an empty transfer, get out
    if file_paths.is_empty() {
        return;
    }

    let conn = Connection::new_session().expect("cannot create send connection");
    let proxy = conn.with_proxy("org.bluez.obex", "/org/bluez/obex", Duration::from_secs(5));

    // describes the properties of the transfer, like the origin and target devices
    let mut hashmap = PropMap::new();

    hashmap.insert("Target".to_string(),Variant(Box::new("OPP".to_string())));

    hashmap.insert("Source".to_string(), Variant(Box::new(source.to_string())));

    let sesh = proxy.create_session(&destination.to_string(), hashmap);

    let send_session = if sesh.is_ok()  {
    	sesh.unwrap()
    }
    else {
        println!("Could not establish connection to send file {:?}", sesh.err().unwrap());
        sender.send(Message::PopupError("obex-transfer-connection-error".to_string(), adw::ToastPriority::Normal)).await.expect("cannot send message");
    	return;
    };
    println!("send session is: {:?}, with filepaths {:?}", send_session, file_paths);

    // for every file, try to send it to the target device
    for file in file_paths {
        println!("file to be sent is {}", file);
        send_file(file.clone(), &send_session, sender.clone()).await;
    }
    println!("done sending files");
}

/// Sends a specified file from the file path to a target device, updating the UI in the process
async fn send_file(source_file: String, session_path: &Path<'_>, sender: Sender<Message>) {
    let conn = Connection::new_session().expect("cannot create send session");
    let proxy = conn.with_proxy("org.bluez.obex", session_path, Duration::from_secs(1));

    // return the path and properties
    let output = proxy.send_file(source_file.as_str()).unwrap();

    println!("send transfer path is: {:?}", output.0.clone());
    println!("send properties are: {:?}\n", output.1);

	// create a new proxy to the transfer path for easier processing of properties
	let transfer_proxy = conn.with_proxy("org.bluez.obex", output.0.clone(), Duration::from_secs(5));

	unsafe {
		CURRENT_TRANSFER = output.0.clone().to_string();
		OUTBOUND = true;
	}

    // changes filesize from bytes(?) to megabytes, then starts a transfer with the filename and size
    // let mb = ((transfer_proxy.size().unwrap_or(9999) as f32 / 1000000.0) * 100.0).round() / 100.0;
    let mb = ((transfer_proxy.size().unwrap_or(9999) as f32 / 1000000.0) * 1000.0).round() / 1000.0;
    sender.send(Message::StartTransfer(output.0.clone().to_string(), transfer_proxy.name().unwrap_or("Unknown File".to_string()), 0.0, 0.0, mb, true)).await.expect("cannot send message");

	transfer_proxy.match_signal(move |signal: PropertiesPropertiesChanged, _: &Connection, _: &dbus::Message| {
        block_on(handle_properties_updated(signal.interface_name, signal.changed_properties, output.0.to_string()));
	    true
    }).expect("can't match signal");

    unsafe {
        BREAKING = false;
        CANCEL = false;

        while !CANCEL && !BREAKING {
            // process dbus requests to that path
            conn.process(Duration::from_millis(1000)).expect("cannot process request");
        }

        let sender = CURRENT_SENDER.clone().unwrap();

        // stop sending this file
        if CANCEL {
            if let Err(err) = transfer_proxy.cancel() {
                // sender.send(Message::PopupError("obex-transfer-cancel-not-authorized".to_string(), adw::ToastPriority::Normal)).await.expect("cannot send message");
                println!("error while canceling transfer {:?}", err.message());
            }
        	let transferred = (transfer_proxy.transferred().unwrap_or(9999) as f32 / 1000000.0).round() / 100.0;
            sender.send(Message::UpdateTransfer(CURRENT_TRANSFER.clone(), CURRENT_FILE_NAME.clone(), transferred, 0, "error".to_string())).await.expect("cannot send message");
            drop(transfer_proxy);
            drop(proxy);
            CANCEL = false;
        }

        BREAKING = false;

        // remove the transfer from the list after 1 minute
        std::thread::spawn(async move || {
            tokio::time::sleep(Duration::from_secs(60)).await;
            sender.send(Message::RemoveTransfer(CURRENT_TRANSFER.clone(), CURRENT_FILE_NAME.clone())).await.expect("cannot send message");
        });
    }
}

/// Moves a received file to where the user needs it to be
/// needed because returning a file path in the agent's "AuthorizePush" method won't work because bluetooth :D
pub async fn move_to_store_folder(sender: &Sender<Message>) {
	if unsafe { OUTBOUND } {
		return;
	}

    let filename = unsafe {
        CURRENT_FILE_NAME.clone()
    };
    let store_folder = OVERSKRIDE_PROPS.lock().unwrap().store_folder.clone();
    // default path of stored file by obexd
    let filepath = if let Some(cache_dir) = glib::user_cache_dir().to_str() {
        cache_dir.to_string() + "/obexd/" + &filename
    }
    else {
    	sender.send(Message::PopupError("obex-transfer-cant-move".to_string(), adw::ToastPriority::High)).await.expect("cannot send message");
        println!("unable to save file to store folder, it should still remain in ~/.cache/obexd");
        return;
    };

    let new_filepath = store_folder + &filename;

    // move file to location and handle error
    match std::fs::rename(filepath, new_filepath) {
        Ok(()) => {
            println!("file moved to directory");
        },
        Err(err) => {
          	sender.send(Message::PopupError("obex-transfer-cant-move".to_string(), adw::ToastPriority::High)).await.expect("cannot send message");
            println!("file was not moved due to {:?}", err);
        },
    }
}

/// Gets the name and trusted value of a specified device
#[tokio::main]
async fn get_device_props(address_slice: &str) -> bluer::Result<(String, bool)> {
	let adapter = bluer::Session::new().await?.adapter(
        &OVERSKRIDE_PROPS.lock().unwrap().current_adapter
    )?;
	let address = bluer::Address::from_str(address_slice).unwrap_or(bluer::Address::any());
	let device = adapter.device(address)?;

	let trusted = device.is_trusted().await?;
	let name = device.alias().await?;

	Ok((name, trusted))
}
