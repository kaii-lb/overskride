use glib::Object;
use gtk::glib;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;    

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/kaii_lb/Overskride/gtk/receiving-popover.ui")]
    pub struct ReceivingPopover {
        #[template_child]
        pub listbox: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ReceivingPopover {
        const NAME: &'static str = "ReceivingPopover";
        type Type = super::ReceivingPopover;
        type ParentType = gtk::Popover;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    
    impl ObjectImpl for ReceivingPopover {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for ReceivingPopover {}
    impl PopoverImpl for ReceivingPopover {}
}

glib::wrapper! {
    pub struct ReceivingPopover(ObjectSubclass<imp::ReceivingPopover>)
        @extends gtk::Popover, gtk::Widget,
        @implements gtk::Accessible, gtk::Native, gtk::Buildable, gtk::ConstraintTarget, gtk::ShortcutManager;
}

impl ReceivingPopover {
    /// creates a new `ReceivingPopover`, no values in, no values out.
    pub fn new() -> Self {
        Object::builder()
            .build()
    }

    pub fn get_listbox(&self) -> gtk::ListBox {
        self.imp().listbox.get()
    }
}

impl Default for ReceivingPopover {
	fn default() -> Self {
		Self::new()
	}
}
