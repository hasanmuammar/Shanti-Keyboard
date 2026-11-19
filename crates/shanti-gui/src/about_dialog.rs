use adw::prelude::*;
use adw::Window;
use gtk::prelude::*;
use gtk::AboutDialog;

pub struct AboutDialog {
    pub window: Window,
}

impl AboutDialog {
    pub fn new() -> Self {
        let about = AboutDialog::builder()
            .program_name("Shanti Keyboard")
            .version(env!("CARGO_PKG_VERSION"))
            .website("https://github.com/hasanmuammar/Shanti-Keyboard")
            .comments("Bangla input method for GNOME and IBus")
            .build();

        let window = Window::builder()
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
