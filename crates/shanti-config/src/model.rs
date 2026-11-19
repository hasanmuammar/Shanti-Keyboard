/*
 *  Typed settings model — Rust port of src/shared/Settings (Phase 3).
 *
 *  Defaults are copied verbatim from Settings.cpp getters. The INI key
 *  mapping is QSettings' ("Shanti", "Shanti") section/key folding, which
 *  Config already mirrors.
 */

use std::io;
use std::path::Path;

use crate::Config;

/// Typed view of the settings store shared by the GUI and the engines.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub layout_name: String,
    pub layout_path: String,

    pub enter_key_closes_prev_win: bool,
    pub show_prev_win_fixed: bool,
    pub auto_vowel_form_fixed: bool,
    pub auto_chandra_pos_fixed: bool,
    pub traditional_kar_fixed: bool,
    pub number_pad_fixed: bool,
    pub old_reph: bool,
    pub candidate_win_horizontal: bool,
    pub show_cw_phonetic: bool,
    pub suggestion_include_english: bool,
    pub fixed_old_kar_order: bool,
    pub ansi_encoding: bool,
    pub smart_quoting: bool,
    pub previous_user_data_remains: bool,
    pub top_bar_visibility: bool,

    pub tray_info_count: u32,
    pub top_bar_position: (i32, i32),
    pub layout_viewer_position: (i32, i32),
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            layout_name: "Avro Phonetic".to_string(),
            layout_path: "avro_phonetic".to_string(),
            enter_key_closes_prev_win: false,
            show_prev_win_fixed: true,
            auto_vowel_form_fixed: true,
            auto_chandra_pos_fixed: true,
            traditional_kar_fixed: false,
            number_pad_fixed: true,
            old_reph: true,
            candidate_win_horizontal: true,
            show_cw_phonetic: true,
            suggestion_include_english: true,
            fixed_old_kar_order: false,
            ansi_encoding: false,
            smart_quoting: true,
            previous_user_data_remains: true,
            top_bar_visibility: true,
            tray_info_count: 0,
            top_bar_position: (0, 0),
            layout_viewer_position: (0, 0),
        }
    }
}

/// QSettings serializes QPoint as `@Point(x y)`; an absent/invalid value
/// becomes a default-constructed QPoint(0, 0).
fn point(config: &Config, key: &str) -> (i32, i32) {
    let raw = config.get_string(key, "");
    let inner = raw
        .strip_prefix("@Point(")
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or("");
    let mut parts = inner.split_whitespace();
    match (parts.next(), parts.next(), parts.next()) {
        (Some(x), Some(y), None) => match (x.parse(), y.parse()) {
            (Ok(x), Ok(y)) => (x, y),
            _ => (0, 0),
        },
        _ => (0, 0),
    }
}

impl Settings {
    /// Load from an explicit path (missing file = all defaults).
    pub fn load_from(path: &Path) -> io::Result<Self> {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(e),
        };
        Ok(Self::from_ini(&text))
    }

    /// Load from the active store (`$XDG_CONFIG_HOME/Shanti/Shanti.conf`).
    pub fn load() -> io::Result<Self> {
        Self::load_from(&shanti_paths::settings_file())
    }

    /// Write to an explicit path, creating its parent directory.
    pub fn save_to(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, self.to_ini())
    }

    /// Write to the active store.
    pub fn save(&self) -> io::Result<()> {
        self.save_to(&shanti_paths::settings_file())
    }

    /// Build the typed model from QSettings INI text. Missing keys keep the
    /// exact Settings.cpp defaults.
    pub fn from_ini(text: &str) -> Self {
        let config = Config::parse(text);
        let d = Self::default();
        Self {
            layout_name: config.get_string("layout/name", &d.layout_name),
            layout_path: config.get_string("layout/path", &d.layout_path),
            enter_key_closes_prev_win: config.get_bool(
                "settings/EnterKeyClosesPrevWin",
                d.enter_key_closes_prev_win,
            ),
            show_prev_win_fixed: config
                .get_bool("settings/FixedLayout/ShowPrevWin", d.show_prev_win_fixed),
            auto_vowel_form_fixed: config.get_bool(
                "settings/FixedLayout/AutoVowelForm",
                d.auto_vowel_form_fixed,
            ),
            auto_chandra_pos_fixed: config.get_bool(
                "settings/FixedLayout/AutoChandraPos",
                d.auto_chandra_pos_fixed,
            ),
            traditional_kar_fixed: config.get_bool(
                "settings/FixedLayout/TraditionalKar",
                d.traditional_kar_fixed,
            ),
            number_pad_fixed: config.get_bool("settings/FixedLayout/NumberPad", d.number_pad_fixed),
            old_reph: config.get_bool("settings/FixedLayout/OldReph", d.old_reph),
            candidate_win_horizontal: config.get_bool(
                "settings/CandidateWin/Horizontal",
                d.candidate_win_horizontal,
            ),
            show_cw_phonetic: config.get_bool("settings/CandidateWin/Phonetic", d.show_cw_phonetic),
            suggestion_include_english: config.get_bool(
                "settings/PreviewWin/IncludeEnglish",
                d.suggestion_include_english,
            ),
            fixed_old_kar_order: config
                .get_bool("settings/FixedLayout/OldKarOrder", d.fixed_old_kar_order),
            ansi_encoding: config.get_bool("settings/ANSI", d.ansi_encoding),
            smart_quoting: config.get_bool("settings/SmartQuoting", d.smart_quoting),
            previous_user_data_remains: config.get_bool(
                "settings/PreviousUserDataRemains",
                d.previous_user_data_remains,
            ),
            top_bar_visibility: config.get_bool("window/visibility/TopBar", d.top_bar_visibility),
            tray_info_count: config.get_uint("settings/TrayInfoCount", d.tray_info_count),
            top_bar_position: point(&config, "window/positions/TopBar"),
            layout_viewer_position: point(&config, "window/positions/LayoutViewer"),
        }
    }

    /// Serialize in the exact section/key order QSettings writes. Booleans
    /// are bare `true`/`false`, integers bare decimals, strings unquoted and
    /// QPoints `@Point(x y)`. Writing all keys (including defaults) keeps
    /// the store self-describing and byte-comparable to a Qt-written file.
    pub fn to_ini(&self) -> String {
        format!(
            "[layout]\n\
             name={}\n\
             path={}\n\
             \n\
             [settings]\n\
             ANSI={}\n\
             CandidateWin\\Horizontal={}\n\
             CandidateWin\\Phonetic={}\n\
             EnterKeyClosesPrevWin={}\n\
             FixedLayout\\AutoChandraPos={}\n\
             FixedLayout\\AutoVowelForm={}\n\
             FixedLayout\\NumberPad={}\n\
             FixedLayout\\OldKarOrder={}\n\
             FixedLayout\\OldReph={}\n\
             FixedLayout\\ShowPrevWin={}\n\
             FixedLayout\\TraditionalKar={}\n\
             PreviewWin\\IncludeEnglish={}\n\
             PreviousUserDataRemains={}\n\
             SmartQuoting={}\n\
             TrayInfoCount={}\n\
             \n\
             [window]\n\
             positions\\LayoutViewer=@Point({} {})\n\
             positions\\TopBar=@Point({} {})\n\
             visibility\\TopBar={}\n",
            self.layout_name,
            self.layout_path,
            self.ansi_encoding,
            self.candidate_win_horizontal,
            self.show_cw_phonetic,
            self.enter_key_closes_prev_win,
            self.auto_chandra_pos_fixed,
            self.auto_vowel_form_fixed,
            self.number_pad_fixed,
            self.fixed_old_kar_order,
            self.old_reph,
            self.show_prev_win_fixed,
            self.traditional_kar_fixed,
            self.suggestion_include_english,
            self.previous_user_data_remains,
            self.smart_quoting,
            self.tray_info_count,
            self.layout_viewer_position.0,
            self.layout_viewer_position.1,
            self.top_bar_position.0,
            self.top_bar_position.1,
            self.top_bar_visibility,
        )
    }
}
