use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use cairo;
use gdk::prelude::*;
use gtk::prelude::*;
use gtk::{DrawingArea, WidgetExt};

const K_BASE_UNIT: f64 = 56.0;
const K_BEZEL_PAD: f64 = 14.0;
const K_KEY_GAP: f64 = 3.0;
const K_ROWS: i32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
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
                ascii_bottom: Some(lo),
                key_top: Some(format!("Key_{}", up)),
                key_bottom: Some(format!("Key_{}", lo)),
            }
        };
        let sym = |bottom: &str,
                   top: &str,
                   key_bottom: &str,
                   key_top: &str,
                   width: f64| -> Key {
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

#[derive(Clone, Default)]
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

        let state_c = Rc::clone(&state);
        let keymap_c = Rc::clone(&keymap);
        area.set_draw_func(move |_area, cr, width, height| {
            Self::paint_keyboard(cr, width as f64, height as f64, &keymap_c, &state_c);
        });

        Self { area, state, keymap }
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
            if key.ends_with("_AltGr")
                && !value.is_empty()
                && {
                    let normal_key = format!("{}_Normal", key.trim_end_matches("_AltGr"));
                    keymap.get(&normal_key).map_or(true, |v| v != value)
                }
            {
                return true;
            }
        }
        false
    }

    fn glyph(&self, key_name: Option<&str>, ascii_default: Option<&str>) -> Option<String> {
        let key_name = key_name?;
        let suffix = match self.state.borrow().mode {
            Mode::Normal => "_Normal",
            Mode::AltGr => "_AltGr",
        };
        let full = format!("{}{}", key_name, suffix);
        self.keymap
            .borrow()
            .get(&full)
            .filter(|v| !v.is_empty())
            .cloned()
            .or_else(|| ascii_default.map(|s| s.to_string()))
    }

    fn paint_keyboard(
        cr: &cairo::Context,
        width: f64,
        height: f64,
        keymap: &Rc<RefCell<BTreeMap<String, String>>>,
        state: &Rc<RefCell<KeyboardState>>,
    ) {
    let state = state.borrow();
    let keymap = keymap.borrow();

    // Theme-aware color palette, mirroring the Qt light.qss / dark.qss.
    let (bezel, keycap, border, modifier_text, ascii_text, bangla_text,
         alt_gr_highlight, alt_gr_highlight_text) = if state.dark_mode {
        (
            (0.12, 0.12, 0.14),   // dark bezel
            (0.22, 0.24, 0.27),   // ~3a3f47-ish dark keycap
            (0.45, 0.48, 0.51),   // dark border
            (0.70, 0.73, 0.76),   // light modifier text
            (0.60, 0.63, 0.66),   // light ASCII text
            (0.55, 0.70, 0.90),   // light Bangla text
            (0.086, 0.639, 0.580), // alt-gr highlight
            (1.0, 1.0, 1.0),      // light highlight text
        )
    } else {
        (
            (0.227, 0.247, 0.278), // ~3a3f47 light bezel
            (0.961, 0.965, 0.969), // ~f5f6f7 light keycap
            (0.788, 0.800, 0.820), // ~c9ccd1 light border
            (0.353, 0.376, 0.412), // ~5a6069 modifier text
            (0.486, 0.510, 0.549), // ~7c828c ASCII text
            (0.106, 0.227, 0.557), // ~1b3a8f Bangla text
            (0.086, 0.639, 0.580), // ~16a394 alt-gr highlight
            (1.0, 1.0, 1.0),       // ~ffffff highlight text
        )
    };
        cr.rectangle(0.0, 0.0, width, height);
        cr.fill();

        // Area available for the keys, and the unit size that fits it while
        // keeping the keyboard's aspect ratio.
        let area_w = width - 2.0 * K_BEZEL_PAD;
        let area_h = height - 2.0 * K_BEZEL_PAD;
        let unit = f64::min(area_w / state.total_units, area_h / K_ROWS as f64);
        let board_w = unit * state.total_units;
        let board_h = unit * K_ROWS as f64;
        let origin_x = (width - board_w) / 2.0;
        let origin_y = (height - board_h) / 2.0;

        let tp = unit * 0.11; // inner text padding

        for (r, row) in state.rows.iter().enumerate() {
            let row_units: f64 = row.iter().map(|k| k.width).sum();
            let x_offset = origin_x + (board_w - row_units * unit) / 2.0;
            let y = origin_y + r as f64 * unit;

            let mut x = x_offset;
            for key in row {
                let cell_w = key.width * unit;
                let cap_x = x + K_KEY_GAP;
                let cap_y = y + K_KEY_GAP;
                let cap_w = cell_w - 2.0 * K_KEY_GAP;
                let cap_h = unit - 2.0 * K_KEY_GAP;

                let highlight = state.mode == Mode::AltGr
                    && (key.label.as_deref() == Some("Ctrl")
                        || key.label.as_deref() == Some("Alt"));

                // keycap
                if highlight {
                    cr.set_source_rgb(alt_gr_highlight.0, alt_gr_highlight.1, alt_gr_highlight.2);
                } else {
                    cr.set_source_rgb(keycap.0, keycap.1, keycap.2);
                }
                let radius = f64::max(3.0, unit * 0.12);
                cr.rectangle(cap_x, cap_y, cap_w, cap_h);
                cr.fill();

                // key border
                cr.set_source_rgb(border.0, border.1, border.2);
                cr.set_line_width(1.0);
                cr.rectangle(cap_x, cap_y, cap_w, cap_h);
                cr.stroke();

                let inner = cap_x + tp;
                let inner_y = cap_y + tp;

                if key.key_bottom.is_none() && key.key_top.is_none() {
                    if let Some(ref label) = key.label {
                        cr.set_source_rgb(modifier_text.0, modifier_text.1, modifier_text.2);
                        cr.move_to(inner, inner_y);
                        cr.show_text(label);
                    }
                    x += cell_w;
                    continue;
                }

                // ASCII legends in a narrow left column
                cr.set_source_rgb(ascii_text.0, ascii_text.1, ascii_text.2);
                if let Some(ref top) = key.ascii_top {
                    cr.move_to(inner, inner_y);
                    cr.show_text(top);
                }
                if let Some(ref bottom) = key.ascii_bottom {
                    cr.move_to(inner, inner_y + cap_h * 0.5);
                    cr.show_text(bottom);
                }

                // Bangla glyphs on the right band
                cr.set_source_rgb(bangla_text.0, bangla_text.1, bangla_text.2);
                let glyph_x = x + cell_w * 0.5;
                let glyph_y = cap_y;
                let glyph_h = cap_h;
                let bangla_top = Self::glyph_for(
                    &keymap,
                    key.key_top.as_deref(),
                    key.ascii_top.as_deref(),
                    state.mode,
                );
                let bangla_bottom = Self::glyph_for(
                    &keymap,
                    key.key_bottom.as_deref(),
                    key.ascii_bottom.as_deref(),
                    state.mode,
                );

                match (&bangla_top, &bangla_bottom) {
                    (Some(top), Some(bottom)) => {
                        cr.move_to(glyph_x, glyph_y + unit * 0.25);
                        cr.show_text(top);
                        cr.move_to(glyph_x, glyph_y + glyph_h - unit * 0.1);
                        cr.show_text(bottom);
                    }
                    (Some(top), None) => {
                        cr.move_to(glyph_x, glyph_y + glyph_h * 0.5);
                        cr.show_text(top);
                    }
                    (None, Some(bottom)) => {
                        cr.move_to(glyph_x, glyph_y + glyph_h * 0.75);
                        cr.show_text(bottom);
                    }
                    (None, None) => {}
                }

                x += cell_w;
            }
        }
    }

    fn glyph_for(
        keymap: &BTreeMap<String, String>,
        key_name: Option<&str>,
        ascii_default: Option<&str>,
        mode: Mode,
    ) -> Option<String> {
        let key_name = key_name?;
        let suffix = match mode {
            Mode::Normal => "_Normal",
            Mode::AltGr => "_AltGr",
        };
        let full = format!("{}{}", key_name, suffix);
        keymap
            .get(&full)
            .filter(|v| !v.is_empty())
            .cloned()
            .or_else(|| ascii_default.map(|s| s.to_string()))
    }
}

impl Default for KeyboardWidget {
    fn default() -> Self {
        Self::new()
    }
}
