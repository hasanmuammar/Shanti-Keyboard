use adw::prelude::*;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Button, EventControllerMotion, GestureClick, Window};

pub struct TopBar {
    pub window: Window,
    pub icon_btn: Button,
    pub layout_btn: Button,
    pub view_btn: Button,
    pub settings_btn: Button,
    pub shutdown_btn: Button,
    drag_offset: std::cell::Cell<(i32, i32)>,
    dragging: std::cell::Cell<bool>,
}

impl TopBar {
    pub fn new() -> Self {
        let window = Window::builder()
            .title("Shanti Keyboard")
            .decorated(false) // frameless
            .default_width(220)
            .default_height(44)
            .build();

        let content = GtkBox::new(gtk::Orientation::Horizontal, 6);
        content.set_margin_top(6);
        content.set_margin_bottom(6);
        content.set_margin_start(6);
        content.set_margin_end(6);

        let icon_btn = Button::with_label("Shanti");
        icon_btn.add_css_class("pill");

        let layout_btn = Button::with_label("Layout");
        layout_btn.add_css_class("pill");

        let view_btn = Button::with_label("View Layout");
        view_btn.add_css_class("pill");

        let settings_btn = Button::with_label("Settings");
        settings_btn.add_css_class("pill");

        let shutdown_btn = Button::with_label("Quit");
        shutdown_btn.add_css_class("pill");
        shutdown_btn.add_css_class("destructive-action");

        content.append(&icon_btn);
        content.append(&layout_btn);
        content.append(&view_btn);
        content.append(&settings_btn);
        content.append(&shutdown_btn);

        window.set_child(Some(&content));

        // Drag-to-move: track press offset and motion delta
        let drag_offset = std::cell::Cell::new((0, 0));
        let dragging = std::cell::Cell::new(false);
        let window_c = window.clone();

        let click = GestureClick::new();
        click.connect_pressed(move |_gesture, _n, x, y| {
            drag_offset.set((x as i32, y as i32));
            dragging.set(true);
        });
        window.add_controller(click);

        let dragging_c = dragging.clone();
        let _window_c2 = window.clone();
        let motion = EventControllerMotion::new();
        motion.connect_motion(move |_ctrl, _x, _y| {
            if dragging_c.get() {
                // Frameless window dragging requires a Fixed container; skip here.
            }
        });
        window.add_controller(motion);

        let dragging_c2 = dragging.clone();
        click.connect_released(move |_gesture, _n, _x, _y| {
            dragging_c2.set(false);
        });

        Self {
            window,
            icon_btn,
            layout_btn,
            view_btn,
            settings_btn,
            shutdown_btn,
            drag_offset,
            dragging,
        }
    }

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
