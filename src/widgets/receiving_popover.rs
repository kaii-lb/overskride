use adw::prelude::WidgetExt;
use glib::Object;
use gtk::glib;
use gtk::prelude::{IsA, Cast};
use gtk::subclass::prelude::*;

use crate::receiving_row::ReceivingRow;

mod imp {
    use super::*;    

    /// holds all the current transfer ongoing allowing the user easy management of them
    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/kaii_lb/Overskride/gtk/receiving-popover.ui")]
    pub struct ReceivingPopover {
        #[template_child]
        pub listbox: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub default_row: TemplateChild<gtk::ListBoxRow>,
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

    /// adds a row, enabling or disabling the "no transfers" label as it sees fit
    pub fn add_row(&self, row: &impl IsA<gtk::Widget>) {
        let listbox = self.imp().listbox.get();
        listbox.append(row);

        println!("added row");

        if listbox.row_at_index(2).is_some() {
            listbox.set_show_separators(true);
        } 
        self.imp().default_row.get().set_visible(false);
    }

    /// remove the row from this popover, using the transfer and filename as guidance
    pub fn remove_row(&self, transfer: String, filename: String) {
        let listbox = self.imp().listbox.get();

        let mut index = 0;
        while let Some(row) = listbox.row_at_index(index) {
            if let Ok(receiving_row) = row.clone().downcast::<ReceivingRow>() {
                if receiving_row.transfer().contains(&transfer) && receiving_row.filename().contains(&filename) {
                    listbox.remove(&row);
                    println!("removed row");
                }
            }

            index += 1;
        }

        if listbox.row_at_index(1).is_none() {
            self.imp().default_row.get().set_visible(true);
            listbox.set_show_separators(false);
        }
        else {
            self.imp().default_row.get().set_visible(false);
            
            if listbox.row_at_index(2).is_some() {
                listbox.set_show_separators(true);
            }
        }
    }

    pub fn get_row_by_transfer(&self, transfer: &String, filename: &String) -> Option<ReceivingRow> {
        let listbox = self.imp().listbox.get();

        let mut index = 0;
        while let Some(row) = listbox.row_at_index(index) {
            if let Ok(receiving_row) = row.clone().downcast::<ReceivingRow>() {
                if receiving_row.transfer().contains(transfer) && receiving_row.filename().contains(filename) {
                    return Some(receiving_row);
                }
            }

            index += 1;
        }
        println!("unknown row {} {}", transfer, filename);
        None
    }
}

impl Default for ReceivingPopover {
	fn default() -> Self {
		Self::new()
	}
}
