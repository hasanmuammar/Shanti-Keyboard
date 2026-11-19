/*
 *  Setup/removal flow behavior mirrored from FlatpakIbusIntegration.cpp.
 *  The systemd/DBus calls sit behind ServiceControl so the file actions can
 *  be exercised headlessly with a recording fake.
 */

use std::cell::RefCell;
use std::fs;
use std::os::unix::fs::PermissionsExt;

use shanti_desktop::{
    has_managed_files, install, install_files, remove, setup_needed_files, DesktopEnvironment,
    Error, Paths, ServiceControl,
};

#[derive(Default)]
struct FakeService {
    active: bool,
    calls: RefCell<Vec<&'static str>>,
}

impl ServiceControl for FakeService {
    fn is_active(&self) -> Result<bool, Error> {
        Ok(self.active)
    }
    fn reload(&self) -> Result<(), Error> {
        self.calls.borrow_mut().push("reload");
        Ok(())
    }
    fn restart_unit(&self) -> Result<(), Error> {
        self.calls.borrow_mut().push("restart");
        Ok(())
    }
}

fn sandbox(tag: &str) -> (std::path::PathBuf, Paths) {
    let root =
        std::env::temp_dir().join(format!("shanti-desktop-flow-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let paths = Paths::new(&root);
    (root, paths)
}

#[test]
fn applicability_requires_the_ibus_flatpak_id_and_gnome() {
    assert!(shanti_desktop::is_applicable(
        Some("org.shanti.ShantiIbus"),
        DesktopEnvironment::Gnome
    ));
    assert!(!shanti_desktop::is_applicable(
        Some("org.shanti.Other"),
        DesktopEnvironment::Gnome
    ));
    assert!(!shanti_desktop::is_applicable(
        None,
        DesktopEnvironment::Gnome
    ));
    assert!(!shanti_desktop::is_applicable(
        Some("org.shanti.ShantiIbus"),
        DesktopEnvironment::Other
    ));
}

#[test]
fn install_files_writes_all_three_contract_files_and_drops_the_legacy_one() {
    let (_root, paths) = sandbox("install");
    fs::create_dir_all(paths.legacy_component.parent().unwrap()).unwrap();
    fs::write(
        &paths.legacy_component,
        "<!--Managed by Shanti Keyboard Shanti Flatpak integration.-->\n",
    )
    .unwrap();

    install_files(&paths, "3.0.0").unwrap();

    assert!(fs::metadata(&paths.component).unwrap().permissions().mode() & 0o777 == 0o644);
    assert!(fs::metadata(&paths.launcher).unwrap().permissions().mode() & 0o777 == 0o755);
    assert!(fs::metadata(&paths.drop_in).unwrap().permissions().mode() & 0o777 == 0o644);
    assert!(!paths.legacy_component.exists());
    assert!(has_managed_files(&paths).unwrap());
    assert!(!setup_needed_files(&paths).unwrap());
}

#[test]
fn install_refuses_an_unmanaged_drop_in_without_touching_anything() {
    let (_root, paths) = sandbox("conflict");
    fs::create_dir_all(paths.drop_in.parent().unwrap()).unwrap();
    fs::write(&paths.drop_in, "ExecStart=/someone/elses/ibus\n").unwrap();

    let err = install_files(&paths, "3.0.0").unwrap_err();
    assert!(matches!(err, Error::Conflict(_)));
    assert_eq!(
        fs::read_to_string(&paths.drop_in).unwrap(),
        "ExecStart=/someone/elses/ibus\n"
    );
    assert!(!paths.component.exists() && !paths.launcher.exists());
}

#[test]
fn install_checks_service_health_and_reloads_systemd() {
    let (_root, paths) = sandbox("install-service");
    let service = FakeService {
        active: true,
        ..Default::default()
    };
    install(&paths, "3.0.0", &service).unwrap();
    assert_eq!(*service.calls.borrow(), vec!["reload", "restart"]);

    let (_root2, paths2) = sandbox("install-inactive");
    let inactive = FakeService {
        active: false,
        ..Default::default()
    };
    assert!(matches!(
        install(&paths2, "3.0.0", &inactive),
        Err(Error::ServiceUnavailable)
    ));
    assert!(!paths2.component.exists());
}

#[test]
fn remove_deletes_managed_files_then_reloads_systemd() {
    let (_root, paths) = sandbox("remove");
    let service = FakeService {
        active: true,
        ..Default::default()
    };
    install(&paths, "3.0.0", &service).unwrap();
    service.calls.borrow_mut().clear();

    remove(&paths, &service).unwrap();
    assert_eq!(*service.calls.borrow(), vec!["reload", "restart"]);
    assert!(!has_managed_files(&paths).unwrap());
    assert!(setup_needed_files(&paths).unwrap());
}
