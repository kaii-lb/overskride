use glib::Object;
use gtk::glib;
use adw::subclass::prelude::AdwApplicationWindowImpl;
use gtk::subclass::prelude::*;
use adw::prelude::PreferencesRowExt;
use adw::prelude::ExpanderRowExt;
use gtk::prelude::WidgetExt;

mod imp {
    use super::*;    

    /// a custom error message if the startup failed, showing the user a way to fix the error
    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/kaii_lb/Overskride/gtk/more-info-page.ui")]
    pub struct MoreInfoPage {
        #[template_child]
        pub name_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub address_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub manufacturer_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub type_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub services_row: TemplateChild<adw::ExpanderRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MoreInfoPage {
        const NAME: &'static str = "MoreInfoPage";
        type Type = super::MoreInfoPage;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    
    impl ObjectImpl for MoreInfoPage {
        fn constructed(&self) {
            self.parent_constructed();            
        }
    }

    impl WidgetImpl for MoreInfoPage {}
    impl AdwApplicationWindowImpl for MoreInfoPage {}
    impl ApplicationWindowImpl for MoreInfoPage {}
    impl WindowImpl for MoreInfoPage {}

    impl MoreInfoPage {}
}

glib::wrapper! {
    pub struct MoreInfoPage(ObjectSubclass<imp::MoreInfoPage>)
        @extends adw::ApplicationWindow, gtk::Widget, gtk::Window, gtk::ApplicationWindow,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl MoreInfoPage {
    /// creates a new `MoreInfoPage`
    pub fn new() -> Self {
        Object::builder()
            .build()
    }

    pub fn initialize_from_info(&self, name: String, address: String, manufactuer: String, device_type: String, services_list: Vec<String>) {
		self.imp().name_row.get().set_title(&("Name: ".to_string() + &name));
		self.imp().address_row.get().set_title(&("Address: ".to_string() + &address));
		self.imp().manufacturer_row.get().set_title(&("Manufacturer: ".to_string() + &manufactuer));
		self.imp().type_row.get().set_title(&("Type: ".to_string() + &device_type));

		let expander_row = self.imp().services_row.get();
		expander_row.set_title("Available Services");
		if services_list.is_empty() {
			expander_row.set_sensitive(false);
		}
		else {
			expander_row.set_sensitive(true);
			
			for service in services_list {
				let row = adw::ActionRow::new();
				row.set_title(&service);
				row.set_title_selectable(true);
				
				expander_row.add_row(&row);
			}	
		}
    }
}

impl Default for MoreInfoPage {
	fn default() -> Self {
		Self::new()
	}
}
