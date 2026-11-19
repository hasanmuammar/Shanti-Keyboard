use std::path::PathBuf;
use std::rc::Rc;

use adw::prelude::*;
use adw::ViewStack;
use gdk::prelude::*;
use gdk_pixbuf;
use gtk::prelude::*;
use gtk::{
    Box as GtkBox, Button, Label, Orientation, Picture, WidgetExt,
};

use crate::keyboard::KeyboardWidget;
use shanti_layout::{decode_image, LayoutFile, LayoutKind};

#[derive(Clone, Default)]
pub struct LayoutViewer {
    pub stack: ViewStack,
    keyboard_widget: Rc<KeyboardWidget>,
    image_box: GtkBox,
    normal_button: Button,
    altgr_button: Button,
    about_button: Button,
    header_box: GtkBox,
    current_desc: Option<shanti_layout::LayoutDesc>,
}

impl LayoutViewer {
    pub fn new() -> Self {
        let stack = ViewStack::new();
        stack.set_vexpand(true);
        stack.set_hexpand(true);

        let keyboard_widget = Rc::new(KeyboardWidget::new());
        stack.add_titled(
            keyboard_widget.widget(),
            Some("keyboard"),
            Some("Keyboard"),
        );

        let image_box = GtkBox::new(Orientation::Vertical, 0);
        image_box.set_vexpand(true);
        image_box.set_hexpand(true);
        stack.add_titled(&image_box, Some("image"), Some("Image"));

        let header_box = GtkBox::new(Orientation::Horizontal, 6);
        header_box.set_hexpand(false);
        header_box.set_margin_top(6);
        header_box.set_margin_start(6);
        header_box.set_margin_end(6);
        header_box.set_margin_bottom(6);
        header_box.set_visible(false);

        let normal_button = Button::with_label("Normal");
        normal_button.add_css_class("flat");
        normal_button.set_sensitive(false);

        let altgr_button = Button::with_label("AltGr");
        altgr_button.add_css_class("flat");
        altgr_button.set_sensitive(false);

        let about_button = Button::with_label("About");
        about_button.add_css_class("flat");
        about_button.set_sensitive(false);

        header_box.append(&normal_button);
        header_box.append(&altgr_button);
        header_box.append(&about_button);

        stack.set_hhomogeneous(false);
        stack.set_vhomogeneous(false);

        Self {
            stack,
            keyboard_widget,
            image_box,
            normal_button,
            altgr_button,
            about_button,
            header_box,
            current_desc: None,
        }
    }

    pub fn widget(&self) -> &ViewStack {
        &self.stack
    }

    pub fn header(&self) -> &GtkBox {
        &self.header_box
    }

    pub fn normal_button(&self) -> &Button {
        &self.normal_button
    }

    pub fn altgr_button(&self) -> &Button {
        &self.altgr_button
    }

    pub fn about_button(&self) -> &Button {
        &self.about_button
    }

    pub fn set_layout(&self, layout_text: &str) {
        let layout = match LayoutFile::parse(layout_text) {
            Ok(layout) => layout,
            Err(_) => return,
        };

        self.current_desc = Some(layout.desc().clone());

        match layout.desc().kind {
            LayoutKind::Fixed => {
                let keymap = layout.keymap();
                self.keyboard_widget.set_keymap(keymap);
                self.stack.set_visible_child_name("keyboard");
                self.header_box.set_visible(true);
                self.normal_button.set_sensitive(true);
                let has_altgr = self.keyboard_widget_has_altgr();
                self.altgr_button.set_sensitive(has_altgr);
                self.about_button.set_sensitive(true);
                self.keyboard_widget.set_mode(crate::keyboard::Mode::Normal);
                self.normal_button.add_css_class("raised");
                self.altgr_button.remove_css_class("raised");
            }
            LayoutKind::Phonetic | LayoutKind::Khipro => {
                self.stack.set_visible_child_name("image");
                self.header_box.set_visible(true);
                self.normal_button.set_sensitive(false);
                self.altgr_button.set_sensitive(false);
                self.about_button.set_sensitive(true);
                self.render_image(layout.desc().image.clone());
            }
        }
    }

    fn keyboard_widget_has_altgr(&self) -> bool {
        self.keyboard_widget.has_altgr()
    }

    fn render_image(&self, image_b64: String) {
        if let Some(child) = self.image_box.first_child() {
            self.image_box.remove(&child);
        }

        if image_b64.is_empty() {
            let label = Label::new(Some("No image to display!"));
            label.set_vexpand(true);
            label.set_hexpand(true);
            label.set_halign(gtk::Align::Center);
            label.set_valign(gtk::Align::Center);
            self.image_box.append(&label);
            return;
        }

        let decoded = decode_image(&image_b64);
        if decoded.is_empty() {
            let label = Label::new(Some("Failed to decode image"));
            label.set_vexpand(true);
            label.set_hexpand(true);
            label.set_halign(gtk::Align::Center);
            label.set_valign(gtk::Align::Center);
            self.image_box.append(&label);
            return;
        }

        // Try to load as PNG via GdkPixbuf / GTexture
        let loader = match gdk_pixbuf::PixbufLoader::new() {
            Ok(l) => l,
            Err(_) => {
                let label = Label::new(Some("Failed to create image loader"));
                label.set_vexpand(true);
                label.set_hexpand(true);
                label.set_halign(gtk::Align::Center);
                label.set_valign(gtk::Align::Center);
                self.image_box.append(&label);
                return;
            }
        };
        if loader.write(&decoded).is_err() {
            let label = Label::new(Some("Failed to decode image"));
            label.set_vexpand(true);
            label.set_hexpand(true);
            label.set_halign(gtk::Align::Center);
            label.set_valign(gtk::Align::Center);
            self.image_box.append(&label);
            return;
        }
        loader.close().ok();
        let pixbuf = match loader.pixbuf() {
            Some(p) => p,
            None => {
                let label = Label::new(Some("Failed to decode image"));
                label.set_vexpand(true);
                label.set_hexpand(true);
                label.set_halign(gtk::Align::Center);
                label.set_valign(gtk::Align::Center);
                self.image_box.append(&label);
                return;
            }
        };

        let picture = Picture::for_pixbuf(&pixbuf);
        picture.set_hexpand(true);
        picture.set_vexpand(true);
        picture.set_content_fit(gtk::PictureContentFit::Contain);
        self.image_box.append(&picture);
    }

    pub fn desc(&self) -> Option<&shanti_layout::LayoutDesc> {
        self.current_desc.as_ref()
    }
}

impl Default for LayoutViewer {
    fn default() -> Self {
        Self::new()
    }
}
