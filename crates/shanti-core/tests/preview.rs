//! Preview parity tests.
//!
//! Expected values are the standard Avro Phonetic transliterations the
//! retired Qt dialog showed for the same ASCII input, so the engine-backed
//! preview stays recognisable to users. The engine itself is riti's real
//! phonetic method (the one the IBus frontend uses), not a test double.

use shanti_core::preview::transliterate;

#[test]
fn transliterates_typed_words() {
    assert_eq!(transliterate("ami"), "আমি");
    assert_eq!(transliterate("kotha"), "কথা");
    assert_eq!(transliterate("bangla"), "বাংলা");
    assert_eq!(transliterate("tui"), "তুই");
    // Avro's aspirated/retroflex digraphs are case-sensitive: "dh" is ধ,
    // "Dh" is ঢ.
    assert_eq!(transliterate("dhaka"), "ধাকা");
    assert_eq!(transliterate("Dhaka"), "ঢাকা");
}

#[test]
fn converts_digits() {
    assert_eq!(transliterate("123"), "১২৩");
}

#[test]
fn each_word_is_converted_separately() {
    assert_eq!(transliterate("ami kotha"), "আমি কথা");
}

#[test]
fn passthrough_text_is_preserved() {
    // Spaces and already-Bangla text pass through; the ASCII before them is
    // still converted.
    assert_eq!(transliterate("ami আমি"), "আমি আমি");
    assert_eq!(transliterate(""), "");
}

#[test]
fn a_reused_preview_converts_every_call_independently() {
    let mut preview = shanti_core::preview::Preview::new();
    assert_eq!(preview.transliterate("ami"), "আমি");
    assert_eq!(preview.transliterate("kotha"), "কথা");
    assert_eq!(preview.transliterate("ami"), "আমি");
}

#[test]
fn punctuation_commits_the_pending_word() {
    // Avro maps a plain period to the Bengali danda.
    assert_eq!(transliterate("ami."), "আমি।");
}
