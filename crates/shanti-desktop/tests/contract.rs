/*
 *  Byte-exact parity for the GNOME IBus desktop-integration contract files.
 *
 *  Expectations are captured from the real Qt 5.15 writer (see
 *  tests/fixtures/README.md), never from the Rust code under test.
 */

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use shanti_desktop::{
    atomic_write, backup_path, component_xml, has_conflicting_drop_in, has_marker,
    has_owned_marker, host_wrapper, remove_files, service_drop_in, systemd_escape, Paths, MARKER,
};

const HOME: &str = "/home/testuser";

#[test]
fn paths_match_the_qt_integration_exactly() {
    let p = Paths::new(Path::new(HOME));
    assert_eq!(p.data_home, Path::new("/home/testuser/.local/share"));
    assert_eq!(p.config_home, Path::new("/home/testuser/.config"));
    assert_eq!(
        p.component,
        Path::new("/home/testuser/.local/share/ibus/component/shanti-flatpak.xml")
    );
    assert_eq!(
        p.legacy_component,
        Path::new("/home/testuser/.local/share/ibus/component/shanti-flatpak-shanti.xml")
    );
    assert_eq!(
        p.launcher,
        Path::new("/home/testuser/.local/share/shanti-ibus/ibus-flatpak-host-wrapper.sh")
    );
    assert_eq!(
        p.drop_in,
        Path::new("/home/testuser/.config/systemd/user/org.freedesktop.IBus.session.GNOME.service.d/90-shanti-flatpak-ibus.conf")
    );
}

#[test]
fn component_xml_is_byte_identical_to_qt() {
    let p = Paths::new(Path::new(HOME));
    assert_eq!(
        component_xml(&p.launcher, "3.0.0"),
        include_str!("fixtures/component.xml.golden")
    );
}

#[test]
fn host_wrapper_is_byte_identical_to_qt() {
    assert_eq!(host_wrapper(), include_str!("fixtures/wrapper.sh.golden"));
}

#[test]
fn service_drop_in_is_byte_identical_to_qt() {
    let p = Paths::new(Path::new(HOME));
    assert_eq!(
        service_drop_in(&p),
        include_str!("fixtures/dropin.conf.golden")
    );
}

#[test]
fn single_quote_in_launcher_path_is_shell_escaped() {
    let xml = component_xml(Path::new("/tmp/it's/ibus-flatpak-host-wrapper.sh"), "1.0");
    assert!(xml.contains("<exec>'/tmp/it'\\''s/ibus-flatpak-host-wrapper.sh'</exec>"));
}

#[test]
fn systemd_escape_matches_the_qt_rules() {
    assert_eq!(systemd_escape("plain"), "plain");
    assert_eq!(systemd_escape("100%a\\b\"c"), "100%%a\\\\b\\\"c");
}

#[test]
fn marker_detection_accepts_current_and_legacy_markers() {
    assert!(has_owned_marker(format!("header\n# {MARKER}\n").as_bytes()));
    assert!(has_owned_marker(
        b"# Managed by Shanti Keyboard Shanti Flatpak integration.\n"
    ));
    assert!(!has_owned_marker(b"unmanaged contents\n"));
}

#[test]
fn has_marker_rejects_symlinks_and_unmanaged_files() {
    let root = std::env::temp_dir().join(format!("shanti-desktop-marker-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();

    let managed = root.join("managed.conf");
    fs::write(&managed, format!("# {MARKER}\nkey=value\n")).unwrap();
    assert!(has_marker(&managed).unwrap());

    let unmanaged = root.join("unmanaged.conf");
    fs::write(&unmanaged, "key=value\n").unwrap();
    assert!(!has_marker(&unmanaged).unwrap());

    let link = root.join("link.conf");
    std::os::unix::fs::symlink(&managed, &link).unwrap();
    assert!(!has_marker(&link).unwrap());
    assert!(!has_marker(&root.join("missing.conf")).unwrap());

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn atomic_write_creates_parents_and_refuses_unmanaged_files() {
    let root = std::env::temp_dir().join(format!("shanti-desktop-write-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let target = root.join("sub/dir/file.conf");

    atomic_write(&target, format!("# {MARKER}\nvalue\n").as_bytes(), 0o644).unwrap();
    assert_eq!(
        fs::read(&target).unwrap(),
        format!("# {MARKER}\nvalue\n").as_bytes()
    );
    assert_eq!(
        fs::metadata(&target).unwrap().permissions().mode() & 0o777,
        0o644
    );

    // Rewriting an existing managed file is allowed only when it still carries
    // the owned marker; an unmanaged file is never replaced.
    atomic_write(&target, format!("# {MARKER}\nnew\n").as_bytes(), 0o644).unwrap();
    assert_eq!(
        fs::read(&target).unwrap(),
        format!("# {MARKER}\nnew\n").as_bytes()
    );

    let foreign = root.join("foreign.conf");
    fs::write(&foreign, "do not touch\n").unwrap();
    assert!(atomic_write(&foreign, b"overwrite\n", 0o644).is_err());
    assert_eq!(fs::read(&foreign).unwrap(), b"do not touch\n");

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn conflicting_drop_in_is_detected_but_managed_and_own_files_are_not() {
    let root = std::env::temp_dir().join(format!("shanti-desktop-conflict-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let dir = root.join("service.d");
    fs::create_dir_all(&dir).unwrap();
    let own = dir.join("90-shanti-flatpak-ibus.conf");
    fs::write(&own, format!("# {MARKER}\n[Service]\n")).unwrap();
    let managed = dir.join("10-managed.conf");
    fs::write(&managed, format!("# {MARKER}\nExecStart=other\n")).unwrap();
    assert_eq!(has_conflicting_drop_in(&dir, &own).unwrap(), None);

    let foreign = dir.join("20-foreign.conf");
    fs::write(&foreign, "IBUS_COMPONENT_PATH=/evil\n").unwrap();
    assert_eq!(
        has_conflicting_drop_in(&dir, &own).unwrap(),
        Some(foreign.clone())
    );

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn backup_path_uses_the_qt_suffix_and_numbering() {
    assert_eq!(
        backup_path(Path::new("/x/90-shanti-flatpak-ibus.conf")),
        PathBuf::from("/x/90-shanti-flatpak-ibus.conf.shanti-backup")
    );
}

#[test]
fn remove_files_refuses_unmanaged_files_and_keeps_them() {
    let root = std::env::temp_dir().join(format!("shanti-desktop-remove-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let component = root.join("ibus/component/shanti-flatpak.xml");
    let launcher = root.join("shanti-ibus/ibus-flatpak-host-wrapper.sh");
    let drop_in = root.join("systemd/90-shanti-flatpak-ibus.conf");
    let legacy = root.join("ibus/component/shanti-flatpak-shanti.xml");
    fs::create_dir_all(component.parent().unwrap()).unwrap();
    fs::create_dir_all(launcher.parent().unwrap()).unwrap();
    fs::create_dir_all(drop_in.parent().unwrap()).unwrap();
    fs::write(&component, format!("<!--{MARKER}-->\n")).unwrap();
    fs::write(&launcher, format!("# {MARKER}\n")).unwrap();
    fs::write(&drop_in, format!("# {MARKER}\n")).unwrap();
    fs::write(&legacy, "unmanaged legacy file\n").unwrap();

    let p = Paths {
        component: component.clone(),
        launcher: launcher.clone(),
        drop_in: drop_in.clone(),
        legacy_component: legacy.clone(),
        ..Paths::new(&root)
    };
    assert!(remove_files(&p).is_err());
    assert!(component.exists() && launcher.exists() && drop_in.exists() && legacy.exists());

    // Once the legacy file also carries the marker, all four are removed.
    fs::write(&legacy, format!("<!--{MARKER}-->\n")).unwrap();
    remove_files(&p).unwrap();
    assert!(!component.exists() && !launcher.exists() && !drop_in.exists() && !legacy.exists());

    fs::remove_dir_all(&root).unwrap();
}
