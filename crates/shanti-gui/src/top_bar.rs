use adw::prelude::*;
use gtk::gdk;
use gtk::{Box as GtkBox, Button, GestureClick, Window};

pub struct TopBar {
    pub window: Window,
    pub icon_btn: Button,
    pub layout_btn: Button,
    pub view_btn: Button,
    pub settings_btn: Button,
    pub shutdown_btn: Button,
}

impl TopBar {
    pub fn new() -> Self {
        let window = Window::builder()
            .title("Shanti Keyboard")
            .decorated(false) // frameless
            .default_width(220)
            .default_height(44)
            .build();
        window.set_hide_on_close(true);

        let content = GtkBox::new(gtk::Orientation::Horizontal, 6);
        content.set_margin_top(6);
        content.set_margin_bottom(6);
        content.set_margin_start(6);
        content.set_margin_end(6);

        let icon_btn = Button::from_icon_name("org.shanti.ShantiIbus");
        icon_btn.add_css_class("pill");
        icon_btn.set_tooltip_text(Some("About Shanti Keyboard"));

        let layout_btn = Button::from_icon_name("input-keyboard");
        layout_btn.add_css_class("pill");
        layout_btn.set_tooltip_text(Some("Keyboard Layout"));

        let view_btn = Button::from_icon_name("accessories-text-editor");
        view_btn.add_css_class("pill");
        view_btn.set_tooltip_text(Some("View Layout"));

        let settings_btn = Button::from_icon_name("preferences-system");
        settings_btn.add_css_class("pill");
        settings_btn.set_tooltip_text(Some("Settings"));

        let shutdown_btn = Button::from_icon_name("system-shutdown");
        shutdown_btn.add_css_class("pill");
        shutdown_btn.add_css_class("destructive-action");
        shutdown_btn.set_tooltip_text(Some("Quit"));

        content.append(&icon_btn);
        content.append(&layout_btn);
        content.append(&view_btn);
        content.append(&settings_btn);
        content.append(&shutdown_btn);

        window.set_child(Some(&content));

        // Frameless windows have no titlebar to grab, so let a press anywhere
        // on the bar start a compositor move — the GTK4 equivalent of the Qt
        // TopBar's manual drag handling.
        let click = GestureClick::new();
        let window_c = window.clone();
        click.connect_pressed(move |gesture, _n_press, x, y| {
            let (Some(device), Some(surface)) =
                (gesture.current_event_device(), window_c.surface())
            else {
                return;
            };
            let Some(toplevel) = surface.downcast_ref::<gdk::Toplevel>() else {
                return;
            };
            let time = gesture
                .current_event()
                .map(|event| event.time())
                .unwrap_or_default();
            toplevel.begin_move(&device, gesture.current_button() as i32, x, y, time);
        });
        window.add_controller(click);

        Self {
            window,
            icon_btn,
            layout_btn,
            view_btn,
            settings_btn,
            shutdown_btn,
        }
    }

    #[allow(dead_code)]
    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn present(&self) {
        self.window.present();
    }
}

impl Default for TopBar {
    fn default() -> Self {
        Self::new()
    }
}
