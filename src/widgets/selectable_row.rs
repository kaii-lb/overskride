use glib::{Object, Properties};
use gtk::glib;
use adw::subclass::prelude::{ActionRowImpl, PreferencesRowImpl};
use gtk::subclass::prelude::*;
use gtk::prelude::ObjectExt;
use std::cell::RefCell;
use gtk::prelude::WidgetExt;

mod imp {
    use super::*;    

    /// a custom type that has a checkmark next to the row, showing it if the row is selected, hiding it if not
    #[derive(Properties, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/kaii_lb/Overskride/gtk/selectable-row.ui")]
    #[properties(wrapper_type = super::SelectableRow)]
    pub struct SelectableRow {
        #[template_child]
        pub check_icon: TemplateChild<gtk::Image>,

        #[property(get, set)]
        pub profile: RefCell<String>,
        #[property(get, set = Self::private_set_row_selected)]
        pub selected: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SelectableRow {
        const NAME: &'static str = "SelectableRow";
        type Type = super::SelectableRow;
        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    
    #[glib::derived_properties]
    impl ObjectImpl for SelectableRow {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl ActionRowImpl for SelectableRow {}
    impl WidgetImpl for SelectableRow {}
    impl ListBoxRowImpl for SelectableRow {}
    impl PreferencesRowImpl for SelectableRow {}
    
    impl SelectableRow {
        /// adds a checkmark next to the row if selected, removes it if not
        pub fn private_set_row_selected(&self, active: bool) {
            *self.selected.borrow_mut() = active;
            let check_icon = self.check_icon.get();
    
            if *self.selected.borrow() {
                check_icon.set_visible(true);
            }
            else {
                check_icon.set_visible(false);
            }
        }
        
        pub fn private_get_row_profile(&self) -> String {
            self.profile.borrow().clone()
        }

        pub fn private_set_row_profile(&self, profile: String) {
            *self.profile.borrow_mut() = profile;
        }
    }
}

glib::wrapper! {
    pub struct SelectableRow(ObjectSubclass<imp::SelectableRow>)
        @extends adw::ActionRow, gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl SelectableRow {
    /// creates a new `SelectableRow`, no values in, no values out.
    pub fn new() -> Self {
        Object::builder()
            .build()
    }
    
    pub fn set_row_selected(&self, selected: bool) {
        self.imp().private_set_row_selected(selected);
    }
    
    pub fn get_row_profile(&self) -> String {
        self.imp().private_get_row_profile()
    }

    pub fn set_row_profile(&self, profile: String) {
        self.imp().private_set_row_profile(profile);
    }
}

impl Default for SelectableRow {
	fn default() -> Self {
		Self::new()
	}
}


