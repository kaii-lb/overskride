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
use gtk::glib::{Sender, clone};
use gtk::{gio, glib};

use std::cell::{OnceCell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use gtk::glib::SignalHandlerId;

use crate::device_action_row::DeviceActionRow;
use crate::{bluetooth_settings, device, connected_switch_row::ConnectedSwitchRow};
use crate::message::Message;

// U N S A F E T Y 
pub static mut CURRENT_ADDRESS: bluer::Address = bluer::Address::any();
static mut CURRENT_INDEX: i32 = 0;
static mut CURRENT_SENDER: Option<Sender<Message>> = None;
static mut RSSI_LUT: Option<HashMap<String, i32>> = None;
static mut ORIGINAL_ADAPTER: String = String::new();
pub static mut CURRENT_ADAPTER: String = String::new();
pub static mut DEVICES_LUT: Option<HashMap<bluer::Address, String>> = None;
pub static mut ADAPTERS_LUT: Option<HashMap<String, String>> = None;
pub static mut CURRENTLY_LOOPING: bool = false;
pub static mut DISPLAYING_DIALOG: bool = false;
pub static mut PIN_CODE: String = String::new();
pub static mut PASS_KEY: u32 = 0;
pub static mut CONFIRMATION_AUTHORIZATION: bool = false;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/kaii_lb/Overskride/gtk/window.ui")]
    pub struct OverskrideWindow {
        #[template_child]
        pub main_listbox: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub refresh_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub connected_switch_row: TemplateChild<ConnectedSwitchRow>,
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
        pub default_controller_expander: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub split_view: TemplateChild<adw::OverlaySplitView>,
        #[template_child]
        pub show_sidebar_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub listbox_image_box: TemplateChild<gtk::Box>,

        pub settings: OnceCell<Settings>,
        pub display_pass_key_dialog: RefCell<Option<adw::MessageDialog>>,
        pub index: RefCell<u32>,
        pub timeout_signal_id: OnceCell<SignalHandlerId>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for OverskrideWindow {
        const NAME: &'static str = "OverskrideWindow";
        type Type = super::OverskrideWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            ConnectedSwitchRow::ensure_type();

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
        
        win.setup();
        
        win
    }

    fn setup_settings(&self) {
        let settings = Settings::new("io.github.kaii_lb.Overskride");
        self.imp().settings.set(settings).expect("settings not setup");
    }

    /// Sets up the application. Basically it binds actions to stuff and updates what needs to be updated.
    fn setup(&self) {
        let (sender, receiver) = glib::MainContext::channel::<Message>(glib::Priority::default());
        
        self.pre_setup(sender.clone()).expect("cannot start presetup, something got REALLY fucked");

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

                    std::thread::sleep(std::time::Duration::from_millis(200));
                    connected_switch_row.set_switch_active(active);
                },
                Message::SwitchActiveSpinner(spinning) => {
                    let connected_switch_row = clone.imp().connected_switch_row.get();
                    
                    connected_switch_row.set_spinning(spinning);
                },
                Message::SwitchName(alias, optional_old_alias) => {
                    let list_box = clone.imp().main_listbox.get();
                    let index = unsafe { 
                    	CURRENT_INDEX 
                    };
                    let mut listbox_index = 0;

                    if optional_old_alias.is_none() {
                        if let Some(some_row) = list_box.row_at_index(index) {
                            let action_row = some_row.downcast::<adw::ActionRow>().unwrap();
                            action_row.set_title(alias.as_str());
                        }
                    }
                    else {
                        while let Some(row) = list_box.clone().row_at_index(listbox_index) {
                            //println!("{}", index);
                            let action_row = row.downcast::<adw::ActionRow>().expect("cannot downcast to action row.");
                            //println!("{:?}", action_row.clone().title());
                            if action_row.clone().title() == optional_old_alias.clone().unwrap() {
                                action_row.set_title(alias.as_str());
                            }

                            listbox_index += 1;
                        }
                    }
                },
                Message::SwitchRssi(device_name, rssi) => {
                    let list_box = clone.imp().main_listbox.get();
                    let mut listbox_index = 0;
                    
                    while let Some(row) = list_box.clone().row_at_index(listbox_index) {
                        //println!("{}", index);
                        let action_row = row.downcast::<DeviceActionRow>().expect("cannot downcast to device action row.");
                        //println!("{:?}", action_row.clone().title());
                        
                        println!("device {}, with rssi {} changed", device_name.clone(), rssi);

                        if action_row.clone().title() == device_name {
                            action_row.set_rssi(rssi);
                            action_row.update_rssi_icon();
                        }

                        listbox_index += 1;
                    }
                },
                Message::AddRow(device) => {
                    let row = add_child_row(device);
                    if let Ok(ok_row) = row {
                        let main_listbox = clone.imp().main_listbox.get();
                        main_listbox.append(&ok_row);
                        main_listbox.invalidate_sort();
                    }
                },
                Message::RemoveDevice(name) => {
                    let listbox = clone.clone().imp().main_listbox.get();
                    let mut index = 0;
                    while let Some(row) = listbox.clone().row_at_index(index) {
                        // println!("{}", index);
                        let action_row = row.downcast::<adw::ActionRow>().expect("cannot downcast to action row.");
                        // println!("{:?}", action_row.clone());
                        if action_row.clone().title() == name {
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
                    
                    if let Some(name) = alias {
	                    entry_row.set_text(name.as_str());
	                    device_title.set_text(name.as_str());
                    }

					if let Some(icon) = icon_name {
	                    let final_icon_name = icon.clone() + "-symbolic";

	                    device_icon.set_icon_name(Some(final_icon_name.as_str()));
	                    println!("icon name is: {}", icon);	
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
                    
                    let listbox = revealer.last_child().unwrap().downcast::<gtk::ListBox>().unwrap();
                    
                    let mut index = 0;
                    while let Some(row) = listbox.clone().row_at_index(index) {
                        let action_row = row.downcast::<adw::ActionRow>().expect("cannot downcast to action row.");
                    
                        if action_row.clone().title() == old_alias {
                            action_row.set_title(new_alias.as_str());
                        }
                        index += 1;
                    }
                    let adapter_name_entry = clone.imp().adapter_name_entry.get();

                    adapter_name_entry.set_text(new_alias.as_str());
                },
                Message::SwitchAdapterTimeout(timeout) => {
                    let timeout_time_adjustment = clone.imp().timeout_time_adjustment.get();
                    timeout_time_adjustment.block_signal(clone.imp().timeout_signal_id.get().expect("cannot get signal id"));
                    timeout_time_adjustment.set_value(timeout as f64);
                    timeout_time_adjustment.unblock_signal(clone.imp().timeout_signal_id.get().expect("cannot get signal id"));
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
                        let holder = unsafe {
                        	ORIGINAL_ADAPTER.to_string()
                        };
                        
                        let name = val.clone().unwrap_or(holder);
                        //println!("name is {}", name.clone());
                        //println!("alias is {}", alias.clone());

                        row.set_title(alias.as_str());
                                                
                        let suffix = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                        let icon = gtk::Image::new();
                        icon.set_icon_name(Some("check-plain-symbolic"));
                        suffix.append(&icon);
                        
                        unsafe {
                            if CURRENT_ADAPTER == name.clone() {
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
                                while let Some(row) = listbox_clone.clone().unwrap().row_at_index(index) {
                                    //println!("{}", index);
                                    let action_row = row.downcast::<adw::ActionRow>().expect("cannot downcast to action row.");
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
                Message::PopupError(string, priority, state) => {
                    let toast_overlay = clone.imp().toast_overlay.get();
                    let toast = adw::Toast::new("");

                    let custom_title = gtk::Label::new(Some(string.as_str()));
                    
                    toast.set_priority(priority);
                    match priority {
                        adw::ToastPriority::High => {
                            toast.set_timeout(5);
                            custom_title.set_css_classes(&["warning", state.as_str()]);
                        },
                        _ => {
                            toast.set_timeout(3);
                            custom_title.set_css_classes(&[state.as_str()]);
                        }
                    }
                    toast.set_custom_title(Some(&custom_title));

                    toast_overlay.add_toast(toast);
                },
                Message::UpdateListBoxImage() => {
                    let listbox_image_box = clone.imp().listbox_image_box.get();
                    let main_listbox = clone.imp().main_listbox.get();

                    let exists = main_listbox.row_at_index(0).is_some();

                    if exists {
                        listbox_image_box.set_visible(false);
                        main_listbox.set_visible(true);
                    }
                    else {
                        listbox_image_box.set_visible(true);
                        main_listbox.set_visible(false);
                    }
                },
                Message::RequestPinCode(request) => {
                    let device: String;
                    let adapter: String;
                    unsafe {
                        device = DEVICES_LUT.clone().unwrap().get(&request.device).unwrap_or(&"Unknown Device".to_string()).to_string();
                        adapter = ADAPTERS_LUT.clone().unwrap().get(&request.adapter).unwrap_or(&"Unknown Adapter".to_string()).to_string();
                        DISPLAYING_DIALOG = true
                    }
            
                    let body = device + "has requested pairing on " + adapter.as_str() + ", please enter the correct pin code.";
                    let popup = adw::MessageDialog::new(Some(&clone), Some("Pin Code Requested"), Some(body.as_str()));
            
                    // popup.set_modal(true);
                    popup.set_destroy_with_parent(true);
                    
                    popup.add_response("cancle", "Cancel");
                    popup.add_response("confirm", "Confirm");
                    popup.set_response_appearance("confirm", adw::ResponseAppearance::Suggested);
                    popup.set_default_response(Some("confirm"));
                    popup.set_close_response("cancel");
            
                    let entry = gtk::Entry::new();
                    entry.set_placeholder_text(Some("12345 or abcde"));
                    popup.set_extra_child(Some(&entry));
                    popup.set_response_enabled("confirm", false);
            
                    entry.connect_changed(clone!(@weak popup => move |entry| {
                        let is_empty = entry.text().is_empty();
            
                        popup.set_response_enabled("confirm", !is_empty);
            
                        if is_empty {
                            entry.add_css_class("error");
                        }
                        else {
                            entry.remove_css_class("error");
                        }
                    }));
                    entry.add_css_class("error");
                    
                    let pin_code = Rc::new(RefCell::new(String::new()));
                    popup.clone().choose(gtk::gio::Cancellable::NONE, move |response| {
                        match response.to_string() {
                            s if s.contains("confirm") => {
                                *pin_code.borrow_mut() = entry.text().to_string();
                            }
                            _ => {
                                *pin_code.borrow_mut() = String::new();
                            }
                        }
                        unsafe {
                            DISPLAYING_DIALOG = false;
                            PIN_CODE = pin_code.borrow().clone();
                        }
                    });
                },
                Message::DisplayPinCode(request) => {
                    let pin_code = &request.pincode;
                    let device: String;
                    unsafe {
                        device = DEVICES_LUT.clone().unwrap().get(&request.device).unwrap_or(&"Unknown Device".to_string()).to_string();
                        DISPLAYING_DIALOG = true;
                    }

                    let body = "Please enter this pin code on ".to_string() + device.as_str();
                    let popup = adw::MessageDialog::new(Some(&clone), None, Some(body.as_str()));
                    
                    let label = gtk::Label::new(Some(pin_code.as_str()));
            
                    popup.set_extra_child(Some(&label));
                    popup.add_response("okay", "Okay");
                    popup.set_close_response("okay");
            
                    popup.clone().choose(gtk::gio::Cancellable::NONE,  move |_| {
                        unsafe {
                            DISPLAYING_DIALOG = false;
                        }                            
                    });
                },
                Message::RequestPassKey(request) => {
                    let device: String;
                    let adapter: String;
                    unsafe {
                        device = DEVICES_LUT.clone().unwrap().get(&request.device).unwrap_or(&"Unknown Device".to_string()).to_string();
                        adapter = ADAPTERS_LUT.clone().unwrap().get(&request.adapter).unwrap_or(&"Unknown Adapter".to_string()).to_string();
                        DISPLAYING_DIALOG = true;
                    }
            
                    let body = device + "has requested pairing on " + adapter.as_str() + ", please enter the correct pass key.";
                    let popup = adw::MessageDialog::new(Some(&clone), Some("Pass Key Requested"), Some(body.as_str()));
            
                    popup.set_close_response("cancel");
                    // popup.set_modal(true);
                    popup.set_destroy_with_parent(true);
            
                    popup.add_response("cancle", "Cancel");
                    popup.add_response("confirm", "Confirm");
                    popup.set_response_appearance("confirm", adw::ResponseAppearance::Suggested);
                    popup.set_default_response(Some("confirm"));
            
                    let entry = gtk::Entry::new();
                    entry.set_placeholder_text(Some("0-999999"));
                    entry.set_input_purpose(gtk::InputPurpose::Digits);
                    entry.set_max_length(6);
            
                    popup.set_extra_child(Some(&entry));
                    popup.set_response_enabled("confirm", false);
            
                    entry.connect_changed(clone!(@weak popup => move |entry| {
                        let is_empty = entry.text().is_empty();
            
                        popup.set_response_enabled("confirm", !is_empty);
            
                        if is_empty {
                            entry.add_css_class("error");
                        }
                        else {
                            entry.remove_css_class("error");
                        }
                    }));
                    entry.add_css_class("error");
            
                    let pass_key = Rc::new(RefCell::new(String::new()));
                    popup.clone().choose(gtk::gio::Cancellable::NONE, move |response| {
                        match response.to_string() {
                            s if s.contains("confirm") => {
                                *pass_key.borrow_mut() = entry.text().to_string();
                            }
                            _ => {
                                *pass_key.borrow_mut() = String::new();
                            }
                        }
                        unsafe {
                            DISPLAYING_DIALOG = false;
                            PASS_KEY = pass_key.borrow().parse::<u32>().unwrap_or(0);
                        }
                    });
                },
                Message::DisplayPassKey(request) => {
                    let pin_code = &request.passkey;
                    let device: String;
                    unsafe {
                        device = DEVICES_LUT.clone().unwrap().get(&request.device).unwrap_or(&"Unknown Device".to_string()).to_string();
                        DISPLAYING_DIALOG = true;
                    }
            
                    if clone.imp().display_pass_key_dialog.borrow().clone().is_some() {
                        let dialog = clone.imp().display_pass_key_dialog.borrow().clone().unwrap();
                        let label = dialog.extra_child().unwrap().downcast::<gtk::Label>().unwrap();
            
                        label.set_text(pin_code.to_string().as_str());
                    }
                    else {
                        let body = "Please enter this pin code on ".to_string() + device.as_str();
                        let popup = adw::MessageDialog::new(Some(&clone), None, Some(body.as_str()));
                        
                        let label = gtk::Label::new(Some(pin_code.to_string().as_str()));
                
                        popup.set_extra_child(Some(&label));
                        popup.add_response("okay", "Okay");
                        popup.set_close_response("okay");
                        
                        popup.clone().choose(gtk::gio::Cancellable::NONE,  move |_| {
                            unsafe {
                                DISPLAYING_DIALOG = false;
                            }
                        });
                        *clone.imp().display_pass_key_dialog.borrow_mut() = Some(popup.clone());
                    }
                },
                Message::RequestConfirmation(request) => {
                    let device: String;
                    let adapter: String;
                    let passkey = &request.passkey.to_string();
                    unsafe {
                        DISPLAYING_DIALOG = true;
                        device = DEVICES_LUT.clone().unwrap().get(&request.device).unwrap_or(&"Unknown Device".to_string()).to_string();
						let mut holder = String::new();
						for key in ADAPTERS_LUT.clone().unwrap().keys() {
							if let Some(pair) = ADAPTERS_LUT.clone().unwrap().get_key_value(key) {
								if pair.1 == &request.adapter {
									holder = pair.0.to_string();
								}
							}
						}
						if holder.is_empty() {
							adapter = "Unknown Adapter".to_string();
						}
						else {
							adapter = holder;
						}
                    }
            
                    let body = "Is this the right code for ".to_string() + device.as_str() + " on " + adapter.as_str();
                    let popup = adw::MessageDialog::new(Some(&clone), Some("Pairing Request"), None);
                    popup.set_body_use_markup(true);
                    popup.set_body(body.as_str());
            
                    popup.set_close_response("cancel");
                    // popup.set_modal(true);
                    popup.set_destroy_with_parent(true);
            
                    popup.add_response("cancle", "Cancel");
                    popup.add_response("allow", "Allow");
                    popup.set_response_appearance("allow", adw::ResponseAppearance::Suggested);
                    popup.set_default_response(Some("allow"));
            
                    let string = "<span font_weight='bold' font_size='32pt'>".to_string() + passkey + "</span>";
                    let label = gtk::Label::new(None);
                    label.set_use_markup(true);
                    label.set_label(string.as_str());
            
                    popup.set_extra_child(Some(&label));
       
                    let pass_key = Rc::new(RefCell::new(false));
                    popup.clone().choose(gtk::gio::Cancellable::NONE, move |response| {
                        match response.to_string() {
                            s if s.contains("allow") => {
                                *pass_key.borrow_mut() = true;
                            }
                            _ => {
                                *pass_key.borrow_mut() = false;
                            }
                        }
                        unsafe {
                            DISPLAYING_DIALOG = false;
                            CONFIRMATION_AUTHORIZATION = *pass_key.borrow();
                        }
                    });            
                },
                Message::RequestAuthorization(request) => {
                    let device: String;
                    let adapter: String;
                    unsafe {
                        device = DEVICES_LUT.clone().unwrap().get(&request.device).unwrap_or(&"Unknown Device".to_string()).to_string();
                        adapter = ADAPTERS_LUT.clone().unwrap().get(&request.adapter).unwrap_or(&"Unknown Adapter".to_string()).to_string();
                        DISPLAYING_DIALOG = true;
                    }
            
                    let body = "Is ".to_string() + device.as_str() + " on " + adapter.as_str() + " allowed to pair?";
                    let popup = adw::MessageDialog::new(Some(&clone), Some("Pairing Request"), None);
                    popup.set_body_use_markup(true);
                    popup.set_body(body.as_str());
            
                    popup.set_close_response("cancel");
                    // popup.set_modal(true);
                    popup.set_destroy_with_parent(true);
            
                    popup.add_response("cancle", "Cancel");
                    popup.add_response("allow", "Allow");
                    popup.set_response_appearance("allow", adw::ResponseAppearance::Suggested);
                    popup.set_default_response(Some("allow"));
                            
                    let pass_key = Rc::new(RefCell::new(false));
                    popup.clone().choose(gtk::gio::Cancellable::NONE, move |response| {
                        match response.to_string() {
                            s if s.contains("allow") => {
                                *pass_key.borrow_mut() = true;
                            }
                            _ => {
                                *pass_key.borrow_mut() = false;
                            }
                        }
                        unsafe {
                            DISPLAYING_DIALOG = false;
                            CONFIRMATION_AUTHORIZATION = *pass_key.borrow();
                        }
                    });            
                },
                Message::AuthorizeService(request) => {
                    let device: String;
                    let adapter: String;
                    unsafe {
                        DISPLAYING_DIALOG = true;
                        device = DEVICES_LUT.clone().unwrap().get(&request.device).unwrap_or(&"Unknown Device".to_string()).to_string();
                        adapter = ADAPTERS_LUT.clone().unwrap().iter()
                        	.find_map(|(key, val)| if val == &request.adapter { Some(key) } else { None })
                       		.unwrap_or(&"Unknown Adapter".to_string()).to_string();
                    }
            
                    let body = "Is ".to_string() + device.as_str() + " on " + adapter.as_str() + " allowed to authorize this service?";
                    let popup = adw::MessageDialog::new(Some(&clone), Some("Service Authorization Request"), None);
                    popup.set_body_use_markup(true);
                    popup.set_body(body.as_str());
            
            		let service_id = match bluer::id::Service::try_from(request.service) {
                        Ok(name) =>{
                        	println!("service name is: {}", name.clone());
                        	format!("{}", name)	
                        },
                        Err(_) => {
                           	println!("service id is: {}", request.service);
                        	format!("{:?}", request.service)	
                        },
                    };
                    let string = "<span font_weight='bold' font_size='24pt'>".to_string() + service_id.as_str() + "</span>";
            
                    let label = gtk::Label::new(None);
                    label.set_use_markup(true);
                    label.set_label(string.as_str());
            
                    popup.set_close_response("cancel");
                    // popup.set_modal(true);
                    popup.set_destroy_with_parent(true);
            
                    popup.add_response("cancle", "Cancel");
                    popup.add_response("allow", "Allow");
                    popup.set_response_appearance("allow", adw::ResponseAppearance::Suggested);
                    popup.set_default_response(Some("allow"));
                            
                    let pass_key = Rc::new(RefCell::new(false));
                    popup.clone().choose(gtk::gio::Cancellable::NONE, move |response| {
                        match response.to_string() {
                            s if s.contains("allow") => {
                                *pass_key.borrow_mut() = true;
                            }
                            _ => {
                                *pass_key.borrow_mut() = false;
                            }
                        }
                        unsafe {
                            DISPLAYING_DIALOG = false;
                            CONFIRMATION_AUTHORIZATION = *pass_key.borrow();
                        }
                    });
                },
                Message::GoToBluetoothSettings(doso) => {
                    if doso {
                        let bluetooth_settings_row = clone.imp().bluetooth_settings_row.get();
                        bluetooth_settings_row.emit_activate();
                    }
                    else {
                        let listbox = clone.imp().main_listbox.get();
                        
                        if let Some(row) = listbox.row_at_index(0) {
                            listbox.select_row(Some(&row));
                        } 
                    }
                },
                Message::InvalidateSort() => {
                	let main_listbox = clone.imp().main_listbox.get();
                	main_listbox.invalidate_sort();	
                },
            }
        
            glib::ControlFlow::Continue
        });        

        let refresh_button = self.imp().refresh_button.get();
        let sender0 = sender.clone();
        refresh_button.connect_clicked(move |button| {
                button.set_sensitive(false);
                let sender_clone = sender0.clone();
                
                std::thread::spawn(move || {
                   std::thread::sleep(std::time::Duration::from_secs(2));
                   sender_clone.send(Message::SetRefreshSensitive(true)).expect("cannot send message");
                });
                
                let can_loop = unsafe {
                    !CURRENTLY_LOOPING
                };
                
                if can_loop {
                    unsafe {
                        if CURRENTLY_LOOPING {
                            sender0.send(Message::PopupError("Started searching for devices".to_string(), adw::ToastPriority::High, "success".to_string())).expect("cannot send message");
                        }
                    }
                    let sender = sender0.clone();
                    let adapter_name = unsafe {
                        CURRENT_ADAPTER.clone()
                    };

                    std::thread::spawn(move || {
                        if let Err(err) = device::get_devices_continuous(sender.clone(), adapter_name) {
                            let string = match err.message {
                                s if s.to_lowercase().contains("resource not ready") => {
                                    "Adapter is not powered".to_string()
                                },
                                s => {
                                    s
                                }
                            };
                
                            sender.send(Message::PopupError(string, adw::ToastPriority::High, "error".to_string())).expect("cannot send message");
                            sender.send(Message::UpdateListBoxImage()).expect("cannot send message");
                        }
                    });
                }
                else {
                    sender0.send(Message::PopupError("Already searching for devices".to_string(), adw::ToastPriority::Normal, "".to_string())).expect("can't send message");
                }
                // println!("trying to available devices");
        });
        refresh_button.emit_clicked();
        
        let main_listbox = self.imp().main_listbox.get();
        main_listbox.set_sort_func(|row_one, row_two| {
        	let actionrow_one = row_one.clone().downcast::<DeviceActionRow>().unwrap();
        	let actionrow_two = row_two.clone().downcast::<DeviceActionRow>().unwrap();
        	
            let binding_one = actionrow_one.title();
            let binding_two = actionrow_two.title();
            
            let rssi_one = actionrow_one.rssi();
            let rssi_two = actionrow_two.rssi();
            // println!("binding one {} binding two {}", binding_one, binding_two);
            
            let mut one = binding_one.as_str();
            let mut two = binding_two.as_str();
            
        	let one_str = one.to_lowercase();
            let two_str = two.to_lowercase();
            
            one = one_str.as_str();
            two = two_str.as_str();
            
            let name_result = one.cmp(two);
            let rssi_result = rssi_one.cmp(&rssi_two);
            //println!("rssi result {:?}", rssi_result);
            
            let final_result = if rssi_result == std::cmp::Ordering::Equal {
                name_result
            }
            else {
                rssi_result
            };
            // println!("rssi one {} rssi two {}", rssi_one, rssi_two);
            // println!("rssi result {:?}", final_result);
            
            final_result.into()
        });
        main_listbox.invalidate_sort();
        
        let connected_switch_row = self.imp().connected_switch_row.get();
        let sender1 = sender.clone();
        connected_switch_row.set_activatable(true);
        connected_switch_row.connect_activated(move |row| {
            if row.spinning() {
                row.set_spinning(false);
            }

            let sender_clone = sender1.clone();
            let address = unsafe { 
            	CURRENT_ADDRESS 
            };
            let adapter_name = unsafe {
                CURRENT_ADAPTER.clone()
            };
            
            row.set_active(!row.active());
            std::thread::spawn(move || {
                if let Err(err) = device::set_device_active(address, sender_clone.clone(), adapter_name) {
                    let string = err.clone().message;
                    println!("error while connecting {:?}\n", err);

                    sender_clone.send(Message::PopupError(string, adw::ToastPriority::High, "error".to_string())).expect("cannot send message");
                    sender_clone.send(Message::SwitchActive(false)).expect("cannot send message");
                    sender_clone.send(Message::SwitchActiveSpinner(false)).expect("cannot send message");
                }
            });
        });
        let sender10 = sender.clone();
        connected_switch_row.child().unwrap().downcast::<gtk::Box>().unwrap().last_child().unwrap().downcast::<gtk::Box>().unwrap()
            .first_child().unwrap().downcast::<gtk::Box>().unwrap().last_child().unwrap().downcast::<gtk::Switch>().unwrap()
            .connect_active_notify(move |_| {
            	println!("swithced");
                if connected_switch_row.spinning() {
                    connected_switch_row.set_spinning(false);
                }
    
                let sender_clone = sender10.clone();
                let address = unsafe { 
                	CURRENT_ADDRESS 
                };
                let adapter_name = unsafe {
                    CURRENT_ADAPTER.clone()
                };
                
                connected_switch_row.set_active(!connected_switch_row.active());
                std::thread::spawn(move || {
                    if let Err(err) = device::set_device_active(address, sender_clone.clone(), adapter_name) {
                        let string = err.clone().message;
                        println!("error while connecting {:?}\n", err);
    
                        sender_clone.send(Message::PopupError(string, adw::ToastPriority::High, "error".to_string())).expect("cannot send message");
                        sender_clone.send(Message::SwitchActive(false)).expect("cannot send message");
                        sender_clone.send(Message::SwitchActiveSpinner(false)).expect("cannot send message");
                    }
                });
            });
        
        let blocked_row = self.imp().blocked_row.get();
        let sender2 = sender.clone();
        blocked_row.connect_activated(move |row| {
            let sender_clone = sender2.clone();
            let address = unsafe { 
            	CURRENT_ADDRESS 
            };
            let adapter_name = unsafe {
                CURRENT_ADAPTER.clone()
            };
            let current_state = !row.is_active();
            
            std::thread::spawn(move || {
                if let Err(err) = device::set_device_blocked(address, sender_clone.clone(), adapter_name) {
                    let string = err.message;
                    sender_clone.send(Message::PopupError(string, adw::ToastPriority::High, "error".to_string())).expect("cannot send message");
	                sender_clone.send(Message::SwitchBlocked(current_state)).expect("cannot send message");
                }
            });
        });

        let trusted_row = self.imp().trusted_row.get();
        let sender3 = sender.clone();
        trusted_row.connect_activated(move |row| {
            let sender_clone = sender3.clone();
            let address = unsafe { 
            	CURRENT_ADDRESS 
            };
            let adapter_name = unsafe {
                CURRENT_ADAPTER.clone()
            };
            let trusted = !row.is_active();

            std::thread::spawn(move || {
                if let Err(err) = device::set_device_trusted(address, sender_clone.clone(), adapter_name) {
                    let string = err.message;
                    sender_clone.send(Message::PopupError(string, adw::ToastPriority::High, "error".to_string())).expect("cannot send message");
                    sender_clone.send(Message::SwitchTrusted(trusted)).expect("cannot send message");
                };
            });
        });

        let device_name_entry = self.imp().device_name_entry.get();
        let sender4 = sender.clone();
        device_name_entry.connect_apply(move |entry| {
            let sender_clone = sender4.clone();
            let name = entry.text().to_string();
            let address = unsafe { 
           		CURRENT_ADDRESS 
            };
            let adapter_name = unsafe {
                CURRENT_ADAPTER.clone()
            };

            std::thread::spawn(move || {
                if let Err(err) = device::set_device_name(address, name, sender_clone.clone(), adapter_name) {
                    let string = err.message;
                    sender_clone.send(Message::PopupError(string, adw::ToastPriority::High, "error".to_string())).expect("cannot send message");
                }
            });
        });

        let remove_device_button = self.imp().remove_device_button.get();
        let sender4 = sender.clone();
        remove_device_button.connect_clicked(move |_| {
            let sender_clone = sender4.clone();
            let address = unsafe { 
            	CURRENT_ADDRESS 
            };
            let adapter_name = unsafe {
                CURRENT_ADAPTER.clone()
            };
            
            std::thread::spawn(move || {
                if let Err(err) = device::remove_device(address, sender_clone.clone(), adapter_name) {
                    let string = err.message;
                    sender_clone.send(Message::PopupError(string, adw::ToastPriority::High, "error".to_string())).expect("cannot send message");
                }
            });
        });

        let powered_switch_row = self.imp().powered_switch_row.get();
        let sender5 = sender.clone();
        powered_switch_row.connect_activated(move |_| {
            let sender_clone = sender5.clone();
            let adapter_name = unsafe {
                CURRENT_ADAPTER.clone()
            };

            std::thread::spawn(move || {
                if let Err(err) = bluetooth_settings::set_adapter_powered(adapter_name, sender_clone.clone()) {
                    let string = err.message;
                    sender_clone.send(Message::PopupError(string, adw::ToastPriority::High, "error".to_string())).expect("cannot send message");
                    sender_clone.send(Message::SwitchAdapterPowered(false)).expect("cannot send message");
                }
            });
        });

        let discoverable_switch_row = self.imp().discoverable_switch_row.get();
       	let sender6 = sender.clone();
       	discoverable_switch_row.connect_activated(move |_| {
            let sender_clone = sender6.clone();
            let adapter_name = unsafe {
                CURRENT_ADAPTER.clone()
            };

            std::thread::spawn(move || {
                if let Err(err) = bluetooth_settings::set_adapter_discoverable(adapter_name, sender_clone.clone()) {
                    let string = "Adapter ".to_string() + &err.message;
                    sender_clone.send(Message::PopupError(string, adw::ToastPriority::High, "error".to_string())).expect("cannot send message");
                    sender_clone.send(Message::SwitchAdapterDiscoverable(false)).expect("cannot send message");
                }
            });
        });

        let adapter_name_entry = self.imp().adapter_name_entry.get();
        let sender7 = sender.clone();
        adapter_name_entry.connect_apply(move |entry| {
            let new_name = entry.text().to_string();
            let sender_clone = sender7.clone();
            let adapter_name = unsafe {
                CURRENT_ADAPTER.clone()
            };

            std::thread::spawn(move || {
                if let Err(err) = bluetooth_settings::set_adapter_name(new_name, adapter_name, sender_clone.clone()) {
                    let string = "Adapter ".to_string() + &err.message;
                    sender_clone.send(Message::PopupError(string, adw::ToastPriority::High, "error".to_string())).expect("cannot send message");
                }
            });
        });

        let timeout_adjustment = self.imp().timeout_time_adjustment.get();
        let sender8 = sender.clone();
        let id = timeout_adjustment.connect_value_changed(move |adjustment| {
            let value = adjustment.value();
            let sender_clone = sender8.clone();
            let adapter_name = unsafe {
                CURRENT_ADAPTER.clone()
            };

            std::thread::spawn(move || {
                if let Err(err) = bluetooth_settings::set_timeout_duration(value as u32, adapter_name, sender_clone.clone()) {
                    let string = err.message;
                    sender_clone.send(Message::PopupError(string, adw::ToastPriority::High, "error".to_string())).expect("cannot send message");
                    sender_clone.send(Message::SwitchAdapterTimeout(0)).expect("cannot send message");
                }  
            });
        });
        self.imp().timeout_signal_id.set(id).expect("cannot set timeout signal id");

        let bluetooth_settings_row = self.imp().bluetooth_settings_row.get();
        let sender9 = sender.clone();
        let self_clone3 = self.clone();
        bluetooth_settings_row.connect_activated(move |_| {
            let sender_clone = sender9.clone();
            std::thread::spawn(move || {
                let adapter_names = bluetooth_settings::populate_adapter_expander();
                let sender = unsafe {
                    CURRENT_SENDER.clone().unwrap()
                };
                let adapter_name = unsafe {
                    CURRENT_ADAPTER.clone()
                };

                if let Ok(names) = adapter_names {
                    if let Err(err) = bluetooth_settings::get_adapter_properties(names, sender, adapter_name) {
	                    let string = "Adapter ".to_string() + &err.message;
	                    sender_clone.send(Message::PopupError(string, adw::ToastPriority::Normal, "error".to_string())).expect("cannot send message");    
                    }
                }
            });
            
            let main_listbox = self_clone3.imp().main_listbox.get();
            main_listbox.unselect_all();
            
            let main_stack = self_clone3.imp().main_stack.get();
            let pages = main_stack.pages();
            pages.select_item(1, true);

            let split_view = self_clone3.imp().split_view.get();
            if split_view.is_collapsed() {
                split_view.set_show_sidebar(false);
            }
        });
        bluetooth_settings_row.emit_activate();

        let split_view = self.imp().split_view.get();
        let self_clone4 = self.clone();
        split_view.connect_show_sidebar_notify(move |view| {
            let show_sidebar_button = self_clone4.imp().show_sidebar_button.get();
			let active = view.shows_sidebar();

            let text = match active {
                true => {
                    "Hide Sidebar"
                },
                false => {
                    "Show Sidebar"
                }
            };
            show_sidebar_button.set_tooltip_text(Some(text));
            show_sidebar_button.set_active(active);
        });
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

    #[tokio::main]
    async fn pre_setup(&self, sender: Sender<Message>) -> bluer::Result<()> {
        let settings = self.imp().settings.get().unwrap();

        unsafe { 
            CURRENT_SENDER = Some(sender.clone());
            DEVICES_LUT = Some(HashMap::new());
            RSSI_LUT = Some(HashMap::new());
            let name = settings.string("current-adapter-name").to_string();
            let session = bluer::Session::new().await?;

            if name.is_empty() {
                let adapter = session.default_adapter().await?;
                CURRENT_ADAPTER = adapter.name().to_string();
                ORIGINAL_ADAPTER = CURRENT_ADAPTER.clone().to_string();
                settings.set_string("current-adapter-name", CURRENT_ADAPTER.as_str()).expect("cannot set default adapter at start");
                settings.set_string("original-adapter-name", CURRENT_ADAPTER.as_str()).expect("cannot set original adapter at start");
            }
            else {
                CURRENT_ADAPTER = name.clone();
            }
            
            let mut lut = HashMap::new();
            
            let adapter = session.adapter(CURRENT_ADAPTER.clone().as_str())?;
            let alias = adapter.alias().await?;
            println!("startup alias is: {}\n", alias);
            self.imp().timeout_time_adjustment.get().set_value(adapter.discoverable_timeout().await?.into());

            lut.insert(alias.to_string(), CURRENT_ADAPTER.to_string());
            ADAPTERS_LUT = Some(lut);
        }
        
        Ok(())
    }    
}

#[tokio::main]
async fn add_child_row(device: bluer::Device) -> bluer::Result<DeviceActionRow> {
    let child_row = DeviceActionRow::new();
    // println!("added device name is {:?}", device.name().await?);

    let name = device.alias().await?;
    let address = device.address();
    let rssi = match device.rssi().await? {
        None => {
            0
        },
        Some(n) => {
            n as i32
        }
    };
    
    child_row.set_bluer_address(address);
    child_row.set_title(name.clone().as_str());
    child_row.set_activatable(true);
    child_row.set_adapter_name(unsafe {CURRENT_ADAPTER.clone()});

    child_row.set_rssi(rssi);   
    
    unsafe {
        let mut devices_lut = DEVICES_LUT.clone().unwrap();
        devices_lut.insert(address, name.clone());
        //println!("lut (add) is: {:?}", devices_lut);
        DEVICES_LUT = Some(devices_lut);
        //println!("big lut (add) is: {:?}", DEVICES_LUT.clone());
    } 
	let sender = unsafe { 
        CURRENT_SENDER.clone().unwrap() 
    };
    sender.send(Message::InvalidateSort()).expect("cannot send message");
    sender.send(Message::SwitchRssi(name.clone(), rssi)).expect("cannot send message");

    child_row.connect_activated(move |row| {        
        unsafe {
            CURRENT_INDEX = row.index();
            CURRENT_ADDRESS = row.get_bluer_address();
        }
        
        let address = row.get_bluer_address();
        let adapter_name = row.adapter_name();
		let sender_clone = sender.clone();

        println!("row address {} with adapter {}", address.clone(), adapter_name.clone());

        std::thread::spawn(move || {
            let sender_clone_clone = sender_clone.clone(); // lmao

            if let Err(err) = device::get_device_properties(address, sender_clone_clone.clone(), adapter_name) {
	            let string = err.message;

	            sender_clone_clone.send(Message::GoToBluetoothSettings(true)).expect("cannot send message");
	            sender_clone_clone.send(Message::PopupError(string, adw::ToastPriority::High, "error".to_string())).expect("cannot send message");
            }
        });
    });

    Ok(child_row)
}



// TODO
// - add a match rule for weird ass device names (address for name) and add address as subtext
// - add a match rule for device rssi change and handle icon change and invalidate sort
// - add a spinner (preffered hig) or loading bar (looks better?) for long action (connecting to device) // current
// - refresh button should stop discovering and restart it
// - gray out actions that take a while so user doesn't fuck up stuff
// - set all popups to modal
// - use fxhashmap for even faster lookups
// - add option to auto trust device on pair (include warning about how dangerous it is)
// - fix get devices continous being wrapped in another useless functions
// - background running, with a status taskbar thingy wtv its name is
// - add popup error priority. (perhaps with colors)
