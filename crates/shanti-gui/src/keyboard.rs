use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::DrawingArea;

const K_BASE_UNIT: f64 = 56.0;
const K_BEZEL_PAD: f64 = 14.0;
const K_KEY_GAP: f64 = 3.0;
const K_ROWS: i32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    AltGr,
}

struct Key {
    width: f64,
    label: Option<String>,
    ascii_top: Option<String>,
    ascii_bottom: Option<String>,
    key_top: Option<String>,
    key_bottom: Option<String>,
}

struct KeyboardState {
    rows: Vec<Vec<Key>>,
    mode: Mode,
    total_units: f64,
    dark_mode: bool,
}

impl KeyboardState {
    fn build_rows() -> Vec<Vec<Key>> {
        let letter = |c: char| -> Key {
            let up = c.to_uppercase().to_string();
            let lo = c.to_lowercase().to_string();
            Key {
                width: 1.0,
                label: None,
                ascii_top: Some(up.clone()),
                ascii_bottom: Some(lo.clone()),
                key_top: Some(format!("Key_{}", up)),
                key_bottom: Some(format!("Key_{}", lo)),
            }
        };
        let sym = |bottom: &str, top: &str, key_bottom: &str, key_top: &str, width: f64| -> Key {
            Key {
                width,
                label: None,
                ascii_top: Some(top.to_string()),
                ascii_bottom: Some(bottom.to_string()),
                key_top: Some(key_top.to_string()),
                key_bottom: Some(key_bottom.to_string()),
            }
        };
        let mod_key = |width: f64, label: &str| -> Key {
            Key {
                width,
                label: Some(label.to_string()),
                ascii_top: None,
                ascii_bottom: None,
                key_top: None,
                key_bottom: None,
            }
        };

        let mut rows: Vec<Vec<Key>> = Vec::new();

        rows.push(vec![
            sym("`", "~", "Key_Grave", "Key_Tilde", 1.0),
            sym("1", "!", "Key_1", "Key_Exclaim", 1.0),
            sym("2", "@", "Key_2", "Key_At", 1.0),
            sym("3", "#", "Key_3", "Key_Hash", 1.0),
            sym("4", "$", "Key_4", "Key_Dollar", 1.0),
            sym("5", "%", "Key_5", "Key_Percent", 1.0),
            sym("6", "^", "Key_6", "Key_Circum", 1.0),
            sym("7", "&", "Key_7", "Key_Ampersand", 1.0),
            sym("8", "*", "Key_8", "Key_Asterisk", 1.0),
            sym("9", "(", "Key_9", "Key_ParenLeft", 1.0),
            sym("0", ")", "Key_0", "Key_ParenRight", 1.0),
            sym("-", "_", "Key_Minus", "Key_UnderScore", 1.0),
            sym("=", "+", "Key_Equals", "Key_Plus", 1.0),
            mod_key(2.0, "Backspace"),
        ]);

        rows.push(vec![
            mod_key(1.5, "Tab"),
            letter('Q'),
            letter('W'),
            letter('E'),
            letter('R'),
            letter('T'),
            letter('Y'),
            letter('U'),
            letter('I'),
            letter('O'),
            letter('P'),
            sym("[", "{", "Key_BracketLeft", "Key_BraceLeft", 1.0),
            sym("]", "}", "Key_BracketRight", "Key_BraceRight", 1.0),
            sym("\\", "|", "Key_BackSlash", "Key_Bar", 1.5),
        ]);

        rows.push(vec![
            mod_key(1.75, "Caps Lock"),
            letter('A'),
            letter('S'),
            letter('D'),
            letter('F'),
            letter('G'),
            letter('H'),
            letter('J'),
            letter('K'),
            letter('L'),
            sym(";", ":", "Key_Semicolon", "Key_Colon", 1.0),
            sym("'", "\"", "Key_Apostrophe", "Key_Quote", 1.0),
            mod_key(2.25, "Enter"),
        ]);

        rows.push(vec![
            mod_key(2.25, "Shift"),
            letter('Z'),
            letter('X'),
            letter('C'),
            letter('V'),
            letter('B'),
            letter('N'),
            letter('M'),
            sym(",", "<", "Key_Comma", "Key_Less", 1.0),
            sym(".", ">", "Key_Period", "Key_Greater", 1.0),
            sym("/", "?", "Key_Slash", "Key_Question", 1.0),
            mod_key(2.75, "Shift"),
        ]);

        rows.push(vec![
            mod_key(1.25, "Ctrl"),
            mod_key(1.25, "Win"),
            mod_key(1.25, "Alt"),
            mod_key(6.25, ""),
            mod_key(1.25, "Alt"),
            mod_key(1.25, "Menu"),
            mod_key(1.25, "Ctrl"),
        ]);

        rows
    }

    fn new() -> Self {
        let rows = Self::build_rows();
        let mut total_units = 0.0;
        for row in &rows {
            let row_units: f64 = row.iter().map(|k| k.width).sum();
            if row_units > total_units {
                total_units = row_units;
            }
        }
        let dark_mode = gtk::Settings::default()
            .map(|s| s.is_gtk_application_prefer_dark_theme())
            .unwrap_or(false);
        Self {
            rows,
            mode: Mode::Normal,
            total_units,
            dark_mode,
        }
    }

    fn mode(&self) -> Mode {
        self.mode
    }
}

pub struct KeyboardWidget {
    area: DrawingArea,
    state: Rc<RefCell<KeyboardState>>,
    keymap: Rc<RefCell<BTreeMap<String, String>>>,
}

impl KeyboardWidget {
    pub fn new() -> Self {
        let state = Rc::new(RefCell::new(KeyboardState::new()));
        let keymap = Rc::new(RefCell::new(BTreeMap::new()));
        let area = DrawingArea::new();
        area.set_hexpand(true);
        area.set_vexpand(true);

        Self {
            area,
            state,
            keymap,
        }
    }

    pub fn widget(&self) -> &DrawingArea {
        &self.area
    }

    pub fn set_keymap(&self, keymap: BTreeMap<String, String>) {
        *self.keymap.borrow_mut() = keymap;
        self.area.queue_draw();
    }

    pub fn set_mode(&self, mode: Mode) {
        self.state.borrow_mut().mode = mode;
        self.area.queue_draw();
    }

    pub fn mode(&self) -> Mode {
        self.state.borrow().mode
    }

    pub fn has_altgr(&self) -> bool {
        let keymap = self.keymap.borrow();
        for (key, value) in keymap.iter() {
            if key.ends_with("_AltGr") && !value.is_empty() && {
                let normal_key = format!("{}_Normal", key.trim_end_matches("_AltGr"));
                keymap.get(&normal_key).map_or(true, |v| v != value)
            } {
                return true;
            }
        }
        false
    }
}

impl Default for KeyboardWidget {
    fn default() -> Self {
        Self::new()
    }
}
