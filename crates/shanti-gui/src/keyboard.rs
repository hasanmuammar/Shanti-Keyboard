use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{cairo, pango, DrawingArea};
use pangocairo::functions::show_layout;

const K_BASE_UNIT: f64 = 56.0;
const K_BEZEL_PAD: f64 = 14.0;
const K_KEY_GAP: f64 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    AltGr,
}

/// Palette from the retired Qt `KeyboardWidget` defaults, so the GTK viewer
/// keeps the same look on both light and dark shells.
const BEZEL: (f64, f64, f64) = rgb(0x3a3f47);
const KEYCAP: (f64, f64, f64) = rgb(0xf5f6f7);
const KEY_BORDER: (f64, f64, f64) = rgb(0xc9ccd1);
const ASCII_TEXT: (f64, f64, f64) = rgb(0x7c828c);
const BANGLA_TEXT: (f64, f64, f64) = rgb(0x1b3a8f);
const MODIFIER_TEXT: (f64, f64, f64) = rgb(0x5a6069);
const ALTGR_HIGHLIGHT: (f64, f64, f64) = rgb(0x16a394);
const ALTGR_HIGHLIGHT_TEXT: (f64, f64, f64) = rgb(0xffffff);

const fn rgb(hex: u32) -> (f64, f64, f64) {
    (
        ((hex >> 16) & 0xff) as f64 / 255.0,
        ((hex >> 8) & 0xff) as f64 / 255.0,
        (hex & 0xff) as f64 / 255.0,
    )
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
                ascii_bottom: None,
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
        Self {
            rows,
            mode: Mode::Normal,
            total_units,
        }
    }

    fn suffix(&self) -> &'static str {
        match self.mode {
            Mode::Normal => "_Normal",
            Mode::AltGr => "_AltGr",
        }
    }

    /// Look up `keyName` + the current layer suffix, mirroring the Qt
    /// `KeyboardWidget::glyph()` helper.
    fn glyph(&self, key_name: Option<&String>, map: &BTreeMap<String, String>) -> String {
        match key_name {
            Some(name) if !name.is_empty() => map
                .get(&format!("{}{}", name, self.suffix()))
                .cloned()
                .unwrap_or_default(),
            _ => String::new(),
        }
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

        // Natural size follows the Qt widget: the minimum is 60% of the base
        // unit, and the drawing scales to whatever the container gives us.
        {
            let state = state.borrow();
            let min_unit = K_BASE_UNIT * 0.6;
            let width = state.total_units * min_unit + 2.0 * K_BEZEL_PAD;
            let height = state.rows.len() as f64 * min_unit + 2.0 * K_BEZEL_PAD;
            area.set_content_width(width.round() as i32);
            area.set_content_height(height.round() as i32);
        }

        let draw_state = Rc::clone(&state);
        let draw_keymap = Rc::clone(&keymap);
        area.set_draw_func(move |area, cr, width, height| {
            draw_keyboard(
                area,
                cr,
                width,
                height,
                &draw_state.borrow(),
                &draw_keymap.borrow(),
            );
        });

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

#[derive(Clone, Copy)]
enum HAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy)]
enum VAlign {
    Top,
    Center,
    Bottom,
}

fn draw_keyboard(
    area: &DrawingArea,
    cr: &cairo::Context,
    width: i32,
    height: i32,
    state: &KeyboardState,
    keymap: &BTreeMap<String, String>,
) {
    let width = f64::from(width);
    let height = f64::from(height);
    if width <= 0.0 || height <= 0.0 {
        return;
    }

    // Bezel fills the whole widget with rounded corners.
    cr.set_source_rgb(BEZEL.0, BEZEL.1, BEZEL.2);
    rounded_rect(cr, 0.0, 0.0, width, height, 14.0);
    cr.fill().ok();

    let avail_w = width - 2.0 * K_BEZEL_PAD;
    let avail_h = height - 2.0 * K_BEZEL_PAD;
    if avail_w <= 0.0 || avail_h <= 0.0 {
        return;
    }

    let row_count = state.rows.len() as f64;
    let unit = (avail_w / state.total_units).min(avail_h / row_count);
    if unit <= 0.0 {
        return;
    }

    let board_w = unit * state.total_units;
    let board_h = unit * row_count;
    let origin_x = K_BEZEL_PAD + (avail_w - board_w) / 2.0;
    let origin_y = K_BEZEL_PAD + (avail_h - board_h) / 2.0;

    let ascii_px = (unit * 0.24).max(7.0);
    let bangla_px = (unit * 0.34).max(9.0);
    let mod_px = (unit * 0.22).max(7.0);
    let text_pad = unit * 0.11;

    for (row_index, row) in state.rows.iter().enumerate() {
        let row_units: f64 = row.iter().map(|key| key.width).sum();
        // Center rows narrower than the widest row (the modifier row).
        let mut x = origin_x + (board_w - row_units * unit) / 2.0;
        let y = origin_y + row_index as f64 * unit;

        for key in row {
            let cell_w = key.width * unit;
            let cap = (
                x + K_KEY_GAP,
                y + K_KEY_GAP,
                cell_w - 2.0 * K_KEY_GAP,
                unit - 2.0 * K_KEY_GAP,
            );
            x += cell_w;
            draw_cap(
                area, cr, key, state, keymap, cap, unit, ascii_px, bangla_px, mod_px, text_pad,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_cap(
    area: &DrawingArea,
    cr: &cairo::Context,
    key: &Key,
    state: &KeyboardState,
    keymap: &BTreeMap<String, String>,
    cap: (f64, f64, f64, f64),
    unit: f64,
    ascii_px: f64,
    bangla_px: f64,
    mod_px: f64,
    text_pad: f64,
) {
    let (cap_x, cap_y, cap_w, cap_h) = cap;
    let highlight = state.mode == Mode::AltGr && is_altgr_trigger(key.label.as_deref());
    let radius = (unit * 0.12).max(3.0);

    let fill = if highlight { ALTGR_HIGHLIGHT } else { KEYCAP };
    cr.set_source_rgb(fill.0, fill.1, fill.2);
    rounded_rect(cr, cap_x, cap_y, cap_w, cap_h, radius);
    cr.fill().ok();

    cr.set_source_rgb(KEY_BORDER.0, KEY_BORDER.1, KEY_BORDER.2);
    cr.set_line_width(1.0);
    rounded_rect(cr, cap_x, cap_y, cap_w, cap_h, radius);
    cr.stroke().ok();

    let inner = (
        cap_x + text_pad,
        cap_y + text_pad,
        (cap_w - 2.0 * text_pad).max(1.0),
        (cap_h - 2.0 * text_pad).max(1.0),
    );

    let is_modifier = key.key_bottom.is_none() && key.key_top.is_none();
    if is_modifier {
        if let Some(label) = &key.label {
            let color = if highlight {
                ALTGR_HIGHLIGHT_TEXT
            } else {
                MODIFIER_TEXT
            };
            draw_text(
                area,
                cr,
                label,
                mod_px,
                inner,
                HAlign::Left,
                VAlign::Top,
                color,
                None,
            );
        }
        return;
    }

    // ASCII legends live in a narrow left column; the Bangla glyphs get the
    // full inner height on the right so tall matras/conjuncts aren't clipped.
    let top_left = (inner.0, inner.1, inner.2 * 0.45, inner.3 * 0.5);
    let bottom_left = (
        inner.0,
        inner.1 + inner.3 * 0.5,
        inner.2 * 0.45,
        inner.3 * 0.5,
    );
    let right_band = (inner.0 + inner.2 * 0.5, inner.1, inner.2 * 0.5, inner.3);
    let glyph_pad = unit * 0.03;
    let glyph_band = (
        inner.0 + inner.2 * 0.5,
        cap_y + glyph_pad,
        inner.2 * 0.5,
        cap_h - 2.0 * glyph_pad,
    );

    if let Some(text) = &key.ascii_top {
        draw_text(
            area,
            cr,
            text,
            ascii_px,
            top_left,
            HAlign::Left,
            VAlign::Top,
            ASCII_TEXT,
            None,
        );
    }
    if let Some(text) = &key.ascii_bottom {
        draw_text(
            area,
            cr,
            text,
            ascii_px,
            bottom_left,
            HAlign::Left,
            VAlign::Bottom,
            ASCII_TEXT,
            None,
        );
    }

    let bangla_top = state.glyph(key.key_top.as_ref(), keymap);
    let bangla_bottom = state.glyph(key.key_bottom.as_ref(), keymap);
    let ascii_top = key.ascii_top.as_deref().unwrap_or("");
    let ascii_bottom = key.ascii_bottom.as_deref().unwrap_or("");
    let show_top = !bangla_top.is_empty() && bangla_top != ascii_top;
    let show_bottom =
        !bangla_bottom.is_empty() && bangla_bottom != ascii_bottom && bangla_bottom != ascii_top;

    if show_top && show_bottom {
        // Two layers: shifted anchored to the top, normal to the bottom.
        draw_text(
            area,
            cr,
            &bangla_top,
            bangla_px,
            glyph_band,
            HAlign::Right,
            VAlign::Top,
            BANGLA_TEXT,
            Some("Kalpurush"),
        );
        draw_text(
            area,
            cr,
            &bangla_bottom,
            bangla_px,
            glyph_band,
            HAlign::Right,
            VAlign::Bottom,
            BANGLA_TEXT,
            Some("Kalpurush"),
        );
    } else if show_top {
        draw_text(
            area,
            cr,
            &bangla_top,
            bangla_px,
            right_band,
            HAlign::Right,
            VAlign::Center,
            BANGLA_TEXT,
            Some("Kalpurush"),
        );
    } else if show_bottom {
        draw_text(
            area,
            cr,
            &bangla_bottom,
            bangla_px,
            glyph_band,
            HAlign::Right,
            VAlign::Bottom,
            BANGLA_TEXT,
            Some("Kalpurush"),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_text(
    area: &DrawingArea,
    cr: &cairo::Context,
    text: &str,
    pixel_size: f64,
    rect: (f64, f64, f64, f64),
    h_align: HAlign,
    v_align: VAlign,
    color: (f64, f64, f64),
    family: Option<&str>,
) {
    if text.is_empty() {
        return;
    }

    let layout = area.create_pango_layout(Some(text));
    let mut description = pango::FontDescription::from_string(family.unwrap_or("Sans"));
    description.set_absolute_size(pixel_size * f64::from(pango::SCALE));
    layout.set_font_description(Some(&description));

    let (text_w, text_h) = layout.pixel_size();
    let (x, y, w, h) = rect;
    let text_x = match h_align {
        HAlign::Left => x,
        HAlign::Center => x + (w - f64::from(text_w)) / 2.0,
        HAlign::Right => x + w - f64::from(text_w),
    };
    let text_y = match v_align {
        VAlign::Top => y,
        VAlign::Center => y + (h - f64::from(text_h)) / 2.0,
        VAlign::Bottom => y + h - f64::from(text_h),
    };

    cr.set_source_rgb(color.0, color.1, color.2);
    cr.move_to(text_x, text_y);
    show_layout(cr, &layout);
}

fn is_altgr_trigger(label: Option<&str>) -> bool {
    // Linux right Alt (AltGr) and Windows Ctrl+Alt both use these caps.
    matches!(label, Some("Ctrl") | Some("Alt"))
}

fn rounded_rect(cr: &cairo::Context, x: f64, y: f64, width: f64, height: f64, radius: f64) {
    use std::f64::consts::FRAC_PI_2;
    use std::f64::consts::PI;

    let radius = radius.min(width / 2.0).min(height / 2.0);
    cr.new_sub_path();
    cr.arc(x + width - radius, y + radius, radius, -FRAC_PI_2, 0.0);
    cr.arc(
        x + width - radius,
        y + height - radius,
        radius,
        0.0,
        FRAC_PI_2,
    );
    cr.arc(x + radius, y + height - radius, radius, FRAC_PI_2, PI);
    cr.arc(x + radius, y + radius, radius, PI, 1.5 * PI);
    cr.close_path();
}
