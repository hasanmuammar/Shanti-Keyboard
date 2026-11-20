//! Autocorrect preview through the real engine.
//!
//! The retired Qt `AutoCorrectDialog` previewed entries with the
//! frontend-internal `AvroPhonetic` parser. That parser is gone; the GTK
//! dialog previews through the same Rust engine the IBus frontend uses, so a
//! preview can never disagree with what typing produces.
//!
//! The preview store pins the phonetic layout and turns suggestions off: the
//! dialog shows the pure transliteration of what the user typed, not a ranked
//! dictionary word.

use crate::engine::ShantiCore;
use shanti_config::Config as Store;

/// Phonetic layout with suggestions off (`settings/CandidateWin/Phonetic`).
const PREVIEW_INI: &str =
    "[layout]\npath=avro_phonetic\n\n[settings]\nCandidateWin\\Phonetic=false\n";

fn preview_core() -> ShantiCore {
    ShantiCore::with_settings_store(Some(Store::parse(PREVIEW_INI)))
}

/// Reusable preview engine. Building a `ShantiCore` loads the engine data, so
/// the dialog keeps one for its lifetime instead of rebuilding it on every
/// keystroke.
pub struct Preview {
    core: ShantiCore,
}

impl Preview {
    pub fn new() -> Self {
        Self {
            core: preview_core(),
        }
    }

    /// Transliterate `input` the way the engine converts typed keys.
    pub fn transliterate(&mut self, input: &str) -> String {
        transliterate_with(&mut self.core, input)
    }
}

impl Default for Preview {
    fn default() -> Self {
        Self::new()
    }
}

/// One-shot transliteration; convenience for tests and one-off previews.
pub fn transliterate(input: &str) -> String {
    transliterate_with(&mut preview_core(), input)
}

/// ASCII graphic characters are fed as key events (X keysyms equal their ASCII
/// codes for this range) and converted. Everything else — spaces, punctuation
/// the engine does not own, and already-Bangla text — commits the pending
/// word and passes through unchanged, matching the engine's pass-through
/// behaviour for keys it does not consume.
fn transliterate_with(core: &mut ShantiCore, input: &str) -> String {
    let mut out = String::with_capacity(input.len());

    for ch in input.chars() {
        if ch.is_ascii_graphic() {
            core.key_event(ch as u32, 0);
        } else {
            out.push_str(&core.commit_text());
            out.push(ch);
        }
    }
    out.push_str(&core.commit_text());
    out
}
