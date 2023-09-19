/* window.rs
*
 * Copyright 2023 kaii
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */
use adw::subclass::prelude::*;
use adw::prelude::*;
use gtk::gio::Settings;
use gtk::glib::Sender;
use gtk::{gio, glib};
use gtk;

use bluer::{AdapterEvent, AdapterProperty, DeviceEvent, DeviceProperty};
use futures::{pin_mut, stream::SelectAll, StreamExt};
use std::cell::OnceCell;
use std::collections::HashMap;

static mut CURRENT_ADDRESS: bluer::Address = bluer::Address::any();
static mut CURRENT_INDEX: i32 = 0;
static mut CURRENT_SENDER: Option<Sender<Message>> = None;
static mut CURRENT_ADAPTER: String = String::new();
static mut ORIGINAL_ADAPTER: String = String::new();
static mut DEVICES_LUT: Option<HashMap<bluer::Address, String>> = None;
static mut ADAPTERS_LUT: Option<HashMap<String, String>> = None;
static mut RSSI_LUT: Option<HashMap<String, i32>> = None;
static mut CAN_CONTINUE_LOOP: bool = true;
static mut IS_CURRENTLY_LOOPING: bool = false;

enum Message {
    #[allow(dead_code)]
    SwitchTrusted(bool),
    SwitchBlocked(bool),
    SwitchActive(bool),
    SwitchName(String),
    SwitchPage(Option<String>, Option<String>),
    RemoveDevice(String),
    AddRow(bluer::Device),
    SwitchAdapterPowered(bool),
    SwitchAdapterTimeout(u32),
    SwitchAdapterDiscoverable(bool),
    SwitchAdapterName(String, String),
    PopulateAdapterExpander(HashMap<String, String>),
    SetRefreshSensitive(bool),
} 

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/kaii/Overskride/gtk/window.ui")]
    pub struct OverskrideWindow {
        #[template_child]
        pub main_listbox: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub refresh_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub connected_switch_row: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub device_name_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub trusted_row: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub blocked_row: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub remove_device_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub bluetooth_settings_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub device_icon: TemplateChild<gtk::Image>,
        #[template_child]
        pub device_title: TemplateChild<gtk::Label>,
        #[template_child]
        pub main_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub secondary_listbox: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub powered_switch_row: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub discoverable_switch_row: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub adapter_name_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub timeout_time_adjustment: TemplateChild<gtk::Adjustment>,   
        #[template_child]
        pub timeout_row: TemplateChild<adw::SpinRow>, 
        #[template_child]
        pub default_controller_expander: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub split_view: TemplateChild<adw::OverlaySplitView>,
        #[template_child]
        pub show_sidebar_button: TemplateChild<gtk::ToggleButton>,

        pub settings: OnceCell<Settings>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for OverskrideWindow {
        const NAME: &'static str = "OverskrideWindow";
        type Type = super::OverskrideWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            /*klass.install_action("win.refresh_devices", None, move |win, _, _| {
                
            });*/
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for OverskrideWindow {
        fn constructed(&self) {
            self.parent_constructed();
            
            let obj = self.obj();
            obj.setup_settings();
            obj.load_window_size();
        }
    }
    impl WidgetImpl for OverskrideWindow {}
    impl WindowImpl for OverskrideWindow {
        fn close_request(&self) -> glib::Propagation {
            self.obj().save_window_size().expect("cannot save window size");

            glib::Propagation::Proceed
        }
    }
    impl ApplicationWindowImpl for OverskrideWindow {}
    impl AdwApplicationWindowImpl for OverskrideWindow {}
}

glib::wrapper! {
    pub struct OverskrideWindow(ObjectSubclass<imp::OverskrideWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,        
        @implements gio::ActionGroup, gio::ActionMap;
}

impl OverskrideWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        //glib::Object::builder()
        //    .property("application", application)
        //  .build();

        let win: OverskrideWindow = glib::Object::builder().property("application", application).build();

        win.setup().expect("cannot setup window");

        win
    }

    fn setup_settings(&self) {
        let settings = Settings::new("com.kaii.Overskride");
        self.imp().settings.set(settings).expect("settings not setup");
    }

    /// Sets up the application. Basically it binds actions to stuff and updates what needs to be updated.
    #[tokio::main]
    async fn setup(&self) -> bluer::Result<()> {
        let (sender, receiver) = glib::MainContext::channel::<Message>(glib::Priority::default());
        let settings = self.imp().settings.get().unwrap();

        unsafe { 
            CURRENT_SENDER = Some(sender.clone());
            DEVICES_LUT = Some(HashMap::new());
            RSSI_LUT = Some(HashMap::new());
            let name = settings.string("current-adapter-name").to_string();
            if name == "" {
                CURRENT_ADAPTER = bluer::Session::new().await?.default_adapter().await?.name().to_string();
                ORIGINAL_ADAPTER = CURRENT_ADAPTER.clone().to_string();
                settings.set_string("current-adapter-name", CURRENT_ADAPTER.as_str()).expect("cannot set default adapter at start");
                settings.set_string("original-adapter-name", CURRENT_ADAPTER.as_str()).expect("cannot set original adapter at start");
            }
            else {
                CURRENT_ADAPTER = name.clone();
            }
            let mut lut = HashMap::new();
            
            std::process::Command::new("bluetoothctl").arg("select").arg(name.clone().to_string());
            let old_output = String::from_utf8(std::process::Command::new("bluetoothctl").arg("show").output().expect("cant do so").stdout).expect("nah");
            let old_name = old_output.lines().nth(2).unwrap().replace("\tAlias: ", "");
            let alias = &old_name[0..old_name.find("AdvertisementMonitor").unwrap_or(old_name.len())];
            println!("startup alias is: {}", alias.clone());

            lut.insert(alias.to_string(), CURRENT_ADAPTER.to_string());
            ADAPTERS_LUT = Some(lut);
        }

        let self_clone = self.clone();
        receiver.attach(None, move |msg| {
            let clone = self_clone.clone();
            match msg {
                Message::SwitchTrusted(trusted) => {
                    let trusted_row = clone.imp().trusted_row.get();
                    trusted_row.set_active(trusted);
                },
                Message::SwitchBlocked(blocked) => {
                    let blocked_row = clone.imp().blocked_row.get();
                    blocked_row.set_active(blocked);
                },
                Message::SwitchActive(active) => {
                    let connected_switch_row = clone.imp().connected_switch_row.get();
                    connected_switch_row.set_active(active);
                },
                Message::SwitchName(alias) => {
                    let list_box = clone.imp().main_listbox.get();
                    let index: i32;
                    unsafe { index = CURRENT_INDEX }
                    let row = list_box.row_at_index(index);
                    
                    if row.is_some() {
                        let action_row = row.unwrap().downcast::<adw::ActionRow>().unwrap();
                        action_row.set_title(alias.as_str());
                    }
                },
                Message::AddRow(device) => {
                    let row = add_child_row(device);
                    if row.is_ok() {
                        let main_listbox = clone.imp().main_listbox.get();
                        main_listbox.append(&row.unwrap());
                        main_listbox.invalidate_sort();
                    }
                },
                Message::RemoveDevice(name) => {
                    let listbox = clone.clone().imp().main_listbox.get();
                    let mut index = 0;
                    while listbox.clone().row_at_index(index) != None {
                        // println!("{}", index);
                        let action_row = listbox.clone().row_at_index(index).unwrap().downcast::<adw::ActionRow>().expect("cannot downcast to action row.");
                        // println!("{:?}", action_row.clone());
                        if action_row.clone().title().to_string() == name {
                            listbox.clone().remove(&action_row);
                        }
                        index += 1;
                    }

                    let bluetooth_settings_row = clone.clone().imp().bluetooth_settings_row.get();
                    bluetooth_settings_row.emit_activate();
                }
                Message::SwitchPage(alias, icon_name) => {
                    let entry_row = clone.imp().device_name_entry.get();
                    let device_title = clone.imp().device_title.get();
                    let device_icon = clone.imp().device_icon.get();
                    
                    match alias {
                        Some(name) => {
                            entry_row.set_text(name.as_str());
                            device_title.set_text(name.as_str());
                        }
                        None => (),
                    }

                    match icon_name {
                        Some(icon) => {
                            let final_icon_name = icon.clone() + "-symbolic";

                            device_icon.set_icon_name(Some(final_icon_name.as_str()));
                            println!("icon name is: {}", icon);
                        }
                        None => (),
                    }
                    
                    let secondary_listbox = clone.imp().secondary_listbox.get();
                    secondary_listbox.unselect_all();
                    
                    let main_stack = clone.imp().main_stack.get();
                    let pages = main_stack.pages();
                    pages.select_item(0, true);

                    let split_view = clone.imp().split_view.get();
                    if split_view.is_collapsed() {
                        split_view.set_show_sidebar(false);
                    }
                }
                Message::SwitchAdapterPowered(powered) => {
                    let powered_switch_row = clone.imp().powered_switch_row.get();
                    powered_switch_row.set_active(powered);
                },
                Message::SwitchAdapterDiscoverable(discoverable) => {
                    let discoverable_switch_row = clone.imp().discoverable_switch_row.get();
                    discoverable_switch_row.set_active(discoverable);
                },
                Message::SwitchAdapterName(new_alias, old_alias) => {
                    let default_controller_expander = clone.imp().default_controller_expander.get();
                    let listbox = default_controller_expander.last_child().unwrap().downcast::<gtk::Box>().unwrap(); 
                    let revealer = listbox.last_child().unwrap().downcast::<gtk::Revealer>().unwrap();
                    //println!("{:?}", revealer);
                    let listbox = revealer.last_child().unwrap().downcast::<gtk::ListBox>().unwrap();
                    //println!("{:?}", listbox);
                    let mut index = 0;
                    while listbox.clone().row_at_index(index) != None {
                        //println!("{}", index);
                        let action_row = listbox.clone().row_at_index(index).unwrap().downcast::<adw::ActionRow>().expect("cannot downcast to action row.");
                        //println!("{:?}", action_row.clone().title());
                        if action_row.clone().title().to_string() == old_alias.to_string() {
                            action_row.set_title(new_alias.as_str());
                        }
                        index += 1;
                    }
                    let adapter_name_entry = clone.imp().adapter_name_entry.get();

                    adapter_name_entry.set_text(new_alias.as_str()); // causes issue where it wants to reapply
                },
                Message::SwitchAdapterTimeout(timeout) => {
                    let timeout_time_adjustment = clone.imp().timeout_time_adjustment.get();
                    timeout_time_adjustment.set_value(timeout as f64);
                },
                Message::PopulateAdapterExpander(hashmap) => {
                    let default_controller_expander = clone.imp().default_controller_expander.get();
                    let listbox = default_controller_expander.last_child().unwrap().downcast::<gtk::Box>().unwrap().last_child().unwrap().downcast::<gtk::Revealer>().unwrap().last_child().unwrap().downcast::<gtk::ListBox>(); 
                    if listbox.clone().is_ok() {
                        while let Some(supposed_row) = listbox.clone().unwrap().last_child() {
                            listbox.clone().unwrap().remove(&supposed_row);
                        }
                    }

                    let adapter_aliases: Vec<String> = hashmap.clone().keys().cloned().collect();

                    let hashmap_clone = hashmap.clone();
                    for alias in adapter_aliases.clone() {
                        let row = adw::ActionRow::new();
                        let val = hashmap_clone.get(&alias).cloned();
                        let holder: String;
                        unsafe { holder = ORIGINAL_ADAPTER.to_string() }
                        let name = val.clone().unwrap_or(holder);
                        //println!("name is {}", name.clone());
                        //println!("alias is {}", alias.clone());

                        row.set_title(alias.as_str());
                                                
                        let suffix = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                        let icon = gtk::Image::new();
                        icon.set_icon_name(Some("check-plain-symbolic"));
                        suffix.append(&icon);
                        
                        unsafe {
                            if CURRENT_ADAPTER.to_string() == name.clone() {
                                suffix.show();
                            }
                            else {
                                suffix.hide();
                            }
                        } 

                        let listbox_clone = listbox.clone();

                        row.add_suffix(&suffix.clone());
                        row.set_activatable(true);
                        row.connect_activated(move |_| { 
                            let mut index = 0;
                            if listbox_clone.clone().is_ok() {
                                while listbox_clone.clone().unwrap().row_at_index(index) != None {
                                    //println!("{}", index);
                                    let action_row = listbox_clone.clone().unwrap().row_at_index(index).unwrap().downcast::<adw::ActionRow>().expect("cannot downcast to action row.");
                                    //println!("{:?}", action_row.clone().title());
                                    action_row.first_child().unwrap().last_child().unwrap().last_child().unwrap().hide();

                                    index += 1;
                                }
                            }
                            
                            unsafe {
                                CURRENT_ADAPTER = name.to_string();
                                println!("current adapter name is: {}", CURRENT_ADAPTER.clone());
                            }

                            suffix.show();
                        });
                        
                        default_controller_expander.add_row(&row);                        
                    }
                },
                Message::SetRefreshSensitive(sensitive) => {
                    let button = clone.imp().refresh_button.get();
                    button.set_sensitive(sensitive);
                },
            }
        
            glib::ControlFlow::Continue
        });
        

        let button = self.imp().refresh_button.get();
        let sender0 = sender.clone();
        button.connect_clicked(move |button| {
                button.set_sensitive(false);
                let sender_clone = sender0.clone();
                
                std::thread::spawn(move || {
                   std::thread::sleep(std::time::Duration::from_secs(2));
                   sender_clone.send(Message::SetRefreshSensitive(true)).expect("cannot send message");
                });
                unsafe {
                    if IS_CURRENTLY_LOOPING == true {
                        CAN_CONTINUE_LOOP = false;
                    }
                }
                
                get_avaiable_devices().expect("cannot refresh devices");
                println!("trying to available devices");
        });
        let main_listbox = self.imp().main_listbox.get();

        main_listbox.invalidate_sort();
        main_listbox.set_sort_func(|row_one, row_two| {
            let binding_one = row_one.clone().downcast::<adw::ActionRow>().unwrap().title();
            let binding_two = row_two.clone().downcast::<adw::ActionRow>().unwrap().title();
            
            let hashmap: HashMap<String, i32>;
            unsafe {
                hashmap = RSSI_LUT.clone().unwrap();
            }
            let rssi_one = hashmap.get(&binding_one.clone().to_string()).unwrap_or(&(-100 as i32));
            let rssi_two = hashmap.get(&binding_two.clone().to_string()).unwrap_or(&(-100 as i32));
            //println!("rssi one {} rssi two {}", rssi_one, rssi_two);
            
            let mut one = binding_one.as_str();
            let mut two = binding_two.as_str();
            
        	let one_str = one.to_lowercase();
            let two_str = two.to_lowercase();
            
            one = one_str.as_str();
            two = two_str.as_str();
            
            let name_result = one.cmp(&two);
            let rssi_result = rssi_two.cmp(&rssi_one);
            //println!("rssi result {:?}", rssi_result);
            
            let final_result = if rssi_result == std::cmp::Ordering::Equal {
                name_result
            }
            else {
                rssi_result
            };
            //println!("rssi result {:?}", final_result);
            
            final_result.into()
        });
        
        let connected_switch_row = self.imp().connected_switch_row.get();
        let sender1 = sender.clone();
        connected_switch_row.set_activatable(true);
        connected_switch_row.connect_activated(move |_| {
            let address: bluer::Address;
            unsafe { address = CURRENT_ADDRESS }
            
            let sender_clone = sender1.clone();
            
            std::thread::spawn(move || {
                let active = match set_device_active(address) {
                    Ok(bool) => {
                        bool
                    },
                    Err(err) => {
                        println!("cannot set device active: {}", err);
                        false
                    },
                };
                std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                sender_clone.send(Message::SwitchActive(active)).expect("cannot send message");
            });
        });
        
        let blocked_row = self.imp().blocked_row.get();
        let sender2 = sender.clone();
        blocked_row.connect_activated(move |_| {
            let address: bluer::Address;
            unsafe { address = CURRENT_ADDRESS }
            
            let sender_clone = sender2.clone();
            
            std::thread::spawn(move || {
                let blocked = match set_device_blocked(address) {
                    Ok(bool) => {
                        bool
                    },
                    Err(err) => {
                        println!("cannot set device blocked: {}", err);
                        false
                    },
                };
                std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                sender_clone.send(Message::SwitchBlocked(blocked)).expect("cannot send message");
            });
        });

        let trusted_row = self.imp().trusted_row.get();
        let sender3 = sender.clone();
        trusted_row.connect_activated(move |_| {
            let address: bluer::Address;
            unsafe { address = CURRENT_ADDRESS }
            
            let sender_clone = sender3.clone();
            
            std::thread::spawn(move || {
                let trusted = match set_device_trusted(address) {
                    Ok(bool) => {
                        bool
                    },
                    Err(err) => {
                        println!("cannot set device trusted: {}", err);
                        false
                    },
                };
                std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                sender_clone.send(Message::SwitchTrusted(trusted)).expect("cannot send message");
            });
        });

        let device_name_entry = self.imp().device_name_entry.get();
        let sender4 = sender.clone();
        device_name_entry.connect_apply(move |entry| {
            let address: bluer::Address;
            unsafe { address = CURRENT_ADDRESS }
            let name = entry.text().to_string();

            let sender_clone = sender4.clone();

            std::thread::spawn(move || {
                let name = match set_device_name(address, name) {
                    Ok(name) => {
                        name
                    },
                    Err(err) => {
                        println!("cannot set device alias: {}", err);
                        return;
                    },
                };
                sender_clone.send(Message::SwitchName(name)).expect("cannot send message");
            });
        });

        let remove_device_button = self.imp().remove_device_button.get();
        let sender4 = sender.clone();
        remove_device_button.connect_clicked(move |_| {
            let sender_clone = sender4.clone();
            
            let address: bluer::Address;
            unsafe { address = CURRENT_ADDRESS }

            std::thread::spawn(move || {
                let name = match remove_device(address) {
                    Ok(name) => {
                        name
                    },
                    Err(err) => {
                        println!("cannot remove device: {}", err);
                        return;
                    },
                };
                sender_clone.send(Message::RemoveDevice(name)).expect("can't send message");
            });
        });

        let bluetooth_settings_row = self.imp().bluetooth_settings_row.get();
        let self_clone2 = self.clone();
        bluetooth_settings_row.connect_activated(move |_| {
            std::thread::spawn(move || {
                let adapter_names = populate_adapter_expander();
                if adapter_names.is_ok() {
                    get_adapter_properties(adapter_names.unwrap()).expect("cannot get adapter properties {}");
                }
            });
            
            let main_listbox = self_clone2.imp().main_listbox.get();
            main_listbox.unselect_all();
            
            let main_stack = self_clone2.imp().main_stack.get();
            let pages = main_stack.pages();
            pages.select_item(1, true);

            let split_view = self_clone2.imp().split_view.get();
            if split_view.is_collapsed() {
                split_view.set_show_sidebar(false);
            }
        });
        bluetooth_settings_row.emit_activate();

        let powered_switch_row = self.imp().powered_switch_row.get();
        let sender5 = sender.clone();
        powered_switch_row.connect_activated(move |_| {
            let sender_clone = sender5.clone();

            std::thread::spawn(move || {
                let powered = match set_adapter_powered() {
                    Ok(bool) => {
                        bool
                    },
                    Err(err) => {
                        println!("cannot set adapter powered: {}", err);
                        false
                    },
                };
    
                std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                sender_clone.send(Message::SwitchAdapterPowered(powered)).expect("can't send message");
                sender_clone.send(Message::SetRefreshSensitive(false)).expect("cannot send message");
                std::thread::sleep(std::time::Duration::from_secs(2));
                sender_clone.send(Message::SetRefreshSensitive(true)).expect("cannot send message");
            });
        });

        let discoverable_switch_row = self.imp().discoverable_switch_row.get();
       	let sender6 = sender.clone();
       	discoverable_switch_row.connect_activated(move |_| {
            let sender_clone = sender6.clone();
            
            std::thread::spawn(move || {
                let discoverable = match set_adapter_discoverable() {
                    Ok(bool) => {
                        bool
                    },
                    Err(err) => {
                        println!("cannot set adapter discoverable: {}", err);
                        false
                    },
                };
 
                std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                sender_clone.send(Message::SwitchAdapterDiscoverable(discoverable)).expect("can't send message"); 	
            });
        });

        let adapter_name_entry = self.imp().adapter_name_entry.get();
        let sender7 = sender.clone();
        adapter_name_entry.connect_apply(move |entry| {
            let new_name = entry.text().to_string();

            let sender_clone = sender7.clone();

            std::thread::spawn(move || {
                let name = match set_adapter_name(new_name) {
                    Ok(name) => {
                        name
                    },
                    Err(err) => {
                        println!("cannot set adapter alias: {}", err);
                        return;
                    },
                };
                sender_clone.send(Message::SwitchAdapterName(name[0].clone(), name[1].clone())).expect("cannot send message");
            });
        });

        let timeout_adjustment = self.imp().timeout_time_adjustment.get();
        let sender8 = sender.clone();
        timeout_adjustment.connect_value_changed(move |adjustment| {
            let value = adjustment.value();

            let new_value = set_timeout_duration(value as u32).expect("cannot set timeout duration.");

            sender8.send(Message::SwitchAdapterTimeout(new_value)).expect("cannot send message");
        });

        let split_view = self.imp().split_view.get();
        let self_clone3 = self.clone();
        split_view.connect_show_sidebar_notify(move |view| {
            let show_sidebar_button = self_clone3.imp().show_sidebar_button.get();
            show_sidebar_button.set_active(view.shows_sidebar());
        });

        Ok(())
    }

    fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let size = (self.size(gtk::Orientation::Horizontal), self.size(gtk::Orientation::Vertical));
        // let size = self.SIZE
        let settings = self.imp().settings.get().expect("cannot get settings, setup improperly?");

        println!("size is {:?}", size);

        settings.set_int("window-width", size.0)?;
        settings.set_int("window-height", size.1)?;
        settings.set_boolean("window-maximized", self.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        let settings = self.imp().settings.get().expect("cannot get settings, setup improperly?");
        
        let width = settings.int("window-width");
        let height = settings.int("window-height");
        let maximized = settings.boolean("window-maximized");

        println!("new size is {:?}", (width, height));

        self.set_default_size(width, height);

        self.set_maximized(maximized);
    }
}

/// Gets the available devices around this device. 
/// For now it is manual discovery (by hitting refresh) but should be automated preferably.
#[tokio::main]
async fn get_avaiable_devices() -> bluer::Result<()> {
    std::thread::spawn(move || {
        match get_devices_continuous() {
            Ok(()) => {
                println!("stopped getting devices (gracefully)");
            }
            Err(err) => {
                println!("cannot get devices, {}", err);
            }
        };
    });

    Ok(())
}

/// Set the associated with `address` device's state, between connected and not 
/// connected depending on what was already the case.
/// A little funky and needs fixing but works for now.
#[tokio::main]
async fn set_device_active(address: bluer::Address) -> bluer::Result<bool> {
    let current_session = bluer::Session::new().await?;
    let adapter_name: String;
    unsafe {
        adapter_name = CURRENT_ADAPTER.clone();
    }
    let adapter = current_session.adapter(adapter_name.as_str())?;

    let device = adapter.device(address)?;

    let state = device.is_connected().await?;

    if state == true {
        device.disconnect().await?;
    }
    else if state == false {
        device.connect().await?;
    }

    let updated_state = device.is_connected().await?;

    println!("set state {} for device {}\n", updated_state, device.address());

    Ok(updated_state)
}

/// Set's the device's blocked state based on what was already the case.
/// Basically stops all connections and requests if the device is blocked.
#[tokio::main]
async fn set_device_blocked(address: bluer::Address)  -> bluer::Result<bool> {
    let current_session = bluer::Session::new().await?;
    let adapter_name: String;
    unsafe {
        adapter_name = CURRENT_ADAPTER.clone();
    }
    let adapter = current_session.adapter(adapter_name.as_str())?;

    let device = adapter.device(address)?;

    let blocked = device.is_blocked().await?;

    if blocked == true {
        device.set_blocked(false).await?;
    }
    else if blocked == false {
        device.set_blocked(true).await?
    }

    let new_blocked = device.is_blocked().await?;

    println!("setting blocked {} for device {}", new_blocked, device.address());

    Ok(new_blocked)
}

/// Sets the device's trusted state depending on what was already the case.
/// If trusted, connections to the device won't need pin/passkey everytime.
#[tokio::main]
async fn set_device_trusted(address: bluer::Address) -> bluer::Result<bool> {
    let current_session = bluer::Session::new().await?;
    let adapter_name: String;
    unsafe {
        adapter_name = CURRENT_ADAPTER.clone();
    }
    let adapter = current_session.adapter(adapter_name.as_str())?;

    let device = adapter.device(address).expect("cannot get device {}");

    let trusted = device.is_trusted().await?;

    if trusted == true {
        device.set_trusted(false).await.expect("cannot set untrusted {}");
    }
    else if trusted == false {
        device.set_trusted(true).await.expect("cannot set trusted {}");
    }

    let new_trusted = device.is_trusted().await?;
    //self.imp().connected_switch_row.get().set_active();

    println!("setting trusted {} for device {}", new_trusted, device.address());

    Ok(new_trusted)
}

/// Sets the currently selected device's name, updateing the entry and listboxrow accordingly.
#[tokio::main]
async fn set_device_name(address: bluer::Address, name: String) -> bluer::Result<String> {
    let current_session = bluer::Session::new().await?;
    let adapter_name: String;
    unsafe {
        adapter_name = CURRENT_ADAPTER.clone();
    }
    let adapter = current_session.adapter(adapter_name.as_str())?;

    let device = adapter.device(address).expect("cannot get device {}");

    let mut lut: HashMap<bluer::Address, String>;
    unsafe {
        lut = DEVICES_LUT.clone().unwrap();
        lut.remove(&address);
        lut.insert(address, name.clone());
        DEVICES_LUT = Some(lut);
    }

    device.set_alias(name).await?;
    let current_alias = device.alias().await?;
    Ok(current_alias)
}

#[tokio::main]
async fn set_adapter_powered() -> bluer::Result<bool> {
    let current_session = bluer::Session::new().await?;
    let adapter_name: String;
    unsafe {
        adapter_name = CURRENT_ADAPTER.clone();
    }
    let adapter = current_session.adapter(adapter_name.as_str())?;
    
    let current = adapter.is_powered().await?;
    println!("trying to set adapter powered");
    
    let powered = match adapter.set_powered(!current).await {
        Ok(()) => {
            !current
        }
        Err(err) => {
            println!("cannot set adapter powered {}", err);
            false
        }
    };
    
    Ok(powered)
}

#[tokio::main]
async fn set_adapter_discoverable() -> bluer::Result<bool> {
    let current_session = bluer::Session::new().await?;
    let adapter_name: String;
    unsafe {
        adapter_name = CURRENT_ADAPTER.clone();
    }
    let adapter = current_session.adapter(adapter_name.as_str())?;
    
    let current = adapter.is_discoverable().await?;
    let discoverable = match adapter.set_discoverable(!current).await {
        Ok(()) => {
            !current
        }
        Err(err) => {
            println!("cannot set adapter discoverable {}", err);
            false
        }
    };

    println!("discoverable is: {}", discoverable);

    Ok(discoverable)
}

#[tokio::main]
async fn add_child_row(device: bluer::Device) -> bluer::Result<adw::ActionRow> {
    let child_row = adw::ActionRow::new();
    let current_device = device.clone();
    println!("{:?}", device.name().await?);

    let name = current_device.alias().await?;
    let address = current_device.address();
    let rssi = current_device.rssi().await?;

    child_row.set_title(name.clone().as_str());
    child_row.set_activatable(true);
    //child_row.set_subtitle(&device.address().to_string());
    
    let suffix_box = gtk::Box::new(gtk::Orientation::Horizontal, 16);
    let rssi_icon = gtk::Image::new();

    
    let icon_name = match rssi {
        None => {
            "rssi-none-symbolic"
        },
        Some(n) if (n * -1) <= 50 => {
            "rssi-high-symbolic"
        } 
        Some(n) if (n * -1) <= 60 => {
            "rssi-medium-symbolic"
        }
        Some(n) if (n * -1) <= 70 => {
            "rssi-low-symbolic"
        }
        Some(n) if (n * -1) <= 80 => {
            "rssi-dead-symbolic"
        }
        Some(n) if (n * -1) <= 90 => {
            "rssi-none-symbolic"
        }
        Some(_) => {
            "rssi-not-found-symbolic"
        }
    };
    rssi_icon.set_icon_name(Some(icon_name));
    println!("rssi is: {:?}", rssi.clone());
    
    suffix_box.append(&rssi_icon);
    child_row.add_suffix(&suffix_box);
    unsafe {
        let mut devices_lut = DEVICES_LUT.clone().unwrap();
        devices_lut.insert(address, name.clone());
        //println!("lut (add) is: {:?}", devices_lut);
        DEVICES_LUT = Some(devices_lut);
        //println!("big lut (add) is: {:?}", DEVICES_LUT.clone());
        let mut rssi_lut = RSSI_LUT.clone().unwrap();
        rssi_lut.insert(name, rssi.unwrap_or(-100).into());
        RSSI_LUT = Some(rssi_lut);
    } 

    child_row.connect_activated(move |row| {        
        unsafe {
            CURRENT_INDEX = row.index();
            CURRENT_ADDRESS = device.address();
        }

        let address: bluer::Address;
        unsafe { address = CURRENT_ADDRESS }
        
        std::thread::spawn(move || {
            get_device_properties(address).expect("can't get device properties");
        });
    });

    Ok(child_row)
}


/// Gets the the device associates with `address`, and then retrieves the properties of that device.
/// Its an async method so you have to `await` it else it won't do anything.
/// Still has an issue when trying to select other devices after first device.
#[tokio::main]
async fn get_device_properties(address: bluer::Address) -> bluer::Result<()> {
    let current_session = bluer::Session::new().await?;
    let adapter_name: String;
    unsafe {
        adapter_name = CURRENT_ADAPTER.clone();
    }
    let adapter = current_session.adapter(adapter_name.as_str())?;

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

    let sender: Sender<Message>;
    unsafe { sender = CURRENT_SENDER.clone().unwrap() }
    
    sender.send(Message::SwitchPage(Some(alias), Some(icon_name))).expect("cannot send message {}");
    sender.send(Message::SwitchActive(is_active)).expect("cannot send message {}");
    sender.send(Message::SwitchBlocked(is_blocked)).expect("cannot send message {}");
    sender.send(Message::SwitchTrusted(is_trusted)).expect("cannot send message {}");
    
    println!("the devices properties have been gotten with state: {}", is_active);

    Ok(())
}

#[tokio::main]
async fn populate_adapter_expander() -> bluer::Result<HashMap<String, String>> {
    let current_session = bluer::Session::new().await?;
    let adapter_names = current_session.adapter_names().await?;
    let mut alias_name_hashmap: HashMap<String, String> = HashMap::new();

    for name in adapter_names.clone() {
        let adapter = current_session.adapter(name.as_str())?;
        let address = adapter.address().await?; 
        //println!("adapter address is: {}", address.clone());
        
        std::process::Command::new("bluetoothctl").arg("select").arg(address.to_string());
		let old_output = String::from_utf8(std::process::Command::new("bluetoothctl").arg("show").output().expect("cant do so").stdout).expect("nah");
  		let old_name = old_output.lines().nth(2).unwrap().replace("\tAlias: ", "").replace("hci0 name changed: ", "");
	   	
        let alias = &old_name[0..old_name.find("AdvertisementMonitor").unwrap_or(old_name.len())];
        
        alias_name_hashmap.insert(alias.clone().to_string(), name.clone().to_string());
        //println!("adapter alias is: {}", alias)
    }

    unsafe {
        ADAPTERS_LUT = Some(alias_name_hashmap.clone());
    }

    println!("entire adapter names list: {:?}", alias_name_hashmap);
    Ok(alias_name_hashmap)
}

#[tokio::main]
async fn get_adapter_properties(adapters_hashmap: HashMap<String, String>) -> bluer::Result<()> {
    let current_session = bluer::Session::new().await?;
    let adapter_name: String;
    unsafe {
        adapter_name = CURRENT_ADAPTER.clone();
    }
    let adapter = current_session.adapter(adapter_name.as_str())?;

    let is_powered = adapter.is_powered().await?;
    let is_discoverable = adapter.is_discoverable().await?;
	let old_output = String::from_utf8(std::process::Command::new("bluetoothctl").arg("show").output().expect("cant do so").stdout).expect("nah");
  	let old_name = old_output.lines().nth(2).unwrap().replace("\tAlias: ", "");
   	let name = &old_name[0..old_name.find("AdvertisementMonitor").unwrap_or(old_name.len())];
    let timeout = adapter.discoverable_timeout().await? / 60;

    let sender: Sender<Message>;
    unsafe { sender = CURRENT_SENDER.clone().unwrap() }
    
    sender.send(Message::PopulateAdapterExpander(adapters_hashmap)).expect("cannot send message {}");
    //println!("sent populate adapters message");
    sender.send(Message::SwitchAdapterPowered(is_powered)).expect("cannot send message {}");
    sender.send(Message::SwitchAdapterDiscoverable(is_discoverable)).expect("cannot send message {}");
    sender.send(Message::SwitchAdapterName(name.clone().to_string(), name.to_string())).expect("cannot send message {}");
    sender.send(Message::SwitchAdapterTimeout(timeout)).expect("cannot send message {}");
    
    println!("the adapter properties have been updated.");

    Ok(())
}

#[tokio::main]
async fn set_adapter_name(name: String) -> bluer::Result<Vec<String>> {
    let old_output = String::from_utf8(std::process::Command::new("bluetoothctl").arg("show").output().expect("cant do so").stdout).expect("nah");
   	let old_name = old_output.lines().nth(2).unwrap().replace("\tAlias: ", "");
	let old_alias = old_name[0..old_name.find("AdvertisementMonitor").unwrap_or(old_name.len())].to_string();
    //println!("old alias is: {}", old_alias.to_string());

    let new_output = String::from_utf8(std::process::Command::new("bluetoothctl").arg("system-alias").arg(name).output().expect("cant do so").stdout).expect("nah");
   	let current_alias = new_output.replace("Changing ", "").replace(" succeeded\n", "");
	let name = current_alias[0..current_alias.find("AdvertisementMonitor").unwrap_or(current_alias.len())].to_string();

    let mut lut: HashMap<String, String>;
    unsafe {
        lut = ADAPTERS_LUT.clone().unwrap();
        let bluetooth_name = lut.get(&old_alias.clone()).unwrap().to_string();

        lut.remove(&old_alias.clone());
        lut.insert(name.clone(), bluetooth_name);
        ADAPTERS_LUT = Some(lut);
    }

    //println!("name is: {}", name.clone());
    Ok(vec!(name, old_alias))
}

#[tokio::main]
async fn remove_device(device_address: bluer::Address) -> bluer::Result<String> {
    let current_session = bluer::Session::new().await?;
    let adapter_name: String;
    unsafe {
        adapter_name = CURRENT_ADAPTER.clone();
    }
    let adapter = current_session.adapter(adapter_name.as_str())?;

    let device = adapter.device(device_address);
    
    let exists = match device.clone() {
        Ok(_) => {
            true
        },
        Err(err) => {
            println!("device doesn't exist {}", err);
            false
        },
    };

    let name = device.unwrap().alias().await?;
    if exists {
        adapter.remove_device(device_address).await.expect("cannot remove device");
        unsafe {
            let mut devices_lut = DEVICES_LUT.clone().unwrap();
            if devices_lut.contains_key(&device_address) {
                devices_lut.remove(&device_address);
                DEVICES_LUT = Some(devices_lut);
            }
        }
    }

    Ok(name)
}


#[tokio::main]
async fn get_devices_continuous() -> bluer::Result<()> {
    let current_session = bluer::Session::new().await?;
    let adapter_name: String;
    unsafe {
        adapter_name = CURRENT_ADAPTER.clone();
    }
    let adapter = current_session.adapter(adapter_name.as_str())?;

	let filter = bluer::DiscoveryFilter {
        transport: bluer::DiscoveryTransport::Auto,
        ..Default::default()
    };
    adapter.set_discovery_filter(filter).await?;
	
    let device_events = adapter.discover_devices().await?;
    pin_mut!(device_events);    
    let sender: Sender<Message>;
    unsafe { sender = CURRENT_SENDER.clone().unwrap() }
    
    let mut all_change_events = SelectAll::new();
    
    //unsafe { CAN_CONTINUE_LOOP = true }

    while adapter.is_powered().await? == true  {
        unsafe { 
            if CAN_CONTINUE_LOOP == false {
                //println!("broke");
                break;    
            } 
            IS_CURRENTLY_LOOPING = true;
        }
        tokio::select! {
            Some(device_event) = device_events.next() => {
                match device_event {
                    AdapterEvent::DeviceAdded(addr) => {
		                if adapter.is_powered().await? == true {
	                        let supposed_device = adapter.device(addr);
	    
                            let devices_lut: HashMap<bluer::Address, String>;
                            unsafe {
                                devices_lut =  DEVICES_LUT.clone().unwrap();
                            }

                            if !devices_lut.contains_key(&addr) {
                                if supposed_device.is_err() {
                                    Err( supposed_device.clone().err().unwrap() ).unwrap()
                                }
                                let added_device = supposed_device.unwrap();
                                
                                sender.send(Message::AddRow(added_device)).expect("cannot send message {}"); 
                                //println!("supposedly sent");
                                
                                let device = adapter.device(addr)?;
                                let change_events = device.events().await?.map(move |evt| (addr, evt));
                                all_change_events.push(change_events);
                            }
                            else {
                                println!("device already exists, not adding again.");
                            }

		                }
                    }
                    AdapterEvent::DeviceRemoved(addr) => {
   		                if adapter.is_powered().await? == true {
                        	let sender: Sender<Message>;
                            unsafe { sender = CURRENT_SENDER.clone().unwrap() }

                            let mut devices_lut: HashMap<bluer::Address, String>;
                            unsafe {
                                devices_lut = DEVICES_LUT.clone().unwrap();
                                //println!("big lut (removed) is: {:?}", DEVICES_LUT.clone());
                            } 

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
                            
                            sender.send(Message::RemoveDevice(device_name.clone())).expect("cannot send message {}"); 
                            println!("Device removed: {:?} {}", addr, device_name.clone());    
						}
                    },
                    AdapterEvent::PropertyChanged(AdapterProperty::Powered(powered)) => {
                        let sender: Sender<Message>;
                        unsafe { sender = CURRENT_SENDER.clone().unwrap() }
                        std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                        sender.send(Message::SwitchAdapterPowered(powered)).expect("cannot send message {}"); 
                        println!("powered switch to {}", powered);
                    },
                    AdapterEvent::PropertyChanged(AdapterProperty::Discoverable(discoverable)) => {
                        let sender: Sender<Message>;
                        unsafe { sender = CURRENT_SENDER.clone().unwrap() }
                        std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                        sender.send(Message::SwitchAdapterDiscoverable(discoverable)).expect("cannot send message {}"); 
                        println!("discoverable switch to {}", discoverable);
                    },
                    event => {
                        println!("unhandled event: {:?}", event);
                    }
                }
            }
            Some((addr, DeviceEvent::PropertyChanged(property))) = all_change_events.next() => {
                match property {
                    DeviceProperty::Connected(connected) => {
                        let current_address: bluer::Address;
                        unsafe { current_address = CURRENT_ADDRESS }
                        
                        if addr == current_address {
                            let sender: Sender<Message>;
                            unsafe { sender = CURRENT_SENDER.clone().unwrap() }

                            std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                            sender.send(Message::SwitchActive(connected)).expect("cannot send message");
                        }
                    },
                    DeviceProperty::Trusted(trusted) => {
                        let current_address: bluer::Address;
                        unsafe { current_address = CURRENT_ADDRESS }
                        
                        if addr == current_address {
                            let sender: Sender<Message>;
                            unsafe { sender = CURRENT_SENDER.clone().unwrap() }

                            std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                            sender.send(Message::SwitchTrusted(trusted)).expect("cannot send message");
                        }
                    },
                    DeviceProperty::Blocked(blocked) => {
                        let current_address: bluer::Address;
                        unsafe { current_address = CURRENT_ADDRESS }
                        
                        if addr == current_address {
                            let sender: Sender<Message>;
                            unsafe { sender = CURRENT_SENDER.clone().unwrap() }

                            std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                            sender.send(Message::SwitchBlocked(blocked)).expect("cannot send message");
                        }
                    },
                    DeviceProperty::Alias(name) => {
                        let current_address: bluer::Address;
                        unsafe { current_address = CURRENT_ADDRESS }
                        
                        if addr == current_address {
                            let sender: Sender<Message>;
                            unsafe { sender = CURRENT_SENDER.clone().unwrap() }

                            std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                            sender.send(Message::SwitchName(name.clone())).expect("cannot send message");
                            sender.send(Message::SwitchPage(Some(name.clone()), None)).expect("cannot send message");
                        }
                    },
                    DeviceProperty::Icon(icon) => {
                        let current_address: bluer::Address;
                        unsafe { current_address = CURRENT_ADDRESS }
                        
                        if addr == current_address {
                            let sender: Sender<Message>;
                            unsafe { sender = CURRENT_SENDER.clone().unwrap() }

                            std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                            sender.send(Message::SwitchPage(None, Some(icon))).expect("cannot send message");
                        }
                    },
                    /*DeviceProperty::Rssi(rssi) => {
                        let current_address: bluer::Address;
                        unsafe { current_address = CURRENT_ADDRESS }
                        
                        if addr == current_address {
                            let sender: Sender<Message>;
                            unsafe { sender = CURRENT_SENDER.clone().unwrap() }

                            std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                            sender.send(Message::SwitchPage(None, Some(icon))).expect("cannot send message");
                        }
                    },*/
                    _ => (),
                }
            }
            else => break
        }
    }
    println!("exited loop");
    unsafe { 
        IS_CURRENTLY_LOOPING = false;
        CAN_CONTINUE_LOOP = true;
    }
    Err(bluer::Error { kind: bluer::ErrorKind::Failed, message: "cannot get devices, adapter isn't on?".to_string() })
}

#[tokio::main]
async fn set_timeout_duration(timeout: u32) -> bluer::Result<u32> {
    let current_session = bluer::Session::new().await?;

    let adapter_name: String;
    unsafe {
        adapter_name = CURRENT_ADAPTER.clone();
    }
    let adapter = current_session.adapter(adapter_name.as_str())?;

    adapter.set_discoverable_timeout(timeout * 60).await.expect("cannot set adapter timeout.");

    Ok(adapter.discoverable_timeout().await?)
}




