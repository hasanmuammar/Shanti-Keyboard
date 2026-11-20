use std::cell::RefCell;
use std::rc::Rc;

use crate::auto_correct_dialog::AutoCorrectDialog;
use adw::prelude::*;
use adw::{ActionRow, ComboRow, PreferencesGroup, PreferencesPage, PreferencesWindow};
use gtk::{Button, StringList, Switch};
use shanti_config::Settings;

use shanti_desktop::{install_files, remove_files, Error, Paths};

/// A stub `ServiceControl` used until the GTK panel wires up a real
/// DBus/systemd backend. It records calls so tests can assert on them.
#[derive(Default)]
pub struct StubService {
    pub calls: RefCell<Vec<String>>,
}

impl shanti_desktop::ServiceControl for StubService {
    fn is_active(&self) -> Result<bool, Error> {
        self.calls.borrow_mut().push("is_active".to_string());
        Ok(false)
    }

    fn reload(&self) -> Result<(), Error> {
        self.calls.borrow_mut().push("reload".to_string());
        Ok(())
    }

    fn restart_unit(&self) -> Result<(), Error> {
        self.calls.borrow_mut().push("restart_unit".to_string());
        Ok(())
    }
}

/// A preferences row with a switch. The switch starts from the stored value
/// and writes the store on every change, which is the libadwaita convention
/// (there is no OK/Apply button) and matches what the Qt dialog wrote on OK.
fn add_switch(
    group: &PreferencesGroup,
    title: &str,
    settings: &Rc<RefCell<Settings>>,
    get: fn(&Settings) -> bool,
    apply: fn(&mut Settings, bool),
) -> Switch {
    let row = ActionRow::new();
    row.set_title(title);

    let switch = Switch::new();
    switch.set_active(get(&settings.borrow()));
    switch.set_valign(gtk::Align::Center);
    row.add_suffix(&switch);
    group.add(&row);

    let settings = Rc::clone(settings);
    switch.connect_active_notify(move |switch| {
        let mut settings = settings.borrow_mut();
        apply(&mut settings, switch.is_active());
        let _ = settings.save();
    });

    switch
}

/// A preferences row with a drop-down. `selected` is the index to preselect;
/// `apply` receives the new index.
fn add_combo(
    group: &PreferencesGroup,
    title: &str,
    options: &[&str],
    selected: u32,
    settings: &Rc<RefCell<Settings>>,
    apply: fn(&mut Settings, u32),
) -> ComboRow {
    let row = ComboRow::new();
    row.set_title(title);
    row.set_model(Some(&StringList::new(options)));
    row.set_selected(selected);
    group.add(&row);

    let settings = Rc::clone(settings);
    row.connect_selected_notify(move |row| {
        let mut settings = settings.borrow_mut();
        apply(&mut settings, row.selected());
        let _ = settings.save();
    });

    row
}

pub struct SettingsDialog {
    pub window: PreferencesWindow,
}

impl SettingsDialog {
    pub fn new() -> Self {
        let settings = Rc::new(RefCell::new(Settings::load().unwrap_or_default()));

        let window = PreferencesWindow::new();
        window.set_title(Some("Shanti Keyboard Settings"));
        window.set_default_width(640);
        window.set_default_height(480);

        // ---- General page ----
        let general = PreferencesPage::new();
        general.set_title("General");
        general.set_icon_name(Some("preferences-system-symbolic"));

        let general_group = PreferencesGroup::new();
        general_group.set_title("General");
        general.add(&general_group);

        add_switch(
            &general_group,
            "Enter key closes previous window",
            &settings,
            |s| s.enter_key_closes_prev_win,
            |s, on| s.enter_key_closes_prev_win = on,
        );
        add_switch(
            &general_group,
            "Smart quoting",
            &settings,
            |s| s.smart_quoting,
            |s, on| s.smart_quoting = on,
        );

        add_combo(
            &general_group,
            "Candidate window orientation",
            &["Horizontal", "Vertical"],
            if settings.borrow().candidate_win_horizontal {
                0
            } else {
                1
            },
            &settings,
            |s, index| s.candidate_win_horizontal = index == 0,
        );

        add_switch(
            &general_group,
            "Include English in candidate window",
            &settings,
            |s| s.suggestion_include_english,
            |s, on| s.suggestion_include_english = on,
        );

        add_combo(
            &general_group,
            "Encoding",
            &["Unicode", "ANSI"],
            if settings.borrow().ansi_encoding {
                1
            } else {
                0
            },
            &settings,
            |s, index| s.ansi_encoding = index == 1,
        );

        // ---- Phonetic page ----
        let phonetic = PreferencesPage::new();
        phonetic.set_title("Phonetic");
        phonetic.set_icon_name(Some("accessories-character-map-symbolic"));

        let phonetic_group = PreferencesGroup::new();
        phonetic_group.set_title("Phonetic keyboard layout");
        phonetic.add(&phonetic_group);

        add_switch(
            &phonetic_group,
            "Show candidate window",
            &settings,
            |s| s.show_cw_phonetic,
            |s, on| s.show_cw_phonetic = on,
        );

        let autocorrect_btn = Button::with_label("Manage AutoCorrect entries");
        autocorrect_btn.set_hexpand(true);
        autocorrect_btn.set_halign(gtk::Align::End);
        autocorrect_btn.connect_clicked(move |_| {
            let dialog = AutoCorrectDialog::new();
            dialog.window().present();
        });
        phonetic_group.add(&autocorrect_btn);

        // ---- Fixed page ----
        let fixed = PreferencesPage::new();
        fixed.set_title("Fixed");
        fixed.set_icon_name(Some("accessories-text-editor-symbolic"));

        let fixed_group = PreferencesGroup::new();
        fixed_group.set_title("Fixed keyboard layout");
        fixed.add(&fixed_group);

        add_switch(
            &fixed_group,
            "Show previous window",
            &settings,
            |s| s.show_prev_win_fixed,
            |s, on| s.show_prev_win_fixed = on,
        );
        add_switch(
            &fixed_group,
            "Auto vowel form",
            &settings,
            |s| s.auto_vowel_form_fixed,
            |s, on| s.auto_vowel_form_fixed = on,
        );
        add_switch(
            &fixed_group,
            "Auto chandra position",
            &settings,
            |s| s.auto_chandra_pos_fixed,
            |s, on| s.auto_chandra_pos_fixed = on,
        );
        add_switch(
            &fixed_group,
            "Old reph",
            &settings,
            |s| s.old_reph,
            |s, on| s.old_reph = on,
        );
        add_switch(
            &fixed_group,
            "Traditional kar joining",
            &settings,
            |s| s.traditional_kar_fixed,
            |s, on| s.traditional_kar_fixed = on,
        );
        add_switch(
            &fixed_group,
            "Number pad",
            &settings,
            |s| s.number_pad_fixed,
            |s, on| s.number_pad_fixed = on,
        );

        add_combo(
            &fixed_group,
            "Kar order",
            &["Modern", "Old"],
            if settings.borrow().fixed_old_kar_order {
                1
            } else {
                0
            },
            &settings,
            |s, index| s.fixed_old_kar_order = index == 1,
        );

        // ---- GNOME IBus Integration panel ----
        let ibus = PreferencesPage::new();
        ibus.set_title("GNOME IBus Integration");
        ibus.set_icon_name(Some("system-users-symbolic"));

        let ibus_group = PreferencesGroup::new();
        ibus_group.set_title("Integration");
        ibus.add(&ibus_group);

        let desktop = crate::platform_config::desktop_environment_to_string(
            crate::platform_config::detect_desktop_environment(),
        );
        let status_row = ActionRow::new();
        status_row.set_title("Integration status");
        status_row.set_subtitle(&format!("{desktop} · Not installed"));
        ibus_group.add(&status_row);

        let setup_button = Button::with_label("Set up / repair");
        setup_button.set_hexpand(true);
        setup_button.set_halign(gtk::Align::End);
        ibus_group.add(&setup_button);

        let remove_button = Button::with_label("Remove integration");
        remove_button.set_hexpand(true);
        remove_button.set_halign(gtk::Align::End);
        ibus_group.add(&remove_button);

        window.add(&general);
        window.add(&phonetic);
        window.add(&fixed);
        window.add(&ibus);

        // Wire up IBus integration buttons. Both actions only touch files here;
        // the systemd/DBus half belongs to the concrete ServiceControl backend.
        let service = Rc::new(StubService::default());
        let status_row_c = status_row.clone();
        let setup_button_c = setup_button.clone();
        let remove_button_c = remove_button.clone();
        let service_c = Rc::clone(&service);
        setup_button.connect_clicked(move |_| {
            Self::do_setup(&service_c, &status_row_c, &setup_button_c, &remove_button_c);
        });

        let status_row_c2 = status_row.clone();
        let setup_button_c2 = setup_button.clone();
        let remove_button_c2 = remove_button.clone();
        remove_button.connect_clicked(move |_| {
            Self::do_remove(&status_row_c2, &setup_button_c2, &remove_button_c2);
        });

        Self { window }
    }

    fn do_setup(
        service: &Rc<StubService>,
        status_row: &ActionRow,
        setup_button: &Button,
        remove_button: &Button,
    ) {
        let paths = match Paths::from_env() {
            Ok(p) => p,
            Err(_) => {
                status_row.set_subtitle("Failed to determine paths");
                return;
            }
        };

        let version = option_env!("SHANTI_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));
        match install_files(&paths, version) {
            Ok(()) => {
                status_row.set_subtitle("Installed");
                setup_button.set_sensitive(false);
                remove_button.set_sensitive(true);
            }
            Err(Error::Conflict(path)) => {
                status_row.set_subtitle(&format!("Conflict: {}", path.display()));
            }
            Err(Error::UnsafePath) => {
                status_row.set_subtitle("Unsafe path");
            }
            Err(e) => {
                status_row.set_subtitle(&format!("Error: {}", e));
            }
        }
        service.calls.borrow_mut().push("setup".to_string());
    }

    fn do_remove(status_row: &ActionRow, setup_button: &Button, remove_button: &Button) {
        let paths = match Paths::from_env() {
            Ok(p) => p,
            Err(_) => {
                status_row.set_subtitle("Failed to determine paths");
                return;
            }
        };

        match remove_files(&paths) {
            Ok(()) => {
                status_row.set_subtitle("Not installed");
                setup_button.set_sensitive(true);
                remove_button.set_sensitive(false);
            }
            Err(e) => {
                status_row.set_subtitle(&format!("Error: {}", e));
            }
        }
    }

    pub fn window(&self) -> &PreferencesWindow {
        &self.window
    }
}

impl Default for SettingsDialog {
    fn default() -> Self {
        Self::new()
    }
}
