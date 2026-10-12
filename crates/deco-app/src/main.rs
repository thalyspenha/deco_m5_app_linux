mod credentials;
mod internet;
mod window;

use adw::prelude::*;

const APP_ID: &str = "io.github.thalys.DecoLinux";

fn main() -> gtk::glib::ExitCode {
    dotenvy::dotenv().ok();
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_activate(window::build);
    app.run()
}
