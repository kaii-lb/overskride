use glib::{Object, Properties};
use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::prelude::ObjectExt;
use std::cell::RefCell;

use crate::obex_utils::ObexTransfer1;

mod imp {
    use adw::prelude::ButtonExt;
    use dbus::blocking::Connection;

    use super::*;    

    #[derive(Properties, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/kaii_lb/Overskride/gtk/receiving-row.ui")]
    #[properties(wrapper_type = super::ReceivingRow)]
    pub struct ReceivingRow {
        #[template_child]
        pub title_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub progress_bar: TemplateChild<gtk::ProgressBar>,
        #[template_child]
        pub extra_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub cancel_button: TemplateChild<gtk::Button>,

        #[property(get, set)]
        pub transfer: RefCell<String>,
        #[property(get = Self::get_filename_from_label, set = Self::set_filename_from_label)]
        pub filename: RefCell<String>,
        #[property(get, set = Self::set_progress_bar_fraction)]
        pub percentage: RefCell<u32>,

        pub extra: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ReceivingRow {
        const NAME: &'static str = "ReceivingRow";
        type Type = super::ReceivingRow;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    
    #[glib::derived_properties]
    impl ObjectImpl for ReceivingRow {
        fn constructed(&self) {
            self.parent_constructed();
            let transfer = self.transfer.borrow().clone();
            self.cancel_button.get().connect_clicked(move |_| {
                let conn = Connection::new_session().expect("cannot create connection.");
                let proxy = conn.with_proxy("org.bluez.obex", transfer.clone(), std::time::Duration::from_secs(1));
                proxy.cancel().expect("cannot cancel transfer");
            });
        }
    }

    impl WidgetImpl for ReceivingRow {}
    impl ListBoxRowImpl for ReceivingRow {}

    impl ReceivingRow {
        fn get_filename_from_label(&self) -> String {
            self.filename.borrow().clone()
        }
    
        fn set_filename_from_label(&self, filename: String) {
            *self.filename.borrow_mut() = filename;
        }

        fn set_progress_bar_fraction(&self, fraction: f32) {
            self.progress_bar.get().set_fraction(fraction as f64);
        }
    }
}

glib::wrapper! {
    pub struct ReceivingRow(ObjectSubclass<imp::ReceivingRow>)
        @extends gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl ReceivingRow {
    /// creates a new `ReceivingRow`, no values in, no values out.
    pub fn new(transfer: String, filename: String) -> Self {
        Object::builder()
            .property("transfer", transfer)
            .property("filename", filename)
            .build()
    }

    // fn get_extra(&self) -> String {
    //     self.imp().filename.borrow().clone()
    // }

    #[allow(dead_code)]
    pub fn set_extra(&self, percent: u32, current_mb: f32, filesize_mb: u32) {
        let percentage = percent.to_string() + "% | ";
        let size = current_mb.to_string() + "/" + &filesize_mb.to_string();

        let extra = percentage + size.as_str();
        *self.imp().extra.borrow_mut() = extra;
    }
}
