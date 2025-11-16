use std::collections::HashMap;
use bluer::Error;
use std::{cell::RefCell, rc::Rc, ops::Deref};

use pulseaudio::{mainloop::standard::{IterateResult, Mainloop}, context::Context, proplist::Proplist, def::Retval};
use pulseaudio::context::FlagSet as ContextFlagSet;

pub struct AudioProfiles {
	pub active_profile: String,
	pub profiles: HashMap<String, String>,
}

impl AudioProfiles {
	/// create a new pulse audio connection to a certain device, then returns the current active profile and the other profiles this device supports
	pub fn new(address: String) -> Result<Self, Error> {
		// bla bla make new connection yes very fancy
		let mut proplist = Proplist::new().unwrap();
		proplist.set_str(pulseaudio::proplist::properties::APPLICATION_NAME, "Overskride")
			.unwrap();

		let mainloop = Rc::new(RefCell::new(Mainloop::new()
			.expect("Failed to create mainloop")));

		let context = Rc::new(RefCell::new(Context::new_with_proplist(
			mainloop.borrow().deref(),
			"OverskrideContext",
			&proplist
			).expect("Failed to create new context")));

		context.borrow_mut().connect(None, ContextFlagSet::NOFLAGS, None)
			.expect("Failed to connect context");

		// Wait for context to be ready
		loop {
			match mainloop.borrow_mut().iterate(false) {
				IterateResult::Quit(_) |
				IterateResult::Err(_) => {
					eprintln!("Iterate state was not success, quitting...");
					return Err(Error { kind: bluer::ErrorKind::Failed, message: "iterate state was not a success".to_string() });
				},
				IterateResult::Success(_) => {},
			}
			match context.borrow().get_state() {
				pulseaudio::context::State::Ready => { break; },
				pulseaudio::context::State::Failed |
				pulseaudio::context::State::Terminated => {
					eprintln!("Context state failed/terminated, quitting...");
					return Err(Error { kind: bluer::ErrorKind::Failed, message: "context state failed".to_string() });
				},
				_ => {},
			}
		}

		// pulse audio bluetooth names start with "bluez_card." and instead of : its _ 
		// so like bluez_card.XX_XX_XX_XX_XX_XX
		let card_name = "bluez_card.".to_string() + &address.replace(':', "_");

		let clonable_map = Rc::new(RefCell::new(HashMap::<String, String>::new()));
		let cloned_map = clonable_map.clone();
		let clonable_active_profile = Rc::new(RefCell::new(String::new()));
		let cloned_active_profile = clonable_active_profile.clone();

		let error = Rc::new(RefCell::new(false));
		let error_clone = error.clone();

		// gets the active profile and available profiles of this "card" (it's really a device but wtv)
		let state = context.borrow().introspect().get_card_info_by_name(&card_name, move |card_info_result| {
			match card_info_result {
				pulseaudio::callbacks::ListResult::Item(item) => {	
					let card_profiles = &item.profiles;
					for card_profile in card_profiles {
						// println!("\nprofile: {:?}", card_profile.name);
						// println!("profile description: {:?}", card_profile.description);

						if let Some(profile) = &card_profile.name {
							if let Some(description) = &card_profile.description {
								cloned_map.borrow_mut().insert(profile.to_string(), description.to_string());
							}
						}
					}
					
					*cloned_active_profile.borrow_mut() = if let Some(active) = &item.active_profile {
						if let Some(name) = &active.name {
							name.to_string()
						}
						else {
							"".to_string()
						}
					}
					else {
						"".to_string()
					};

					// println!("\ncard active profile: {:?}", item.active_profile.as_ref().unwrap());
				},
				pulseaudio::callbacks::ListResult::End => {
					println!("device's audio profiles enumerated");
				},
				pulseaudio::callbacks::ListResult::Error => {
					println!("could not get audio profiles for device");
					*error_clone.borrow_mut() = true;
				}
			}
		});

		// process pulse audio requests until the done, if error then return an error
		loop {
			mainloop.borrow_mut().iterate(false);
			std::thread::sleep(std::time::Duration::from_secs(1));
			
			if state.get_state() == pulseaudio::operation::State::Done {
				if *error.borrow() {
					return Err(Error { kind: bluer::ErrorKind::Failed, message: "context state failed".to_string() });
				}
				else {
					break;
				}
			}
		}
		mainloop.borrow_mut().quit(Retval(0));
		context.borrow_mut().disconnect();

		let active = clonable_active_profile.borrow().clone();
		let mut profiles = clonable_map.borrow_mut().clone();

		// remove the "off" profile as that's what the expander switch is for
		profiles.remove("off");

		// return the active profile with the rest of the profiles
		Ok(AudioProfiles { active_profile: active, profiles })
	}
}	

/// sets the profile for a given device
pub fn device_set_profile(address: String, profile: String) {
	// more connection shit 
	let mut proplist = Proplist::new().unwrap();
	proplist.set_str(pulseaudio::proplist::properties::APPLICATION_NAME, "Overskride")
		.unwrap();

	let mainloop = Rc::new(RefCell::new(Mainloop::new()
		.expect("Failed to create mainloop")));

	let context = Rc::new(RefCell::new(Context::new_with_proplist(
		mainloop.borrow().deref(),
		"OverskrideContext",
		&proplist
		).expect("Failed to create new context")));

	context.borrow_mut().connect(None, ContextFlagSet::NOFLAGS, None)
		.expect("Failed to connect context");

	// Wait for context to be ready
	loop {
		match mainloop.borrow_mut().iterate(false) {
			IterateResult::Quit(_) |
			IterateResult::Err(_) => {
				eprintln!("Iterate state was not success, quitting...");
				return;
			},
			IterateResult::Success(_) => {},
		}
		match context.borrow().get_state() {
			pulseaudio::context::State::Ready => { break; },
			pulseaudio::context::State::Failed |
			pulseaudio::context::State::Terminated => {
				eprintln!("Context state failed/terminated, quitting...");
				return;
			},
			_ => {},
		}
	}

	let card_name = "bluez_card.".to_string() + &address.replace(':', "_");

	let clonable_state = Rc::new(RefCell::new(false));
	let clone = clonable_state.clone();
	let done = Rc::new(RefCell::new(false));
	let done_clone = clonable_state.clone();

	println!("{} {}", &card_name, &profile);

	// sets the card profile, then updates the state and the done-ness of this function
	// should move to using the returned "Operation" value instead of weird ass borrows
	context.borrow().introspect().set_card_profile_by_name(&card_name, &profile, Some(Box::new(move |state| {
		*clone.borrow_mut() = state;
		*done_clone.borrow_mut() = true;
	})));

	loop {
		mainloop.borrow_mut().iterate(false);
		std::thread::sleep(std::time::Duration::from_secs(1));

		if *done.borrow() {
			break;
		}		
	}
	mainloop.borrow_mut().quit(Retval(0));
	context.borrow_mut().disconnect();
}
