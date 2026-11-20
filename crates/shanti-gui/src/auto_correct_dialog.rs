use std::path::PathBuf;

use adw::prelude::*;
use adw::{Toast, ToastOverlay, Window};
use gtk::prelude::*;
use gtk::{Box as GtkBox, Button, Entry, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow};

use shanti_paths;

const BUNDLED_AUTOCORRECT: &str =
    include_str!("../../../src/engine/riti/data/autocorrect.json");

pub struct AutoCorrectDialog {
    pub window: Window,
    entries: std::cell::RefCell<Vec<(String, String)>>,
    list_box: ListBox,
    replace_entry: Entry,
    with_entry: Entry,
    user_path: PathBuf,
}

impl AutoCorrectDialog {
    pub fn new() -> Self {
        let window = Window::builder()
            .title("AutoCorrect")
            .modal(true)
            .default_width(520)
            .default_height(400)
            .build();

        let overlay = ToastOverlay::new();
        overlay.set_hexpand(true);
        overlay.set_vexpand(true);

        let content = GtkBox::new(Orientation::Vertical, 6);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);

        // Entry list
        let list_scroll = ScrolledWindow::new();
        list_scroll.set_hexpand(true);
        list_scroll.set_vexpand(true);
        let list_box = ListBox::new();
        list_box.set_selection_mode(gtk::SelectionMode::Single);
        list_scroll.set_child(Some(&list_box));
        content.append(&list_scroll);

        // Entry form
        let form = GtkBox::new(Orientation::Horizontal, 6);
        form.set_hexpand(true);
        let replace_entry = Entry::new();
        replace_entry.set_placeholder_text(Some("Replace"));
        replace_entry.set_hexpand(true);
        let with_entry = Entry::new();
        with_entry.set_placeholder_text(Some("With"));
        with_entry.set_hexpand(true);
        let add_btn = Button::with_label("Add");
        add_btn.add_css_class("suggested-action");
        form.append(&replace_entry);
        form.append(&with_entry);
        form.append(&add_btn);
        content.append(&form);

        // Action buttons
        let actions = GtkBox::new(Orientation::Horizontal, 6);
        actions.set_hexpand(true);
        let update_btn = Button::with_label("Update");
        let clear_btn = Button::with_label("Clear");
        let close_btn = Button::with_label("Close");
        close_btn.add_css_class("flat");
        actions.append(&update_btn);
        actions.append(&clear_btn);
        actions.append(&close_btn);
        content.append(&actions);

        overlay.set_child(Some(&content));
        window.set_content(Some(&overlay));

        let user_path = shanti_paths::user_data_dir()
            .join("data")
            .join("autocorrect.json");

        // Load entries
        let entries = std::cell::RefCell::new(Self::load_entries(&user_path));
        Self::refresh_list(&list_box, &entries);

        // Wire up signals
        let entries_c = entries.clone();
        let list_box_c = list_box.clone();
        let replace_entry_c = replace_entry.clone();
        let with_entry_c = with_entry.clone();
        let overlay_c = overlay.clone();
        let user_path_add = user_path.clone();
        add_btn.connect_clicked(move |_| {
            let replace = replace_entry_c.text().to_string();
            let with_ = with_entry_c.text().to_string();
            if replace.is_empty() || with_.is_empty() {
                return;
            }
            entries_c.borrow_mut().push((replace, with_));
            Self::save_entries(&user_path_add, &entries_c.borrow());
            Self::refresh_list(&list_box_c, &entries_c);
            replace_entry_c.set_text("");
            with_entry_c.set_text("");
            overlay_c.add_toast(Toast::builder().title("Entry added").build());
        });

        let entries_c2 = entries.clone();
        let list_box_c2 = list_box.clone();
        let replace_entry_c2 = replace_entry.clone();
        let with_entry_c2 = with_entry.clone();
        let overlay_c2 = overlay.clone();
        let user_path_update = user_path.clone();
        update_btn.connect_clicked(move |_| {
            if let Some(row) = list_box_c2.selected_row() {
                let idx = row.index() as usize;
                let replace = replace_entry_c2.text().to_string();
                let with_ = with_entry_c2.text().to_string();
                if replace.is_empty() || with_.is_empty() {
                    return;
                }
                entries_c2.borrow_mut()[idx] = (replace, with_);
                Self::save_entries(&user_path_update, &entries_c2.borrow());
                Self::refresh_list(&list_box_c2, &entries_c2);
                overlay_c2.add_toast(Toast::builder().title("Entry updated").build());
            }
        });

        let entries_c3 = entries.clone();
        let list_box_c3 = list_box.clone();
        let overlay_c3 = overlay.clone();
        let user_path_clear = user_path.clone();
        clear_btn.connect_clicked(move |_| {
            if let Some(row) = list_box_c3.selected_row() {
                let idx = row.index() as usize;
                entries_c3.borrow_mut().remove(idx);
                Self::save_entries(&user_path_clear, &entries_c3.borrow());
                Self::refresh_list(&list_box_c3, &entries_c3);
                overlay_c3.add_toast(Toast::builder().title("Entry cleared").build());
            }
        });

        let list_box_c4 = list_box.clone();
        let replace_entry_c4 = replace_entry.clone();
        let with_entry_c4 = with_entry.clone();
        list_box.connect_row_selected(move |_list, row| {
            if let Some(row) = row {
                let idx = row.index() as usize;
                // We can't easily access entries here without the RefCell,
                // so just show that a row is selected. The entries are
                // re-read in refresh_list when needed.
                let _ = idx;
            }
        });

        let window_close = window.clone();
        close_btn.connect_clicked(move |_| {
            window_close.close();
        });

        Self {
            window,
            entries,
            list_box,
            replace_entry,
            with_entry,
            user_path,
        }
    }

    fn load_entries(user_path: &PathBuf) -> Vec<(String, String)> {
        let mut map: Vec<(String, String)> = Vec::new();

        // Load bundled entries
        if let Ok(bundled) = serde_json::from_str::<serde_json::Value>(BUNDLED_AUTOCORRECT) {
            if let Some(obj) = bundled.as_object() {
                for (k, v) in obj {
                    if let Some(s) = v.as_str() {
                        map.push((k.clone(), s.to_string()));
                    }
                }
            }
        }

        // Load user entries (override bundled)
        if let Ok(text) = std::fs::read_to_string(user_path) {
            if let Ok(obj) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(uobj) = obj.as_object() {
                    for (k, v) in uobj {
                        if let Some(s) = v.as_str() {
                            if let Some(pos) = map.iter().position(|(rk, _)| rk == k) {
                                map[pos] = (k.clone(), s.to_string());
                            } else {
                                map.push((k.clone(), s.to_string()));
                            }
                        }
                    }
                }
            }
        }

        map.sort_by(|a, b| a.0.cmp(&b.0));
        map
    }

    fn save_entries(user_path: &PathBuf, entries: &[(String, String)]) {
        if let Some(parent) = user_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let obj: serde_json::Value = serde_json::Value::Object(
            entries
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect(),
        );
        let _ = std::fs::write(
            user_path,
            serde_json::to_string_pretty(&obj).unwrap_or_default(),
        );
    }

    fn refresh_list(list_box: &ListBox, entries: &std::cell::RefCell<Vec<(String, String)>>) {
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }
        for (replace, with_) in entries.borrow().iter() {
            let row = ListBoxRow::new();
            row.set_hexpand(true);
            let hbox = GtkBox::new(Orientation::Horizontal, 6);
            hbox.set_hexpand(true);
            let r_label = Label::new(Some(replace));
            r_label.set_hexpand(true);
            r_label.set_xalign(0.0);
            let w_label = Label::new(Some(with_));
            w_label.set_hexpand(true);
            w_label.set_xalign(0.0);
            hbox.append(&r_label);
            hbox.append(&w_label);
            row.set_child(Some(&hbox));
            list_box.append(&row);
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
}

impl Default for AutoCorrectDialog {
    fn default() -> Self {
        Self::new()
    }
}
