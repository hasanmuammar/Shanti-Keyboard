/*
 *  Layout-file parity against the shipped layouts under `data/` and the Qt
 *  `Layout`/`LayoutViewer` parsing rules (src/frontend/Layout.cpp,
 *  LayoutViewer::decodeAndDecompress). Expected values were read from the
 *  shipped JSON inputs and the C++ source, never from the code under test.
 */

use shanti_layout::{decode_image, LayoutFile, LayoutKind};

const PROBHAT: &str = include_str!("../../../data/Probhat.json");
const AVRO_EASY: &str = include_str!("../../../data/Avro_Easy.json");
const AVRO_PHONETIC: &str = include_str!("../../../data/avrophonetic.json");
const KHIPRO: &str = include_str!("../../../data/khipro.json");

#[test]
fn fixed_layout_description_matches_the_qt_parser() {
    let layout = LayoutFile::parse(PROBHAT).unwrap();
    let desc = layout.desc();

    assert_eq!(desc.kind, LayoutKind::Fixed);
    assert_eq!(desc.name, "Probhat");
    // info.layout.version is a JSON string; Qt reads it with toString().
    assert_eq!(desc.ver, "1.0");
    // info.version is the JSON *string* "3" here, and QJsonValue::toInt() does
    // not convert strings — Qt records 0. fileVer is dead data in the frontend.
    assert_eq!(desc.file_ver, 0);
    assert_eq!(desc.dev_name, "Shabab Mustafa ");
    assert_eq!(
        desc.dev_comment,
        "One of the popular keyboard layouts.\n\nThis layout is imported from Avro Keyboard.\n"
    );
    assert!(desc.image.is_empty());
}

#[test]
fn phonetic_layout_is_typed_by_type_and_name() {
    let phonetic = LayoutFile::parse(AVRO_PHONETIC).unwrap();
    assert_eq!(phonetic.desc().kind, LayoutKind::Phonetic);
    assert_eq!(phonetic.desc().name, "Avro Phonetic");
    assert_eq!(phonetic.desc().ver, "4");
    assert_eq!(phonetic.desc().dev_name, "Mehdi Hasan (OmicronLab)");
    assert!(!phonetic.desc().image.is_empty());

    // Khipro is the transliteration layout carrying the reserved name.
    let khipro = LayoutFile::parse(KHIPRO).unwrap();
    assert_eq!(khipro.desc().kind, LayoutKind::Khipro);
    assert_eq!(khipro.desc().name, "Khipro");
    // khipro.json stores info.version as a real JSON number, so toInt() sees 3.
    assert_eq!(khipro.desc().file_ver, 3);
    assert!(!khipro.desc().image.is_empty());
}

#[test]
fn fixed_layout_exposes_the_full_two_hundred_three_entry_key_map() {
    let layout = LayoutFile::parse(PROBHAT).unwrap();
    let map = layout.keymap();

    assert_eq!(map.len(), 203);
    assert_eq!(map.get("Key_1_Normal").map(String::as_str), Some("১"));
    assert_eq!(map.get("Key_1_AltGr").map(String::as_str), Some("৴"));
    // Qt's getLayoutMap() returns an empty map for every non-fixed layout.
    assert_eq!(LayoutFile::parse(AVRO_PHONETIC).unwrap().keymap().len(), 0);
    assert_eq!(LayoutFile::parse(KHIPRO).unwrap().keymap().len(), 0);
}

#[test]
fn empty_alt_gr_glyphs_stay_empty_strings() {
    let layout = LayoutFile::parse(AVRO_EASY).unwrap();
    let map = layout.keymap();
    assert_eq!(map.len(), 203);
    assert_eq!(map.get("Key_1_AltGr").map(String::as_str), Some(""));
    assert_eq!(layout.desc().name, "Avro Easy");
    assert_eq!(layout.desc().ver, "3");
}

#[test]
fn legacy_v2_phonetic_layout_falls_back_to_image0() {
    // Shipped v2 phonetic layouts keep the single instructional image in
    // "image0"; Layout::loadDesc() falls back to it so they still display.
    let v2 = r#"{
      "info": {"type": "transliteration", "version": "2",
               "layout": {"name": "Old Phonetic", "version": "3", "image0": "QQ=="}},
      "layout": {}
    }"#;
    let layout = LayoutFile::parse(v2).unwrap();
    assert_eq!(layout.desc().kind, LayoutKind::Phonetic);
    assert_eq!(layout.desc().image, "QQ==");

    // A v3 layout with neither key has no image at all.
    let bare = r#"{"info": {"type": "fixed", "version": "3",
                 "layout": {"name": "Bare", "version": "1"}}, "layout": {}}"#;
    assert!(LayoutFile::parse(bare).unwrap().desc().image.is_empty());
}

#[test]
fn decoded_layout_images_are_pngs() {
    // LayoutViewer::decodeAndDecompress() = base91 decode then ZSTD decompress;
    // the shipped phonetic/khipro images are base91+zstd compressed PNGs.
    for (json, label) in [(AVRO_PHONETIC, "avrophonetic"), (KHIPRO, "khipro")] {
        let image = LayoutFile::parse(json).unwrap().desc().image.clone();
        let bytes = decode_image(&image);
        assert_eq!(
            &bytes[..8],
            &b"\x89PNG\r\n\x1a\n"[..],
            "{label} image is a PNG"
        );

        // IHDR width/height are big-endian u32 at offsets 16 and 20.
        let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        assert!(width > 0 && height > 0, "{label} image has real dimensions");
    }
}

#[test]
fn non_zstd_input_falls_back_to_the_decoded_bytes() {
    // On ZSTD_CONTENTSIZE_ERROR / _UNKNOWN Qt returns the base91-decoded bytes
    // unchanged instead of failing. "A" base91-decodes to a single 0x00 byte.
    assert_eq!(decode_image("A"), vec![0u8]);
}
