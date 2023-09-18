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

use self::application::OverskrideApplication;
use self::window::OverskrideWindow;

use config::{GETTEXT_PACKAGE, LOCALEDIR, PKGDATADIR};
use gettextrs::{bind_textdomain_codeset, bindtextdomain, textdomain};
use gtk::{gio, glib};
use gtk::prelude::*;

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
    let app = OverskrideApplication::new("com.kaii.Overskride", &gio::ApplicationFlags::empty());

    // Run the application. This function will block until the application
    // exits. Upon return, we have our exit code to return to the shell. (This
    // is the code you see when you do `echo $?` after running a command in a
    // terminal.
    app.run()
}
