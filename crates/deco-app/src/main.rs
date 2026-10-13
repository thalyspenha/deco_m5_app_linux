mod blocked;
mod client;
mod credentials;
mod deco;
mod internet;
mod window;

use adw::prelude::*;

const APP_ID: &str = "io.github.thalys.DecoLinux";

fn main() -> gtk::glib::ExitCode {
    dotenvy::dotenv().ok();
    gtk::Window::set_default_icon_name(APP_ID);
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_activate(window::build);
    app.run()
}
