use dbus::{blocking::{Connection, 
    stdintf::org_freedesktop_dbus::{ObjectManagerInterfacesAdded, PropertiesPropertiesChanged, Properties}},
    Path, arg::{PropMap, RefArg}, MethodErr};

use dbus_crossroads::Crossroads;
use gtk::glib::Sender;
use std::{time::Duration, collections::HashMap, sync::Mutex};
use dbus::channel::MatchingReceiver;

use crate::{message::Message, obex_utils::{ObexAgentManager1, ObexTransfer1}};

const SESSION_INTERFACE: &str = "org.bluez.obex.Session1";
const TRANSFER_INTERFACE: &str = "org.bluez.obex.Transfer1";

static mut SESSION_BUS: Mutex<Option<Connection>> = Mutex::new(None);
static mut CURRENT_SESSION: String = String::new();
static mut CURRENT_TRANSFER: String = String::new();
static mut BREAKING: bool = false;
static mut CURRENT_FILE_SIZE: u64 = 0;
static mut CURRENT_FILE_NAME: String = String::new();
static mut CURRENT_SENDER: Option<Sender<Message>> = None;
pub static mut CANCEL: bool = false;

// fn approx_equal(a: f32, b: f32, decimal_places: u8) -> bool {
//     let factor = 10.0f32.powi(decimal_places as i32);
//     let a = (a * factor).trunc();
//     let b = (b * factor).trunc();
//     a == b
// }

fn handle_properties_updated(interface: String, changed_properties: PropMap) {
    println!("{:?} {:?}", interface.clone(),changed_properties);

    if interface == TRANSFER_INTERFACE {
        let sender = unsafe {
            CURRENT_SENDER.clone().unwrap()
        };
        let status = if let Some(status_holder) = &changed_properties.get_key_value("Status") {
            let dummy_status = status_holder.1.0.as_str().unwrap();
            
            match dummy_status {
                "active" => {
                    sender.send(Message::PopupError("obex-transfer-active".to_string(), adw::ToastPriority::Normal)).expect("cannot send message");
                },
                "complete" => {
                    sender.send(Message::PopupError("obex-transfer-complete".to_string(), adw::ToastPriority::Normal)).expect("cannot send message");
                },
                "error" => {
                    sender.send(Message::PopupError("obex-transfer-error".to_string(), adw::ToastPriority::Normal)).expect("cannot send message");
                },
                message => {
                    sender.send(Message::PopupError(message.to_string(), adw::ToastPriority::Normal)).expect("cannot send message");
                }
            }

            println!("status {:?}", dummy_status);

            dummy_status
        }
        else {
            ""
        }; 

        let value = if let Some(val) = changed_properties.get_key_value("Transferred") {
            let transferred = val.1.0.as_u64();

            let value_mb = match transferred {
                Some(val) => {
                    let mb = val as f32 / 1000000.0;
                    (mb * 100.0).round() / 100.0
                },
                None => {
                    0.0
                }
            };
            println!("transferred: {}", value_mb);
            value_mb
        }
        else {
            0.0
        };
        sender.send(Message::UpdateTransfer(unsafe { CURRENT_FILE_NAME.clone() }, value, status.to_string())).expect("cannot send message");
    }
}

fn handle_interface_added(path: &Path, interfaces: &HashMap<String, PropMap>) {
    for interface in interfaces {
        if interface.0 == SESSION_INTERFACE && path.contains("server") {
            println!("started session: {:?}", interface.0);
            unsafe {
                let session_name = path.clone().to_string();
                let holder = session_name;
                CURRENT_SESSION = holder.clone();
            }
        }
        else if interface.0 == TRANSFER_INTERFACE && path.contains("server") && path.contains("transfer"){
            let conn: &mut Connection;
            unsafe { 
                conn = SESSION_BUS.get_mut().unwrap().as_mut().unwrap();
                CURRENT_TRANSFER = path.clone().to_string();
                println!("path is {}", path);            
            }
            let proxy = conn.with_proxy("org.bluez.obex", path, Duration::from_millis(1000));
            proxy.match_signal(|signal: PropertiesPropertiesChanged, _: &Connection, _: &dbus::Message| {
                handle_properties_updated(signal.interface_name, signal.changed_properties);
                true
            }).expect("can't match signal");
            
            if let Some(session) = &interface.1.get_key_value("Session").unwrap().1.0.as_str() {
                println!("transfer started at {:?}", session);
            }       
        }
    }
}

pub fn register_obex_agent(sender: Sender<Message>) -> Result<(), dbus::Error> {
    let conn: &mut Connection;
    unsafe {
        SESSION_BUS = Mutex::new(Some(Connection::new_session().unwrap()));
        conn = SESSION_BUS.get_mut().unwrap().as_mut().unwrap();
        CURRENT_SENDER = Some(sender.clone());
    }
    
    let proxy = conn.with_proxy("org.bluez.obex", "/", Duration::from_millis(5000));

    proxy.match_signal(|signal: ObjectManagerInterfacesAdded, _: &Connection, _: &dbus::Message| {
        handle_interface_added(&signal.object, &signal.interfaces);
        // println!("caught signal! {:?}", signal);
        true
    }).expect("cannot match signal");

    drop(proxy);
    let proxy2 = conn.with_proxy("org.bluez.obex", "/org/bluez/obex", Duration::from_millis(5000));

    let mut cr = Crossroads::new();

    create_agent(&mut cr, sender.clone());
    proxy2.register_agent(Path::from_slice("/overskride/agent").unwrap()).expect("cant create agent");

    sender.send(Message::StartTransfer("transfer".to_string(), "filename".to_string(), 0.0, 17.23, 18.31)).expect("cannot send message");
    std::thread::sleep(std::time::Duration::from_secs(10));
    sender.send(Message::UpdateTransfer("filename".to_string(), 18.00, "error".to_string())).expect("cannot send message");
    
    serve(conn, cr)?;

    Ok(())
}

fn serve(conn: &mut Connection, mut cr: Crossroads) -> Result<(), dbus::Error> {
    conn.start_receive(dbus::message::MatchRule::new_method_call(), Box::new(move |msg, conn| {
        cr.handle_message(msg, conn).unwrap();
        true
    }));

    // Serve clients forever.
    unsafe {
        while !BREAKING { 
            // println!("serving");
            conn.process(std::time::Duration::from_millis(1000))?;
            if CANCEL {
                let proxy2 = conn.with_proxy("org.bluez.obex", CURRENT_TRANSFER.clone(), Duration::from_millis(5000));
                proxy2.cancel().expect("cannot cancel transfer");

                let sender = CURRENT_SENDER.clone().unwrap();
                std::thread::sleep(std::time::Duration::from_secs(20));
                sender.send(Message::RemoveTransfer(CURRENT_FILE_NAME.clone())).expect("cannot send message");
            }
        }
    }

    Ok(())
}

fn create_agent(cr: &mut Crossroads, sender: Sender<Message>) {
    let agent = cr.register("org.bluez.obex.Agent1", |b| {
        b.method("AuthorizePush", ("transfer",), ("filename",), move |_, _, (transfer,): (Path,)| {
            println!("authorizing...");
            let conn = Connection::new_session().expect("cannot create connection.");
            let props = conn.with_proxy("org.bluez.obex", transfer.clone(), std::time::Duration::from_secs(1)).get_all(TRANSFER_INTERFACE);

            if let Ok(all_props) = props {
                // let filename = "/home/kaii/Downloads/file_test.mp4";
                let filename = all_props.get("Name").expect("cannot get name of file.").0.as_str().unwrap().to_owned();
                let filesize_holder = &*all_props.get("Size").expect("cannot get file size.").0;
                let filesize = filesize_holder.as_u64().unwrap_or(9999);
                
                println!("filename is: {}", filename);
                println!("filesize is: {:?}", filesize);

                unsafe {
                    CURRENT_FILE_NAME = filename.clone();
                    CURRENT_FILE_SIZE = filesize;
                }
                let mb = ((filesize as f32 / 1000000.0) * 100.0).round() / 100.0;

                println!("transfer is: {:?}", transfer);
                sender.send(Message::StartTransfer(transfer.to_string(), filename.clone(), 0.0, 0.0, mb)).expect("cannot send message");

                Ok((filename,))
            }
            else {
                println!("failed to authorize push");
                Err(MethodErr::failed("org.bluez.obex.Error.Rejected"))
            }
        });

        b.method("Cancel", (), (), move |_, _, _: ()| {
            println!("Cancelling...");
            Ok(())
        });

        b.method("Release", (), (), move |_, _, _: ()| {
            println!("Releasing...");
            Ok(())
        });
    });
    println!("created obex agent");

    cr.insert("/overskride/agent", &[agent], ());
}