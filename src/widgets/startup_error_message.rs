use adw::gio::{ActionGroup, ActionMap};
use glib::Object;
use gtk::glib;
use adw::subclass::prelude::AdwApplicationWindowImpl;
use gtk::subclass::prelude::*;

mod imp {
    use adw::prelude::ButtonExt;

    use super::*;    

    /// a custom error message if the startup failed, showing the user a way to fix the error
    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/kaii_lb/Overskride/gtk/startup-error-message.ui")]
    pub struct StartupErrorMessage {
        #[template_child]
        pub run_enable_bluetooth_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub error_toast_overlay: TemplateChild<adw::ToastOverlay>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StartupErrorMessage {
        const NAME: &'static str = "StartupErrorMessage";
        type Type = super::StartupErrorMessage;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    
    impl ObjectImpl for StartupErrorMessage {
        fn constructed(&self) {
            self.parent_constructed();
            let toast_overlay = self.error_toast_overlay.get();
            self.run_enable_bluetooth_button.connect_clicked(move |_| {
                
                if std::env::var("container").is_err() {
                    let argv = [std::ffi::OsStr::new("pkexec"), std::ffi::OsStr::new("systemctl"), std::ffi::OsStr::new("enable"),
                    std::ffi::OsStr::new("--now"), std::ffi::OsStr::new("bluetooth")];

					let argv2 = [std::ffi::OsStr::new("pkexec"), std::ffi::OsStr::new("systemctl"), std::ffi::OsStr::new("start"),
                    std::ffi::OsStr::new("bluetooth")];
                    
                    gtk::gio::Subprocess::newv(&argv, gtk::gio::SubprocessFlags::STDERR_PIPE).expect("cannot enable bluetooth by pkexec");
                    gtk::gio::Subprocess::newv(&argv2, gtk::gio::SubprocessFlags::STDERR_PIPE).expect("cannot enable bluetooth by pkexec");
                    
                    let toast = adw::Toast::new("applying commands through pkexec");
                    toast.set_timeout(5);

                    toast_overlay.add_toast(toast);
                }
                else {
                    let display = gtk::gdk::Display::default().unwrap();
                    let clipboard = gtk::prelude::DisplayExt::clipboard(&display);
                    clipboard.set_text("sudo systemctl enable --now bluetooth");

                    let toast = adw::Toast::new("copied command to clipboard");
                    toast.set_timeout(5);

                    toast_overlay.add_toast(toast);
                }
            });
        }
    }

    impl WidgetImpl for StartupErrorMessage {}
    impl AdwApplicationWindowImpl for StartupErrorMessage {}
    impl ApplicationWindowImpl for StartupErrorMessage {}
    impl WindowImpl for StartupErrorMessage {}

    impl StartupErrorMessage {}
}

glib::wrapper! {
    pub struct StartupErrorMessage(ObjectSubclass<imp::StartupErrorMessage>)
        @extends adw::ApplicationWindow, gtk::Widget, gtk::Window, gtk::ApplicationWindow,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager, ActionGroup, ActionMap;
}

impl StartupErrorMessage {
    /// creates a new `StartupErrorMessage`
    pub fn new() -> Self {
        Object::builder()
            .build()
    }
}

impl Default for StartupErrorMessage {
	fn default() -> Self {
		Self::new()
	}
}
