use gtk::glib;
use gtk::subclass::prelude::*;
use adw::subclass::prelude::PreferencesRowImpl;
use glib::{Object, Properties};
use gtk::{subclass::{widget::WidgetImpl, prelude::{ListBoxRowImpl, ObjectImpl}}, TemplateChild};
use gtk::prelude::ObjectExt;
use std::cell::RefCell;
use adw::prelude::WidgetExt;

mod imp {
    use super::*;

    #[derive(Properties, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/kaii_lb/Overskride/gtk/battery-indicator.ui")]
    #[properties(wrapper_type = super::BatteryLevelIndicator)]
    pub struct BatteryLevelIndicator {
        #[template_child]
        pub battery_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub level_bar: TemplateChild<gtk::LevelBar>,

        #[property(get, set = Self::set_battery_level_from_i8)]
        pub battery_level: RefCell<i8>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BatteryLevelIndicator {
        const NAME: &'static str = "BatteryLevelIndicator";
        type Type = super::BatteryLevelIndicator;
        type ParentType = adw::PreferencesRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    
    #[glib::derived_properties]
    impl ObjectImpl for BatteryLevelIndicator {
        fn constructed(&self) {
            self.parent_constructed();
            self.level_bar.add_offset_value("full", 100.0);
            self.level_bar.add_offset_value("three-quarters", 75.0);
            self.level_bar.add_offset_value("half", 50.0);
            self.level_bar.add_offset_value("third", 25.0);
        }
    }

    // add offset
    impl WidgetImpl for BatteryLevelIndicator {}
    impl ListBoxRowImpl for BatteryLevelIndicator {}
    impl PreferencesRowImpl for BatteryLevelIndicator {}

    impl BatteryLevelIndicator {
        fn set_battery_level_from_i8(&self, level: i8) {
            let level = level.clamp(-1, 100);
            let levelbar = self.level_bar.get();
            let battery_label = self.battery_label.get();

            if level == -1 {
                levelbar.set_value(100.0);
                levelbar.set_sensitive(false);
                battery_label.set_label(&("Battery: ".to_string() + "Unavailable"));
            }
            else {
                levelbar.set_sensitive(true);
                levelbar.set_value(level as f64);
                battery_label.set_label(&("Battery: ".to_string() + &level.to_string() + "%"));
            }

            self.battery_level.set(level);
        }
    }
}

glib::wrapper! {
    pub struct BatteryLevelIndicator(ObjectSubclass<imp::BatteryLevelIndicator>)
        @extends adw::PreferencesRow, gtk::Widget, gtk::ListBoxRow,
        @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl BatteryLevelIndicator {
    pub fn new() -> Self {
        Object::builder()
            .build()
    }
}

impl Default for BatteryLevelIndicator {
    fn default() -> Self {
        Self::new()
    }
}
    
