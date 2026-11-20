use std::rc::Rc;

use crate::auto_correct_dialog::AutoCorrectDialog;
use adw::prelude::*;
use adw::{
    ActionRow, ComboRow, PreferencesGroup, PreferencesPage, PreferencesWindow, ToggleSwitch,
};
use gtk::{prelude::*, Button};
use shanti_config::Settings;

use shanti_desktop::{install_files, remove_files, Error, Paths};

/// A stub `ServiceControl` used until the GTK panel wires up a real
/// DBus/systemd backend. It records calls so tests can assert on them.
#[derive(Default)]
pub struct StubService {
    pub calls: std::cell::RefCell<Vec<String>>,
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

pub struct SettingsDialog {
    pub window: PreferencesWindow,
    settings: Rc<std::cell::RefCell<Settings>>,
    service: Rc<StubService>,
    status_row: ActionRow,
    setup_button: Button,
    remove_button: Button,
}

impl SettingsDialog {
    pub fn new() -> Self {
        let settings = Rc::new(std::cell::RefCell::new(
            Settings::load().unwrap_or_default(),
        ));
        let service = Rc::new(StubService::default());

        let window = PreferencesWindow::new();
        window.set_title("Shanti Keyboard Settings");
        window.set_default_width(640);
        window.set_default_height(480);

        // ---- General page ----
        let general = PreferencesPage::new();
        general.set_title("General");
        general.set_icon_name("preferences-system-symbolic");

        let general_group = PreferencesGroup::new();
        general_group.set_title("General");
        general.add(&general_group);

        let enter_close = ActionRow::new();
        enter_close.set_title("Enter key closes previous window");
        let enter_close_sw = ToggleSwitch::new();
        enter_close_sw.set_active(settings.borrow().enter_key_closes_prev_win);
        enter_close.add_suffix(&enter_close_sw);
        general_group.add(&enter_close);

        let smart_quote = ActionRow::new();
        smart_quote.set_title("Smart quoting");
        let smart_quote_sw = ToggleSwitch::new();
        smart_quote_sw.set_active(settings.borrow().smart_quoting);
        smart_quote.add_suffix(&smart_quote_sw);
        general_group.add(&smart_quote);

        let orientation = ComboRow::new();
        orientation.set_title("Candidate window orientation");
        general_group.add(&orientation);

        let include_english = ActionRow::new();
        include_english.set_title("Include English in candidate window");
        let include_english_sw = ToggleSwitch::new();
        include_english_sw.set_active(settings.borrow().suggestion_include_english);
        include_english.add_suffix(&include_english_sw);
        general_group.add(&include_english);

        let ansi = ActionRow::new();
        ansi.set_title("ANSI encoding");
        let ansi_sw = ToggleSwitch::new();
        ansi_sw.set_active(settings.borrow().ansi_encoding);
        ansi.add_suffix(&ansi_sw);
        general_group.add(&ansi);

        // ---- Phonetic page ----
        let phonetic = PreferencesPage::new();
        phonetic.set_title("Phonetic");
        phonetic.set_icon_name(Some("accessories-character-map-symbolic"));

        let phonetic_group = PreferencesGroup::new();
        phonetic_group.set_title("Phonetic keyboard layout");
        phonetic.add(&phonetic_group);

        let show_cw = ActionRow::new();
        show_cw.set_title("Show candidate window");
        let show_cw_sw = ToggleSwitch::new();
        show_cw_sw.set_active(settings.borrow().show_cw_phonetic);
        show_cw.add_suffix(&show_cw_sw);
        phonetic_group.add(&show_cw);

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

        let show_prev = ActionRow::new();
        show_prev.set_title("Show previous window");
        let show_prev_sw = ToggleSwitch::new();
        show_prev_sw.set_active(settings.borrow().show_prev_win_fixed);
        show_prev.add_suffix(&show_prev_sw);
        fixed_group.add(&show_prev);

        let auto_vowel = ActionRow::new();
        auto_vowel.set_title("Auto vowel form");
        let auto_vowel_sw = ToggleSwitch::new();
        auto_vowel_sw.set_active(settings.borrow().auto_vowel_form_fixed);
        auto_vowel.add_suffix(&auto_vowel_sw);
        fixed_group.add(&auto_vowel);

        let auto_chandra = ActionRow::new();
        auto_chandra.set_title("Auto chandra position");
        let auto_chandra_sw = ToggleSwitch::new();
        auto_chandra_sw.set_active(settings.borrow().auto_chandra_pos_fixed);
        auto_chandra.add_suffix(&auto_chandra_sw);
        fixed_group.add(&auto_chandra);

        let old_reph = ActionRow::new();
        old_reph.set_title("Old reph");
        let old_reph_sw = ToggleSwitch::new();
        old_reph_sw.set_active(settings.borrow().old_reph);
        old_reph.add_suffix(&old_reph_sw);
        fixed_group.add(&old_reph);

        let kar_joining = ActionRow::new();
        kar_joining.set_title("Traditional kar joining");
        let kar_joining_sw = ToggleSwitch::new();
        kar_joining_sw.set_active(settings.borrow().traditional_kar_fixed);
        kar_joining.add_suffix(&kar_joining_sw);
        fixed_group.add(&kar_joining);

        let number_pad = ActionRow::new();
        number_pad.set_title("Number pad");
        let number_pad_sw = ToggleSwitch::new();
        number_pad_sw.set_active(settings.borrow().number_pad_fixed);
        number_pad.add_suffix(&number_pad_sw);
        fixed_group.add(&number_pad);

        let old_kar_order = ComboRow::new();
        old_kar_order.set_title("Kar order");
        fixed_group.add(&old_kar_order);

        // ---- GNOME IBus Integration panel ----
        let ibus = PreferencesPage::new();
        ibus.set_title("GNOME IBus Integration");
        ibus.set_icon_name(Some("system-users-symbolic"));

        let ibus_group = PreferencesGroup::new();
        ibus_group.set_title("Integration");
        ibus.add(&ibus_group);

        let status_row = ActionRow::new();
        status_row.set_title("Integration status");
        status_row.set_subtitle("Not installed");
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

        // Wire up IBus integration buttons
        let settings_c = Rc::clone(&settings);
        let service_c = Rc::clone(&service);
        let status_row_c = status_row.clone();
        let setup_button_c = setup_button.clone();
        let remove_button_c = remove_button.clone();
        setup_button.connect_clicked(move |_| {
            Self::do_setup(
                &settings_c,
                &service_c,
                &status_row_c,
                &setup_button_c,
                &remove_button_c,
            );
        });

        let settings_c2 = Rc::clone(&settings);
        let service_c2 = Rc::clone(&service);
        let status_row_c2 = status_row.clone();
        let setup_button_c2 = setup_button.clone();
        let remove_button_c2 = remove_button.clone();
        remove_button.connect_clicked(move |_| {
            Self::do_remove(
                &settings_c2,
                &service_c2,
                &status_row_c2,
                &setup_button_c2,
                &remove_button_c2,
            );
        });

        Self {
            window,
            settings,
            service,
            status_row,
            setup_button,
            remove_button,
        }
    }

    fn do_setup(
        _settings: &Rc<std::cell::RefCell<Settings>>,
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

    fn do_remove(
        settings: &Rc<std::cell::RefCell<Settings>>,
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
        service.calls.borrow_mut().push("remove".to_string());
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
