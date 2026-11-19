/*
 *  Typed settings model — Phase 3 port of src/shared/Settings.
 *
 *  Expectations are hand-derived from src/shared/Settings.cpp defaults and
 *  the real-Qt fixture, never from the code under test.
 */

use shanti_config::Settings;

const FIXTURE: &str = include_str!("fixtures/keyboard.conf");

#[test]
fn typed_model_reads_every_qt_written_setting() {
    let s = Settings::from_ini(FIXTURE);

    assert_eq!(s.layout_name, "Avro Phonetic");
    assert_eq!(s.layout_path, "avro_phonetic");
    assert!(!s.enter_key_closes_prev_win);
    assert!(s.show_prev_win_fixed);
    assert!(!s.auto_vowel_form_fixed);
    assert!(s.auto_chandra_pos_fixed);
    assert!(s.traditional_kar_fixed);
    assert!(s.number_pad_fixed);
    assert!(!s.old_reph);
    assert!(s.candidate_win_horizontal);
    assert!(!s.show_cw_phonetic);
    assert!(s.suggestion_include_english);
    assert!(s.fixed_old_kar_order);
    assert!(!s.ansi_encoding);
    assert!(s.smart_quoting);
    assert!(s.previous_user_data_remains);
    assert_eq!(s.tray_info_count, 3);
    assert_eq!(s.top_bar_position, (120, 240));
    assert_eq!(s.layout_viewer_position, (50, 60));
    assert!(s.top_bar_visibility);
}

#[test]
fn serialized_settings_match_qt_fixture_byte_for_byte() {
    // The writer must emit the exact shape Qt 5.15 produced (section order,
    // '\' folded keys, bare bools/uints/strings, @Point(x y), trailing \n).
    assert_eq!(Settings::from_ini(FIXTURE).to_ini(), FIXTURE);
}

#[test]
fn edited_settings_survive_ini_round_trip() {
    let mut s = Settings::from_ini(FIXTURE);
    s.layout_name = "Probhat".to_string();
    s.layout_path = "probhat".to_string();
    s.ansi_encoding = true;
    s.tray_info_count = 7;
    s.top_bar_position = (-10, 200);

    assert_eq!(Settings::from_ini(&s.to_ini()), s);
}

#[test]
fn load_missing_file_returns_settings_cpp_defaults() {
    let path =
        std::env::temp_dir().join(format!("shanti-config-missing-{}.conf", std::process::id()));
    let _ = std::fs::remove_file(&path);
    assert_eq!(Settings::load_from(&path).unwrap(), Settings::default());
}

#[test]
fn load_and_save_round_trip_on_disk() {
    let root = std::env::temp_dir().join(format!("shanti-config-disk-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let original = root.join("original.conf");
    let saved = root.join("saved.conf");

    std::fs::write(&original, FIXTURE).unwrap();
    let mut settings = Settings::load_from(&original).unwrap();
    settings.layout_name = "Munir Optima".to_string();
    settings.smart_quoting = false;
    settings.save_to(&saved).unwrap();

    assert_eq!(std::fs::read_to_string(&saved).unwrap(), settings.to_ini());
    assert_eq!(Settings::load_from(&saved).unwrap(), settings);
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn migrated_legacy_store_is_read_by_the_typed_model() {
    // The legacy store has the same INI shape, pre-rebrand. After the
    // one-time migration, the typed model must see its values unchanged.
    let root = std::env::temp_dir().join(format!("shanti-config-migration-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let legacy = root.join("OpenBangla").join("Keyboard.conf");
    let active = root.join("Shanti").join("Shanti.conf");
    let legacy_text = FIXTURE
        .replace("name=Avro Phonetic", "name=Legacy Probhat")
        .replace("ANSI=false", "ANSI=true");
    std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    std::fs::write(&legacy, &legacy_text).unwrap();

    assert!(shanti_paths::migrate_settings_file(&legacy, &active).unwrap());
    let s = Settings::load_from(&active).unwrap();
    assert_eq!(s.layout_name, "Legacy Probhat");
    assert!(s.ansi_encoding);
    assert_eq!(s.top_bar_position, (120, 240));
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn typed_defaults_match_settings_cpp_when_store_is_empty() {
    let s = Settings::from_ini("");

    assert_eq!(s.layout_name, "Avro Phonetic");
    assert_eq!(s.layout_path, "avro_phonetic");
    assert!(!s.enter_key_closes_prev_win);
    assert!(s.show_prev_win_fixed);
    assert!(s.auto_vowel_form_fixed);
    assert!(s.auto_chandra_pos_fixed);
    assert!(!s.traditional_kar_fixed);
    assert!(s.number_pad_fixed);
    assert!(s.old_reph);
    assert!(s.candidate_win_horizontal);
    assert!(s.show_cw_phonetic);
    assert!(s.suggestion_include_english);
    assert!(!s.fixed_old_kar_order);
    assert!(!s.ansi_encoding);
    assert!(s.smart_quoting);
    assert!(s.previous_user_data_remains);
    assert_eq!(s.tray_info_count, 0);
    assert_eq!(s.top_bar_position, (0, 0));
    assert_eq!(s.layout_viewer_position, (0, 0));
    assert!(s.top_bar_visibility);
}
