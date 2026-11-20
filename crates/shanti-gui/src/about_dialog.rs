use adw::prelude::*;
use gtk::prelude::*;
use gtk::AboutDialog as GtkAboutDialog;

pub struct AboutDialog {
    pub window: adw::Window,
}

impl AboutDialog {
    pub fn new() -> Self {
        let about = GtkAboutDialog::new();
        about.set_program_name(Some("Shanti Keyboard"));
        about.set_version(Some(env!("CARGO_PKG_VERSION")));
        about.set_website("https://github.com/hasanmuammar/Shanti-Keyboard");
        about.set_comments("Bangla input method for GNOME and IBus");

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

    pub fn window(&self) -> &adw::Window {
        &self.window
    }
}

impl Default for AboutDialog {
    fn default() -> Self {
        Self::new()
    }
}
