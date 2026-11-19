use adw::prelude::*;
use adw::Window;
use gtk::prelude::*;
use gtk::{Label, ScrolledWindow, TextView};

use shanti_layout::LayoutDesc;

pub struct AboutFile {
    pub window: Window,
}

impl AboutFile {
    pub fn new(desc: &LayoutDesc) -> Self {
        let window = Window::builder()
            .title(&format!("About {}", desc.name))
            .modal(true)
            .default_width(460)
            .default_height(320)
            .build();

        let scrolled = ScrolledWindow::new();
        scrolled.set_hexpand(true);
        scrolled.set_vexpand(true);

        let text = TextView::new();
        text.set_editable(false);
        text.set_cursor_visible(false);
        text.set_wrap_mode(gtk::WrapMode::Word);

        let mut content = String::new();
        content.push_str(&format!("Name: {}\n", desc.name));
        content.push_str(&format!("Version: {}\n", desc.ver));
        if desc.file_ver != 0 {
            content.push_str(&format!("File version: {}\n", desc.file_ver));
        }
        if !desc.dev_name.is_empty() {
            content.push_str(&format!("Developer: {}\n", desc.dev_name));
        }
        if !desc.dev_comment.is_empty() {
            content.push_str(&format!("\n{}\n", desc.dev_comment));
        }

        text.buffer().set_text(&content);

        scrolled.set_child(Some(&text));
        window.set_content(Some(&scrolled));
        window.present();

        Self { window }
    }
}
