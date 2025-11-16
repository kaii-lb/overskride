use glib::{Object, Properties};
use gtk::glib;
use adw::subclass::prelude::{ActionRowImpl, PreferencesRowImpl};
use gtk::subclass::prelude::*;
use gtk::prelude::ObjectExt;
use std::cell::RefCell;

mod imp {
    use gtk::prelude::WidgetExt;

    use super::*;    

    /// an adw::SwitchRow but with the ability to show a spinning icon next to it
    #[derive(Properties, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/kaii_lb/Overskride/gtk/connected-switch-row.ui")]
    #[properties(wrapper_type = super::ConnectedSwitchRow)]
    pub struct ConnectedSwitchRow {
        #[template_child]
        pub switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub spinner: TemplateChild<gtk::Spinner>,

        #[property(get, set = Self::private_set_row_active)]
        pub active: RefCell<bool>,
        #[property(get = Self::private_is_spinning, set = Self::private_set_spinning)]
        pub spinning: RefCell<bool>,
        #[property(set = Self::private_set_switch_active)]
        pub switch_active: RefCell<bool>,
        #[property(get, set)]
        pub has_obex: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ConnectedSwitchRow {
        const NAME: &'static str = "ConnectedSwitchRow";
        type Type = super::ConnectedSwitchRow;
        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    
    #[glib::derived_properties]
    impl ObjectImpl for ConnectedSwitchRow {
        fn constructed(&self) {
            self.parent_constructed();
            self.switch.get().connect_activate(|meeee| {
                meeee.parent().unwrap().activate();
            });
        }
    }

    impl ActionRowImpl for ConnectedSwitchRow {}
    impl WidgetImpl for ConnectedSwitchRow {}
    impl ListBoxRowImpl for ConnectedSwitchRow {}
    impl PreferencesRowImpl for ConnectedSwitchRow {}
    
    impl ConnectedSwitchRow {
        pub fn private_set_switch_active(&self, active: bool) {
            self.switch.set_active(active);
            *self.active.borrow_mut() = active;
        }
        
        pub fn private_set_row_active(&self, active: bool) {
            let current_active = self.switch.get().is_active();

            if current_active == active {
                return;
            }
            // println!("current active for custom row is: {}", current_active);

            *self.active.borrow_mut() = active;
            self.spinner.set_spinning(true);
        }
        
        pub fn private_set_spinning(&self, spinning: bool) {
            self.spinner.set_spinning(spinning);
        }
        
        pub fn private_is_spinning(&self) -> bool {
            self.spinner.is_spinning()
        }
        
        pub fn private_has_obex(&self) -> bool {
            *self.has_obex.borrow()
        }
        
        pub fn private_set_has_obex(&self, value: bool) {
            *self.has_obex.borrow_mut() = value;
        }
    }
}

glib::wrapper! {
    pub struct ConnectedSwitchRow(ObjectSubclass<imp::ConnectedSwitchRow>)
        @extends adw::ActionRow, gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl ConnectedSwitchRow {
    /// creates a new `ConnectedSwitchRow`, no values in, no values out.
    pub fn new() -> Self {
        Object::builder()
            .build()
    }

    /// sets the `ConnectedSwitchRow`'s state to `active`, make the spinning visible in the process.
    pub fn set_row_active(&self, active: bool) {
        self.imp().private_set_row_active(active);
    }

    /// return the current state of the row's spinner, ie: spinning, or not visible.
    pub fn get_row_spinning(&self) -> bool {
        self.imp().private_is_spinning()
    }

    /// sets the row's spinner to `spinning`.
    pub fn set_row_spinning(&self, spinning: bool) {
        self.imp().private_set_spinning(spinning);
    }

    pub fn set_toggle_switch_active(&self, active: bool) {
        self.imp().private_set_switch_active(active);
    }
    
    pub fn row_has_obex(&self) -> bool {
        self.imp().private_has_obex()
    }

    pub fn set_row_has_obex(&self, value: bool) {
        self.imp().private_set_has_obex(value);
    }
}

impl Default for ConnectedSwitchRow {
	fn default() -> Self {
		Self::new()
	}
}


