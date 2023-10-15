use adw::prelude::ButtonExt;
use glib::{Object, Properties};
use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::prelude::ObjectExt;
use std::cell::RefCell;
use gtk::prelude::WidgetExt;

use crate::obex::CANCEL;

mod imp {
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
        pub percentage: RefCell<f32>,
        #[property(get, set)]
        pub filesize: RefCell<f32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ReceivingRow {
        const NAME: &'static str = "ReceivingRow";
        type Type = super::ReceivingRow;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    
    #[glib::derived_properties]
    impl ObjectImpl for ReceivingRow {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for ReceivingRow {}
    impl ListBoxRowImpl for ReceivingRow {}

    #[gtk::template_callbacks]
    impl ReceivingRow {
        fn get_filename_from_label(&self) -> String {
            self.title_label.get().label().to_string()
        }
    
        fn set_filename_from_label(&self, filename: String) {
            self.title_label.get().set_label(&("Receiving: “".to_string() + &filename + "”"));
        }

        fn set_progress_bar_fraction(&self, fraction: f32) {
            let holder = (fraction / 100.0) as f64;
            // println!("divved {}", holder);
            self.progress_bar.get().set_fraction(holder);
        }

        #[template_callback]
        fn cancel_transfer(&self, button: &gtk::Button) {
            unsafe {
                CANCEL = true;
            }
            button.set_sensitive(false);
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
    pub fn new(transfer: String, filename: String, filesize: f32) -> Self {
        Object::builder()
            .property("transfer", transfer)
            .property("filename", filename)
            .property("filesize", filesize)
            .build()
    }

    pub fn get_extra(&self) -> String {
        self.imp().extra_label.get().label().to_string()
    }

    pub fn set_extra(&self, percent: f32, current_mb: f32, filesize_mb: f32) {
        let percentage = percent.to_string() + "% | ";
        let size = current_mb.to_string() + "/" + &filesize_mb.to_string();

        let extra = "<small>".to_string() + &percentage + size.as_str() + "</small>";
        self.set_filesize(filesize_mb);
        self.set_percentage(percent);
        self.imp().extra_label.get().set_label(&extra);
    }

    pub fn set_error(&self, error: String) {
        let final_string = "<small>".to_string() + &error + "</small>";
        self.imp().extra_label.get().set_label(&final_string);
    }

    pub fn set_active_icon(&self, icon_name: String, filesize: f32) -> bool {
        let cancel_button = self.imp().cancel_button.get();
        let self_destruct: bool;

        let icon = match icon_name.as_str() {
            "complete" => {
                cancel_button.set_sensitive(false);
                self_destruct = true;
                
                let done = "File Transfer Completed (".to_string() + &filesize.to_string() + " MB)";
                self.set_error(done);
                
                "check-plain-symbolic"
            },
            "error" => {
                cancel_button.set_sensitive(false);
                self_destruct = true;
                
                let done = "File Transfer Canceled (".to_string() + &filesize.to_string() + " MB)";
                self.set_error(done);

                "skull-symbolic"
            },
            e => {
                if !e.is_empty() {
                    println!("special icon case: {}", e);
                }
                self_destruct = false;
                "cross-large-symbolic"
            },
        };

        cancel_button.set_icon_name(icon);
        self_destruct
    }
}
