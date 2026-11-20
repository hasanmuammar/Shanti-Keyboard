use adw::prelude::*;
use gtk::AboutDialog as GtkAboutDialog;

pub struct AboutDialog {
    pub window: adw::Window,
}

impl AboutDialog {
    pub fn new() -> Self {
        let about = GtkAboutDialog::new();
        about.set_program_name(Some("Shanti Keyboard"));
        about.set_version(Some(
            option_env!("SHANTI_VERSION").unwrap_or(env!("CARGO_PKG_VERSION")),
        ));
        about.set_website(Some("https://github.com/hasanmuammar/Shanti-Keyboard"));
        about.set_comments(Some("Bangla input method for GNOME and IBus"));
        about.set_license_type(gtk::License::Gpl30);

        let window = adw::Window::builder()
            .title("About Shanti Keyboard")
            .modal(true)
            .default_width(460)
            .default_height(320)
            .build();

        window.set_content(Some(&about));
        window.present();

        Self { window }
    }
}

impl Default for AboutDialog {
    fn default() -> Self {
        Self::new()
    }
}
