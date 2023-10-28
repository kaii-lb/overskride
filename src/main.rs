/* main.rs
 *
 * Copyright 2023 kaii
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

mod application;
mod config;
mod window;
#[path = "bluetooth/message.rs"] mod message;
#[path = "bluetooth/bluetooth_settings.rs"] mod bluetooth_settings;
#[path = "bluetooth/device.rs"] mod device;
#[path = "bluetooth/agent.rs"] mod agent;
#[path = "bluetooth/services.rs"] mod services;
#[path = "bluetooth/audio_profiles.rs"] mod audio_profiles;
#[path = "bluetooth/battery.rs"] mod battery;
#[path = "obex/obex.rs"] mod obex;
#[path = "obex/obex_utils.rs"] mod obex_utils;
#[path = "widgets/connected_switch_row.rs"] mod connected_switch_row;
#[path = "widgets/device_action_row.rs"] mod device_action_row;
#[path = "widgets/receiving_popover.rs"] mod receiving_popover;
#[path = "widgets/receiving_row.rs"] mod receiving_row;
#[path = "widgets/startup_error_message.rs"] mod startup_error_message;
#[path = "widgets/selectable_row.rs"] mod selectable_row;
#[path = "widgets/battery_indicator.rs"] mod battery_indicator;


use self::application::OverskrideApplication;
use self::window::OverskrideWindow;

use config::{GETTEXT_PACKAGE, LOCALEDIR, PKGDATADIR};
use gettextrs::{bind_textdomain_codeset, bindtextdomain, textdomain};
use gtk::{gio, glib};
use gtk::prelude::*;
use gtk::gdk::Display;

fn main() -> glib::ExitCode {
    // Set up gettext translations
    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to set the text domain encoding");
    textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    // Load resources
    // let resources = gio::Resource::load(PKGDATADIR.to_owned() + "/overskride.gresource")
        // .expect("Could not load resources");
    // gio::resources_register(&resources);

	let resources = match std::env::var("MESON_DEVENV") {
    Err(_) => gio::Resource::load(PKGDATADIR.to_owned() + "/overskride.gresource")
        .expect("Unable to find overskride.gresource"),
    Ok(_) => match std::env::current_exe() {
        Ok(path) => {
            let mut resource_path = path;
            resource_path.pop();
            resource_path.push("overskride.gresource");
            gio::Resource::load(&resource_path)
                .expect("Unable to find overskride.gresource in devenv")
            }
            Err(err) => {
                eprintln!("Unable to find the current path: {}", err);
                return 1.into();
            }
        },
    };

    gio::resources_register(&resources);

    // Create a new GtkApplication. The application manages our main loop,
    // application windows, integration with the window manager/compositor, and
    // desktop features such as file opening and single-instance applications.
    let app = OverskrideApplication::new("io.github.kaii_lb.Overskride", &gio::ApplicationFlags::empty());

	app.connect_startup(|_| {
		load_css()
	});

    // Run the application. This function will block until the application
    // exits. Upon return, we have our exit code to return to the shell. (This
    // is the code you see when you do `echo $?` after running a command in a
    // terminal.
    app.run()
}

fn load_css() {
	let provider = gtk::CssProvider::new();
	provider.load_from_resource("/io/github/kaii_lb/Overskride/gtk/style.css");

	gtk::style_context_add_provider_for_display(
		&Display::default().expect("could not connect to a display"),
		&provider,
		gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
	);
}
