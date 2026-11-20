use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use adw::prelude::*;
use adw::{Toast, ToastOverlay, Window};
use gtk::{Box as GtkBox, Button, Entry, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow};

use shanti_core::preview::Preview;

const BUNDLED_AUTOCORRECT: &str = include_str!("../../../src/engine/riti/data/autocorrect.json");

/// Autocorrect entry editor, ported from the retired Qt `AutoCorrectDialog`.
///
/// Bundled entries are read-only; editing or adding writes the user override
/// file (`~/.local/share/shanti-keyboard/autocorrect.json`), which the engine
/// merges over the bundled dictionary. The previews run through the real
/// engine (`shanti_core::preview`), not a second transliteration parser.
pub struct AutoCorrectDialog {
    pub window: Window,
    bundled: BTreeMap<String, String>,
    user: Rc<RefCell<BTreeMap<String, String>>>,
    list_box: ListBox,
    replace_entry: Entry,
    with_entry: Entry,
    preview_replace: Label,
    preview_with: Label,
    user_path: std::path::PathBuf,
}

impl AutoCorrectDialog {
    pub fn new() -> Self {
        let window = Window::builder()
            .title("Edit Phonetic AutoCorrect entries")
            .modal(true)
            .default_width(560)
            .default_height(440)
            .build();

        let overlay = ToastOverlay::new();
        overlay.set_hexpand(true);
        overlay.set_vexpand(true);

        let content = GtkBox::new(Orientation::Vertical, 8);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);

        let title = Label::new(Some("Auto Correct"));
        title.add_css_class("title-2");
        title.set_xalign(0.0);
        content.append(&title);

        let form = GtkBox::new(Orientation::Horizontal, 8);
        form.set_hexpand(true);

        let fields = GtkBox::new(Orientation::Vertical, 4);
        fields.set_hexpand(true);
        let replace_entry = Entry::new();
        replace_entry.set_placeholder_text(Some("Replace"));
        replace_entry.set_hexpand(true);
        let with_entry = Entry::new();
        with_entry.set_placeholder_text(Some("With"));
        with_entry.set_hexpand(true);
        fields.append(&replace_entry);
        fields.append(&with_entry);
        form.append(&fields);

        let previews = GtkBox::new(Orientation::Vertical, 4);
        previews.set_hexpand(true);
        let preview_replace = Label::new(None);
        preview_replace.add_css_class("dim-label");
        preview_replace.set_xalign(0.0);
        let preview_with = Label::new(None);
        preview_with.add_css_class("dim-label");
        preview_with.set_xalign(0.0);
        previews.append(&preview_replace);
        previews.append(&preview_with);
        form.append(&previews);

        content.append(&form);

        let list_scroll = ScrolledWindow::new();
        list_scroll.set_hexpand(true);
        list_scroll.set_vexpand(true);
        let list_box = ListBox::new();
        list_box.set_selection_mode(gtk::SelectionMode::Single);
        list_scroll.set_child(Some(&list_box));
        content.append(&list_scroll);

        let actions = GtkBox::new(Orientation::Horizontal, 6);
        actions.set_halign(gtk::Align::End);
        let clear_btn = Button::with_label("Clear");
        let update_btn = Button::with_label("Add/Update");
        update_btn.add_css_class("suggested-action");
        let close_btn = Button::with_label("Close");
        close_btn.add_css_class("flat");
        actions.append(&clear_btn);
        actions.append(&update_btn);
        actions.append(&close_btn);
        content.append(&actions);

        overlay.set_child(Some(&content));
        window.set_content(Some(&overlay));

        let bundled = Self::bundled_entries();
        let user = Rc::new(RefCell::new(Self::load_user_entries()));
        let user_path = shanti_paths::user_data_dir().join("autocorrect.json");
        let preview = Rc::new(RefCell::new(Preview::new()));

        let dialog = Self {
            window,
            bundled,
            user,
            list_box,
            replace_entry,
            with_entry,
            preview_replace,
            preview_with,
            user_path,
        };
        dialog.refresh_list();

        // Previews and button sensitivity track the two entry fields.
        let replace = dialog.replace_entry.clone();
        let with = dialog.with_entry.clone();
        let preview_replace = dialog.preview_replace.clone();
        let _preview_with = dialog.preview_with.clone();
        let update_btn_c = update_btn.clone();
        let clear_btn_c = clear_btn.clone();
        let preview_c = Rc::clone(&preview);
        replace.connect_changed(move |entry| {
            let text = entry.text().to_string();
            let converted = if text.is_empty() {
                String::new()
            } else {
                preview_c.borrow_mut().transliterate(&text)
            };
            preview_replace.set_text(&converted);
            update_btn_c.set_sensitive(!text.is_empty() && !with.text().is_empty());
            clear_btn_c.set_sensitive(!text.is_empty() || !with.text().is_empty());
        });

        let replace = dialog.replace_entry.clone();
        let with = dialog.with_entry.clone();
        let preview_replace = dialog.preview_replace.clone();
        let preview_with = dialog.preview_with.clone();
        let update_btn_c = update_btn.clone();
        let clear_btn_c = clear_btn.clone();
        let preview_c = Rc::clone(&preview);
        with.connect_changed(move |entry| {
            let text = entry.text().to_string();
            let replace_text = replace.text().to_string();
            // Smiley rule: an entry that maps a string to itself is shown
            // verbatim instead of being transliterated.
            let converted = if text.is_empty() {
                String::new()
            } else if replace_text == text {
                text.clone()
            } else {
                preview_c.borrow_mut().transliterate(&text)
            };
            if replace_text.is_empty() {
                preview_replace.set_text("");
            }
            preview_with.set_text(&converted);
            update_btn_c.set_sensitive(!text.is_empty() && !replace_text.is_empty());
            clear_btn_c.set_sensitive(!text.is_empty() || !replace_text.is_empty());
        });

        // Selecting a row fills the editor.
        let dialog_c = dialog.clone();
        dialog.list_box.connect_row_selected(move |_, row| {
            let Some(row) = row else { return };
            let Some((replace, with)) = dialog_c.row_entry(row.index()) else {
                return;
            };
            dialog_c.replace_entry.set_text(&replace);
            dialog_c.with_entry.set_text(&with);
        });

        let dialog_c = dialog.clone();
        clear_btn.connect_clicked(move |_| {
            dialog_c.replace_entry.set_text("");
            dialog_c.with_entry.set_text("");
        });

        let dialog_c = dialog.clone();
        let overlay_c = overlay.clone();
        update_btn.connect_clicked(move |_| {
            let replace = dialog_c.replace_entry.text().trim().to_string();
            let with = dialog_c.with_entry.text().trim().to_string();
            if replace.is_empty() || with.is_empty() {
                return;
            }
            dialog_c.user.borrow_mut().insert(replace, with);
            dialog_c.save_user_entries();
            dialog_c.refresh_list();
            overlay_c.add_toast(Toast::builder().title("Entry saved").build());
        });

        let window_c = dialog.window.clone();
        close_btn.connect_clicked(move |_| {
            window_c.close();
        });

        dialog
    }

    fn bundled_entries() -> BTreeMap<String, String> {
        serde_json::from_str(BUNDLED_AUTOCORRECT).unwrap_or_default()
    }

    fn load_user_entries() -> BTreeMap<String, String> {
        std::fs::read_to_string(shanti_paths::user_data_dir().join("autocorrect.json"))
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    fn save_user_entries(&self) {
        if let Some(parent) = self.user_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&*self.user.borrow()) {
            let _ = std::fs::write(&self.user_path, json);
        }
    }

    /// Merged view: bundled entries with the user overrides applied.
    fn merged_entries(&self) -> Vec<(String, String)> {
        let user = self.user.borrow();
        let mut entries: BTreeMap<String, String> = self.bundled.clone();
        for (replace, with) in user.iter() {
            entries.insert(replace.clone(), with.clone());
        }
        entries.into_iter().collect()
    }

    fn row_entry(&self, index: i32) -> Option<(String, String)> {
        self.merged_entries().get(index as usize).cloned()
    }

    fn refresh_list(&self) {
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
        for (replace, with) in self.merged_entries() {
            let row = ListBoxRow::new();
            let hbox = GtkBox::new(Orientation::Horizontal, 6);
            hbox.set_hexpand(true);
            let r_label = Label::new(Some(&replace));
            r_label.set_hexpand(true);
            r_label.set_xalign(0.0);
            let w_label = Label::new(Some(&with));
            w_label.set_hexpand(true);
            w_label.set_xalign(0.0);
            hbox.append(&r_label);
            hbox.append(&w_label);
            row.set_child(Some(&hbox));
            self.list_box.append(&row);
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
}

impl Clone for AutoCorrectDialog {
    fn clone(&self) -> Self {
        Self {
            window: self.window.clone(),
            bundled: self.bundled.clone(),
            user: Rc::clone(&self.user),
            list_box: self.list_box.clone(),
            replace_entry: self.replace_entry.clone(),
            with_entry: self.with_entry.clone(),
            preview_replace: self.preview_replace.clone(),
            preview_with: self.preview_with.clone(),
            user_path: self.user_path.clone(),
        }
    }
}

impl Default for AutoCorrectDialog {
    fn default() -> Self {
        Self::new()
    }
}
