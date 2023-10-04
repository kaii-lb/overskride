use glib::{Object, Properties};
use gtk::glib;
use adw::subclass::prelude::{ActionRowImpl, PreferencesRowImpl};
use gtk::subclass::prelude::*;
use gtk::prelude::ObjectExt;
use std::cell::RefCell;

mod imp {
    use super::*;    

    #[derive(Properties, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/kaii_lb/Overskride/gtk/device-action-row.ui")]
    #[properties(wrapper_type = super::DeviceActionRow)]
    pub struct DeviceActionRow {
        #[template_child]
        pub rssi_icon: TemplateChild<gtk::Image>,

        #[property(get, set)]
        pub rssi: RefCell<i32>,
        #[property(get, set)]
        pub adapter_name: RefCell<String>,

        pub address: RefCell<bluer::Address>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DeviceActionRow {
        const NAME: &'static str = "DeviceActionRow";
        type Type = super::DeviceActionRow;
        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    
    #[glib::derived_properties]
    impl ObjectImpl for DeviceActionRow {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl ActionRowImpl for DeviceActionRow {}
    impl WidgetImpl for DeviceActionRow {}
    impl ListBoxRowImpl for DeviceActionRow {}
    impl PreferencesRowImpl for DeviceActionRow {}
    
    impl DeviceActionRow {}
}

glib::wrapper! {
    pub struct DeviceActionRow(ObjectSubclass<imp::DeviceActionRow>)
        @extends adw::ActionRow, gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl DeviceActionRow {
    /// creates a new `DeviceActionRow`, no values in, no values out.
    pub fn new() -> Self {
        Object::builder()
            .build()
    }

    pub fn get_bluer_address(&self) -> bluer::Address {
        self.imp().address.borrow().clone()   
    }

    pub fn set_bluer_address(&self, address: bluer::Address) {
        *self.imp().address.borrow_mut() = address;
    }

    pub fn update_rssi_icon(&self) {
        let icon_name = match self.imp().rssi.borrow().clone() {
            0 => {
                "rssi-none-symbolic"
            },
            n if -n <= 60 => {
                "rssi-high-symbolic"
            } 
            n if -n <= 70 => {
                "rssi-medium-symbolic"
            }
            n if -n <= 80 => {
                "rssi-low-symbolic"
            }
            n if -n <= 90 => {
                "rssi-dead-symbolic"
            }
            n if -n <= 100 => {
                "rssi-none-symbolic"
            }
            val => {
                println!("rssi unknown value: {}", val);
                "rssi-not-found-symbolic"
            }
        };

        self.imp().rssi_icon.set_icon_name(Some(icon_name));
    }
}
