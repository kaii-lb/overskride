use glib::{Object, Properties};
use gtk::glib;
use adw::subclass::prelude::{ActionRowImpl, PreferencesRowImpl};
use gtk::subclass::prelude::*;
use gtk::prelude::ObjectExt;
use std::cell::RefCell;
use gtk::prelude::WidgetExt;

mod imp {
    use super::*;    

    #[derive(Properties, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/kaii_lb/Overskride/gtk/selectable-row.ui")]
    #[properties(wrapper_type = super::SelectableRow)]
    pub struct SelectableRow {
        #[template_child]
        pub check_icon: TemplateChild<gtk::Image>,

        #[property(get, set)]
        pub profile: RefCell<String>,
        #[property(get, set = Self::set_row_selected)]
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
        pub fn set_row_selected(&self, active: bool) {
            *self.selected.borrow_mut() = active;
            let check_icon = self.check_icon.get();
    
            if *self.selected.borrow() {
                check_icon.show();
            }
            else {
                check_icon.hide();
            }
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
}

impl Default for SelectableRow {
	fn default() -> Self {
		Self::new()
	}
}


