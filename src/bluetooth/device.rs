use std::collections::HashMap;

use bluer::{AdapterEvent, AdapterProperty, DeviceEvent, DeviceProperty};
use futures::{pin_mut, stream::SelectAll, StreamExt};
use gtk::glib::Sender;
use tokio_util::sync::CancellationToken;

use crate::{message::Message, window::{DEVICES_LUT, CURRENT_ADDRESS, CONFIRMATION_AUTHORIZATION, DISPLAYING_DIALOG}, agent::{self, wait_for_dialog_exit}};

static mut CANCELLATION_TOKEN: Option<CancellationToken> = None;
/// Set the associated with `address` device's state, between connected and not 
/// connected depending on what was already the case.
/// A little funky and needs fixing but works for now.
#[tokio::main]
pub async fn set_device_active(address: bluer::Address, sender: Sender<Message>, adapter_name: String) -> bluer::Result<()> {
	let adapter = bluer::Session::new().await?.adapter(adapter_name.as_str())?;
	let device = adapter.device(address)?;

	sender.send(Message::SwitchActiveSpinner(true)).expect("cannot set spinner to show.");

    let state = device.is_connected().await?;

    if state {
        device.disconnect().await?;
    }
    else if !device.is_paired().await? {
		// let agent = register_agent(&current_session, true, true).await?;
		// println!("agent is: {:?}\n", agent);
		 
   		device.pair().await?;

   		device.connect().await?;
        device.connect().await?;
		// drop(agent);
   	}
   	else {
        device.connect().await?;
   	}

    let updated_state = device.is_connected().await?;

    println!("set state {} for device {}\n", updated_state, device.address());
	sender.send(Message::SwitchActiveSpinner(false)).expect("cannot set spinner to show.");
    sender.send(Message::SwitchSendFileActive(updated_state)).expect("cannot send message");
	// sender.send(Message::SwitchActiveSpinner(false)).expect("cannot set spinner to show.");
    // connected_switch_row.set_active(!connected_switch_row.active());
    
    Ok(())
}

/// Set's the device's blocked state based on what was already the case.
/// Basically stops all connections and requests if the device is blocked.
#[tokio::main]
pub async fn set_device_blocked(address: bluer::Address, sender: Sender<Message>, adapter_name: String)  -> bluer::Result<()> {
    let adapter = bluer::Session::new().await?.adapter(adapter_name.as_str())?;
	let device = adapter.device(address)?;

    let blocked = !device.is_blocked().await?;

    device.set_blocked(blocked).await?;

	sender.send(Message::SwitchBlocked(blocked)).expect("cannot set device blocked.");

    // println!("setting blocked {} for device {}", new_blocked, device.address());
    Ok(())
}

/// Sets the device's trusted state depending on what was already the case.
/// If trusted, connections to the device won't need pin/passkey everytime.
#[tokio::main]
pub async fn set_device_trusted(address: bluer::Address, sender: Sender<Message>, adapter_name: String) -> bluer::Result<()> {
    let adapter = bluer::Session::new().await?.adapter(adapter_name.as_str())?;
    let device = adapter.device(address)?;

    let trusted = !device.is_trusted().await?;

    device.set_trusted(trusted).await?;

    sender.send(Message::SwitchTrusted(trusted)).expect("cannot set device trusted.");
    // println!("setting trusted {} for device {}", new_trusted, device.address());

    Ok(())
}

/// Sets the currently selected device's name, updateing the entry and listboxrow accordingly.
#[tokio::main]
pub async fn set_device_name(address: bluer::Address, name: String, sender: Sender<Message>, adapter_name: String) -> bluer::Result<()> {
	let adapter = bluer::Session::new().await?.adapter(adapter_name.as_str())?;
	let device = adapter.device(address)?;

    device.set_alias(name).await?;
    let current_alias = device.alias().await?;
	
    let mut lut: HashMap<bluer::Address, String>;
    unsafe {
        lut = DEVICES_LUT.clone().unwrap();
        lut.remove(&address);
        lut.insert(address, current_alias.clone());
        DEVICES_LUT = Some(lut);
    }

	sender.send(Message::SwitchName(current_alias, None)).expect("cannot set device name.");

    Ok(())
}

/// Gets the the device associates with `address`, and then retrieves the properties of that device.
#[tokio::main]
pub async fn get_device_properties(address: bluer::Address, sender: Sender<Message>, adapter_name: String) -> bluer::Result<()> {
    let adapter = bluer::Session::new().await?.adapter(adapter_name.as_str())?;
	let device = adapter.device(address)?;

    let is_active = device.is_connected().await?;
    let is_blocked = device.is_blocked().await?;
    let is_trusted = device.is_trusted().await?;
    let alias = device.alias().await?;
    let icon_name = match device.icon().await? {
        Some(icon) => {
            icon
        },
        None => {
            "image-missing-symbolic".to_string()
        },
    };
    
    sender.send(Message::SwitchPage(Some(alias), Some(icon_name))).expect("cannot set device alias and icon in page.");
    sender.send(Message::SwitchActive(is_active)).expect("cannot set device active in page.");
    sender.send(Message::SwitchBlocked(is_blocked)).expect("cannot set device blocked in page.");
    sender.send(Message::SwitchTrusted(is_trusted)).expect("cannot set device trusted in page.");
    sender.send(Message::SwitchSendFileActive(is_active)).expect("cannot send message");
    
    // println!("the devices properties have been gotten with state: {}", is_active);

    Ok(())
}

#[tokio::main]
pub async fn remove_device(address: bluer::Address, sender: Sender<Message>, adapter_name: String) -> bluer::Result<()> {
	let adapter = bluer::Session::new().await?.adapter(adapter_name.as_str())?;
	let device = adapter.device(address)?;

    let title = "Remove Device?".to_string();
    let subtitle = "Are you sure you want to remove <span font_weight='bold' color='#78aeed'>`".to_string() + &device.alias().await? + "`</span>?";
    let confirm = "Remove".to_string();

    unsafe{
        DISPLAYING_DIALOG = true;
    }
    sender.send(Message::RequestYesNo(title, subtitle, confirm, adw::ResponseAppearance::Destructive)).expect("can't send message");

    wait_for_dialog_exit().await;

    let confirmed = unsafe {
        CONFIRMATION_AUTHORIZATION
    };

    if confirmed {
        println!("removing device...");
        let name = device.alias().await?;
        adapter.remove_device(address).await?;
        unsafe {
            let mut devices_lut = DEVICES_LUT.clone().unwrap();
            if devices_lut.contains_key(&address) {
                devices_lut.remove(&address);
                DEVICES_LUT = Some(devices_lut);
            }
        }
        
        sender.send(Message::RemoveDevice(name, address)).expect("can't send message");
        sender.send(Message::UpdateListBoxImage()).expect("can't send message");    
    }

    Ok(())
}

#[tokio::main]
pub async fn stop_searching() { 
    unsafe {
        if let Some(token) = CANCELLATION_TOKEN.clone() {
            token.cancel();
        }
    }
}

#[tokio::main]
pub async fn get_devices_continuous(sender: Sender<Message>, adapter_name: String) -> bluer::Result<()> {
	let session = bluer::Session::new().await?;
	let adapter = &session.adapter(adapter_name.as_str())?;

	let filter = bluer::DiscoveryFilter {
        transport: bluer::DiscoveryTransport::Auto,
        ..Default::default()
    };
    adapter.set_discovery_filter(filter).await?;
	
    let device_events = adapter.discover_devices().await?;
    pin_mut!(device_events);    

    let mut all_change_events = SelectAll::new();

	let session = bluer::Session::new().await?;
	let sender_clone = sender.clone();

	let agent = agent::register_agent(&session, true, false, sender_clone.clone()).await?;	
	println!("registered agent during discovery {:?}", agent);

    let cancellation_token = CancellationToken::new();
    unsafe {
        CANCELLATION_TOKEN = Some(cancellation_token.clone());
    }

    while adapter.is_powered().await? {
        tokio::select! {
            Some(device_event) = device_events.next() => {
                match device_event {
                    AdapterEvent::DeviceAdded(addr) => {
		                if adapter.is_powered().await? {
	                        let supposed_device = adapter.device(addr);
	    
                            let devices_lut = unsafe {
                                DEVICES_LUT.clone().unwrap()
                            };

                            if !devices_lut.contains_key(&addr) {
                                if let Ok(added_device) = supposed_device {
	                                sender.send(Message::AddRow(added_device)).expect("cannot send message {}"); 
	                                sender.send(Message::UpdateListBoxImage()).expect("cannot send message {}"); 
	                                //println!("supposedly sent");
	                                
	                                let device = adapter.device(addr)?;
	                                let change_events = device.events().await?.map(move |evt| (addr, evt));
	                                all_change_events.push(change_events);
                                }
                                else {
                                	println!("device isn't present, something went wrong.");
                                }
                            }
                            else {
                                println!("device already exists, not adding again.");
                            }
		                }
                    }
                    AdapterEvent::DeviceRemoved(addr) => {
   		                if adapter.is_powered().await? {
                            let mut devices_lut = unsafe {
                                DEVICES_LUT.clone().unwrap()
                            };

                            let device_name = if devices_lut.contains_key(&addr) {
                                let lut = devices_lut.get(&addr).unwrap().clone();
                                unsafe {
                                    devices_lut.remove(&addr);
                                    DEVICES_LUT = Some(devices_lut);
                                }

                                lut
                            }
                            else {
                                String::new()
                            };
                            
                            sender_clone.send(Message::RemoveDevice(device_name.clone(), addr)).expect("cannot send message"); 
                            sender_clone.send(Message::UpdateListBoxImage()).expect("cannot send message");
                            println!("Device removed: {:?} {}\n", addr, device_name.clone());    
						}
                    },
                    AdapterEvent::PropertyChanged(AdapterProperty::Powered(powered)) => {
                        std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                        sender_clone.send(Message::SwitchAdapterPowered(powered)).expect("cannot send message {}"); 
                        println!("powered switch to {}", powered);
                    },
                    AdapterEvent::PropertyChanged(AdapterProperty::Discoverable(discoverable)) => {
                        std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                        sender_clone.send(Message::SwitchAdapterDiscoverable(discoverable)).expect("cannot send message {}"); 
                        println!("discoverable switch to {}", discoverable);
                    },
                    AdapterEvent::PropertyChanged(AdapterProperty::Alias(alias)) => {
                    	std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                    	sender_clone.send(Message::SwitchAdapterName(alias.clone(), alias.clone())).expect("cannot send message {}");
                    },
                    event => {
                        println!("unhandled adapter event: {:?}", event);
                    }
                }
            }
            Some((addr, DeviceEvent::PropertyChanged(property))) = all_change_events.next() => {
                match property {
                    DeviceProperty::Connected(connected) => {
                        let current_address = unsafe { 
                        	CURRENT_ADDRESS 
                        };
                       	
                        if addr == current_address {
                            std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                            sender_clone.send(Message::SwitchActive(connected)).expect("cannot send message");
                        }
                    },
                    DeviceProperty::Trusted(trusted) => {
                        let current_address = unsafe {
                        	CURRENT_ADDRESS 
                        };
                        
                        if addr == current_address {
                            std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                            sender_clone.send(Message::SwitchTrusted(trusted)).expect("cannot send message");
                        }
                    },
                    DeviceProperty::Blocked(blocked) => {
                        let current_address = unsafe {
                        	CURRENT_ADDRESS 
                        };
                        
                        if addr == current_address {
                            std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                            sender_clone.send(Message::SwitchBlocked(blocked)).expect("cannot send message");
                        }
                    },
                    DeviceProperty::Alias(name) => {
                        let current_address = unsafe { 
                            CURRENT_ADDRESS
                        };
                        
                        if addr == current_address {
                            std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                            sender_clone.send(Message::SwitchName(name.clone(), None)).expect("cannot send message");
                            sender_clone.send(Message::SwitchPage(Some(name.clone()), None)).expect("cannot send message");
                        }
                        else {
                            let hashmap = unsafe { 
                            	DEVICES_LUT.clone().unwrap() 
                            };
                            
                            let empty = String::new();
                            let old_alias = hashmap.get(&addr).unwrap_or(&empty);

                            sender_clone.send(Message::SwitchName(name.clone(), Some(old_alias.to_string()))).expect("cannot send message");
                        }
                    },
                    DeviceProperty::Icon(icon) => {
                        let current_address = unsafe {
                       		CURRENT_ADDRESS 
                       	};
                       
                       	if addr == current_address {
                            std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                            sender_clone.send(Message::SwitchPage(None, Some(icon))).expect("cannot send message");
                        }
                    },
                    DeviceProperty::Rssi(rssi) => {
                       	let device = unsafe {
                            DEVICES_LUT.clone().unwrap().get(&addr).unwrap_or(&"Unknown Device".to_string()).to_string()
                        };
                        sender_clone.send(Message::SwitchRssi(device, rssi as i32)).expect("cannot send message");
                        sender_clone.send(Message::InvalidateSort()).expect("cannot send message");
                    },
                    event => {
                        println!("unhandeled device event: {:?}", event);
                    },
                }
            }
            _ = cancellation_token.cancelled() => {
                // println!("exited loop from refresh");
                break;
            }
            else => break
        }

        // if cancellation_token.is_cancelled() {
        //     break;
        // }
    }

    println!("exited loop");
    // drop(agent);
    if cancellation_token.is_cancelled() {
        Ok(())
    }
    else {
        Err(bluer::Error { kind: bluer::ErrorKind::Failed, message: "Stopped searching for devices".to_string() })
    }
}
