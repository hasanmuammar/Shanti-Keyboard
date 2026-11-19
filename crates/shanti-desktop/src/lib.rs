/*
 *  Shanti desktop integration — Rust port of src/frontend/FlatpakIbusIntegration
 *  (Phase 3, GNOME/IBus scope).
 *
 *  The generated component XML, launcher script and systemd drop-in are
 *  byte-identical to what the Qt writer produces; the golden fixtures in
 *  tests/fixtures/ are captured from the real Qt 5.15 writer. The uninstall
 *  allowlist and every installed path are compatibility contracts and must
 *  not change.
 *
 *  Unlike shanti-paths (which mirrors QStandardPaths/XDG for the settings and
 *  data stores), this module deliberately mirrors QDir::homePath() + the
 *  literal ".local/share" / ".config" suffixes used by the C++ integration.
 *  Routing these paths through XDG resolution would silently change them
 *  inside the Flatpak and break the contract.
 */

use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub const APP_ID: &str = "org.shanti.ShantiIbus";
pub const SERVICE_UNIT: &str = "org.freedesktop.IBus.session.GNOME.service";
pub const MARKER: &str = "Managed by the Shanti Flatpak integration.";
pub const LEGACY_MARKER: &str = "Managed by Shanti Keyboard Shanti Flatpak integration.";
pub const COMPONENT_NAME: &str = "org.freedesktop.IBus.Shanti";
pub const ENGINE_NAME: &str = "Shanti";

/// Exact host-side paths written by the integration (see FlatpakIbusIntegration::paths()).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paths {
    pub data_home: PathBuf,
    pub config_home: PathBuf,
    pub component: PathBuf,
    pub legacy_component: PathBuf,
    pub launcher: PathBuf,
    pub drop_in: PathBuf,
}

impl Paths {
    /// Mirror `QDir::homePath() + "/.local/share"` / `+ "/.config"`.
    pub fn new(home: &Path) -> Self {
        let data_home = home.join(".local/share");
        let config_home = home.join(".config");
        Self {
            component: data_home.join("ibus/component/shanti-flatpak.xml"),
            legacy_component: data_home.join("ibus/component/shanti-flatpak-shanti.xml"),
            launcher: data_home.join("shanti-ibus/ibus-flatpak-host-wrapper.sh"),
            drop_in: config_home.join(format!(
                "systemd/user/{SERVICE_UNIT}.d/90-shanti-flatpak-ibus.conf"
            )),
            data_home,
            config_home,
        }
    }

    /// Resolve from `$HOME`, the Qt `QDir::homePath()` equivalent on Linux.
    pub fn from_env() -> io::Result<Self> {
        let home = std::env::var_os("HOME")
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;
        Ok(Self::new(Path::new(&home)))
    }
}

/// Escape text the way QXmlStreamWriter does for an element body.
fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// The IBus component descriptor. `version` is the application version.
pub fn component_xml(launcher: &Path, version: &str) -> String {
    // QSettings-style shell quoting of the launcher inside the <exec> element.
    let quoted = launcher.to_string_lossy().replace('\'', "'\\''");
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!--{MARKER}-->\n\
         <component>\n\
         \x20   <name>{COMPONENT_NAME}</name>\n\
         \x20   <description>Shanti Keyboard (Flatpak engine)</description>\n\
         \x20   <exec>'{}'</exec>\n\
         \x20   <version>{}</version>\n\
         \x20   <author>Shanti</author>\n\
         \x20   <license>GPL-3.0-or-later</license>\n\
         \x20   <homepage>https://github.com/hasanmuammar/Shanti-Keyboard-Shanti</homepage>\n\
         \x20   <textdomain>shanti-keyboard</textdomain>\n\
         \x20   <engines>\n\
         \x20       <engine>\n\
         \x20           <name>{ENGINE_NAME}</name>\n\
         \x20           <longname>Shanti Keyboard</longname>\n\
         \x20           <description>Shanti Bengali input method running in Flatpak</description>\n\
         \x20           <language>bn</language>\n\
         \x20           <license>GPL-3.0-or-later</license>\n\
         \x20           <author>Shanti</author>\n\
         \x20           <layout>default</layout>\n\
         \x20           <rank>80</rank>\n\
         \x20           <symbol>অ</symbol>\n\
         \x20       </engine>\n\
         \x20   </engines>\n\
         </component>\n",
        xml_escape(&quoted),
        xml_escape(version)
    )
}

/// Host launcher that bridges the host IBus socket into the Flatpak engine.
/// Kept as a verbatim copy of the C++ raw string literal.
pub fn host_wrapper() -> &'static str {
    r####"#!/usr/bin/env bash
# Managed by the Shanti Flatpak integration.
set -euo pipefail

fail() { printf 'Shanti IBus integration: %s\n' "$*" >&2; exit 1; }
command -v flatpak >/dev/null 2>&1 || fail 'flatpak command not found'
command -v ibus >/dev/null 2>&1 || fail 'ibus command not found'

ibus_address="$(ibus address)" || fail 'could not get the host IBus address'
[[ "$ibus_address" == unix:path=* ]] || fail 'unsupported host IBus address'
ibus_socket="${ibus_address#unix:path=}"
ibus_socket="${ibus_socket%%,*}"
[[ -n "$ibus_socket" && "$ibus_socket" == /* ]] || fail 'invalid host IBus socket path'

host_config_home="${XDG_CONFIG_HOME:-$HOME/.config}"
config_dir="$host_config_home/Shanti"
flatpak_args=("--filesystem=$ibus_socket:ro" "--env=IBUS_ADDRESS=$ibus_address" "--env=XDG_CONFIG_HOME=$host_config_home")
if [[ -d "$config_dir" ]]; then
    flatpak_args+=("--filesystem=$config_dir:ro")
fi

flatpak_pid=""
cleanup() {
    local status=$?
    trap - EXIT INT TERM HUP
    if [[ -n "$flatpak_pid" ]]; then
        if kill -0 -- "-$flatpak_pid" 2>/dev/null; then
            kill -TERM -- "-$flatpak_pid" 2>/dev/null || true
            for _ in {1..20}; do
                kill -0 -- "-$flatpak_pid" 2>/dev/null || break
                sleep 0.1
            done
            kill -KILL -- "-$flatpak_pid" 2>/dev/null || true
        fi
        wait "$flatpak_pid" 2>/dev/null || true
    fi
    exit "$status"
}
trap cleanup EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM
set -m

flatpak run "${flatpak_args[@]}" \
    --talk-name=org.freedesktop.IBus \
    --own-name=org.freedesktop.IBus.Shanti \
    --command=/app/libexec/ibus-engine-shanti \
    org.shanti.ShantiIbus --ibus "$@" &
flatpak_pid=$!

set +e
wait "$flatpak_pid"
status=$?
set -e
exit "$status"
"####
}

/// systemd unit-file escaping: `%` → `%%`, `\` → `\\`, `"` → `\"`.
pub fn systemd_escape(value: &str) -> String {
    value
        .replace('%', "%%")
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
}

/// The GNOME IBus service drop-in that points IBus at our component file.
pub fn service_drop_in(paths: &Paths) -> String {
    let component_dir = paths.component.parent().unwrap_or(Path::new(""));
    let component_path = format!(
        "{}:/usr/share/ibus/component",
        component_dir.to_string_lossy()
    );
    format!(
        "# {MARKER}\n\
         [Service]\n\
         Environment=\"IBUS_COMPONENT_PATH={}\"\n\
         ExecStart=\n\
         ExecStart=/usr/bin/ibus-daemon --panel disable --cache=refresh\n",
        systemd_escape(&component_path)
    )
}

/// True when the bytes carry either the current or the pre-rebrand marker.
pub fn has_owned_marker(contents: &[u8]) -> bool {
    let text = String::from_utf8_lossy(contents);
    text.contains(MARKER) || text.contains(LEGACY_MARKER)
}

/// Mirror `hasMarker()`: reject symlinks and non-files, then look for a marker.
pub fn has_marker(path: &Path) -> io::Result<bool> {
    let meta = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e),
    };
    if meta.file_type().is_symlink() || !meta.is_file() {
        return Ok(false);
    }
    Ok(has_owned_marker(&fs::read(path)?))
}

fn io_error(message: impl Into<String>) -> io::Error {
    io::Error::other(message.into())
}

/// Write a managed contract file atomically. Never replace an existing
/// unmanaged file or a symlink; create parent directories and set `mode`.
pub fn atomic_write(path: &Path, contents: &[u8], mode: u32) -> io::Result<()> {
    if fs::symlink_metadata(path).is_ok() && !has_marker(path)? {
        return Err(io_error(format!(
            "Refusing to replace an existing unmanaged file: {}",
            path.display()
        )));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    // Write to a sibling temp file then rename, mirroring QSaveFile::commit().
    let tmp = path.with_extension("shanti-tmp");
    fs::write(&tmp, contents)?;
    fs::set_permissions(&tmp, fs::Permissions::from_mode(mode))?;
    match fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = fs::remove_file(&tmp);
            Err(e)
        }
    }
}

/// Find another `*.conf` in the drop-in directory that would conflict with
/// this integration. Our own file and already-managed files are ignored.
pub fn has_conflicting_drop_in(dir: &Path, own: &Path) -> io::Result<Option<PathBuf>> {
    // QDir::entryList() on a not-yet-created directory is empty, not an error.
    let mut entries: Vec<PathBuf> = match fs::read_dir(dir) {
        Ok(read) => read,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    }
    .filter_map(|entry| entry.ok().map(|e| e.path()))
    .filter(|path| {
        path != own
            && path.extension() == Some(OsStr::new("conf"))
            && fs::symlink_metadata(path)
                .map(|m| m.is_file() && !m.file_type().is_symlink())
                .unwrap_or(false)
    })
    .collect();
    entries.sort();
    for path in entries {
        if has_marker(&path)? {
            continue;
        }
        let bytes = fs::read(&path)?;
        let contents = String::from_utf8_lossy(&bytes);
        if contents.contains("IBUS_COMPONENT_PATH") || contents.contains("ExecStart=") {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

/// The `.shanti-backup` name used when preserving a conflicting drop-in.
pub fn backup_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".shanti-backup");
    PathBuf::from(name)
}

/// Numbered backup candidate: `<base>`, then `<base>-2`, `-3`, …
pub fn backup_path_candidate(path: &Path, suffix: u32) -> PathBuf {
    if suffix <= 1 {
        backup_path(path)
    } else {
        let mut name = backup_path(path).into_os_string();
        name.push(format!("-{suffix}"));
        PathBuf::from(name)
    }
}

/// Remove all four managed files, but only after verifying every existing
/// one carries our marker. This is the file-action half of removeIntegration;
/// the systemd Reload/RestartUnit calls belong to the GTK UI layer.
pub fn remove_files(paths: &Paths) -> io::Result<()> {
    let owned = [
        &paths.component,
        &paths.launcher,
        &paths.drop_in,
        &paths.legacy_component,
    ];
    for path in owned {
        if fs::symlink_metadata(path).is_ok() && !has_marker(path)? {
            return Err(io_error(format!(
                "Refusing to remove a file that is not managed by Shanti: {}",
                path.display()
            )));
        }
    }
    for path in owned {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// Desktop scope gate: the C++ integration only runs inside the IBus Flatpak
/// on a GNOME session (`isApplicable()`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopEnvironment {
    Gnome,
    Other,
}

pub fn is_applicable(flatpak_id: Option<&str>, desktop: DesktopEnvironment) -> bool {
    flatpak_id == Some(APP_ID) && desktop == DesktopEnvironment::Gnome
}

/// Errors from the setup/removal flows. Conflicts and unsafe paths carry the
/// information the GTK panel needs to explain the refusal to the user.
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Conflict(PathBuf),
    Unmanaged(PathBuf),
    ServiceUnavailable,
    UnsafePath,
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "{err}"),
            Error::Conflict(path) => write!(
                f,
                "An existing IBus service override may conflict with this setup: {}",
                path.display()
            ),
            Error::Unmanaged(path) => write!(
                f,
                "Refusing to replace an existing unmanaged file: {}",
                path.display()
            ),
            Error::ServiceUnavailable => write!(
                f,
                "The GNOME IBus user service was not found or is unavailable in this session."
            ),
            Error::UnsafePath => write!(
                f,
                "This home directory path cannot be safely represented in the IBus or systemd configuration."
            ),
        }
    }
}

impl std::error::Error for Error {}

/// The systemd user-manager calls used by install/remove. The C++ talks to
/// `org.freedesktop.systemd1` over the session bus; the GTK panel supplies the
/// concrete backend, and tests supply a recording fake.
pub trait ServiceControl {
    fn is_active(&self) -> Result<bool, Error>;
    fn reload(&self) -> Result<(), Error>;
    fn restart_unit(&self) -> Result<(), Error>;
}

/// Any managed file still present? Mirrors `hasManagedFiles()`.
pub fn has_managed_files(paths: &Paths) -> io::Result<bool> {
    Ok(has_marker(&paths.component)?
        || has_marker(&paths.launcher)?
        || has_marker(&paths.drop_in)?
        || has_marker(&paths.legacy_component)?)
}

/// File-only portion of `setupNeeded()`; the service-active check is supplied
/// by `ServiceControl::is_active` at the call site.
pub fn setup_needed_files(paths: &Paths) -> io::Result<bool> {
    let launcher_executable = fs::metadata(&paths.launcher)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false);
    if !has_marker(&paths.component)?
        || !has_marker(&paths.launcher)?
        || !has_marker(&paths.drop_in)?
        || !launcher_executable
    {
        return Ok(true);
    }
    let component_dir = paths.component.parent().unwrap_or(Path::new(""));
    let drop_in = fs::read_to_string(&paths.drop_in).unwrap_or_default();
    Ok(!drop_in.contains(&component_dir.to_string_lossy().into_owned()))
}

/// Write the three contract files and drop a marked legacy component. This is
/// the file-action half of `installIntegration()` (no DBus calls).
pub fn install_files(paths: &Paths, version: &str) -> Result<(), Error> {
    let data = paths.data_home.to_string_lossy();
    let config = paths.config_home.to_string_lossy();
    if data.contains(':')
        || data.contains('\n')
        || data.contains('\r')
        || config.contains('\n')
        || config.contains('\r')
    {
        return Err(Error::UnsafePath);
    }

    if fs::symlink_metadata(&paths.drop_in).is_ok() && !has_marker(&paths.drop_in)? {
        return Err(Error::Conflict(paths.drop_in.clone()));
    }
    let drop_in_dir = paths.drop_in.parent().unwrap_or(Path::new(""));
    if let Some(conflict) = has_conflicting_drop_in(drop_in_dir, &paths.drop_in)? {
        return Err(Error::Conflict(conflict));
    }

    atomic_write(
        &paths.component,
        component_xml(&paths.launcher, version).as_bytes(),
        0o644,
    )?;
    atomic_write(&paths.launcher, host_wrapper().as_bytes(), 0o755)?;
    atomic_write(&paths.drop_in, service_drop_in(paths).as_bytes(), 0o644)?;

    if has_marker(&paths.legacy_component)? {
        fs::remove_file(&paths.legacy_component)?;
    }
    Ok(())
}

/// Full install flow: verify the GNOME IBus service, write the contract files,
/// then reload and restart it.
pub fn install(paths: &Paths, version: &str, service: &dyn ServiceControl) -> Result<(), Error> {
    if !service.is_active()? {
        return Err(Error::ServiceUnavailable);
    }
    install_files(paths, version)?;
    service.reload()?;
    service.restart_unit()?;
    Ok(())
}

/// Full removal flow: validate and delete the managed files, then restart the
/// IBus service. Mirrors `removeIntegration()`.
pub fn remove(paths: &Paths, service: &dyn ServiceControl) -> Result<(), Error> {
    // Validate every managed path before deleting anything.
    let owned = [
        &paths.component,
        &paths.launcher,
        &paths.drop_in,
        &paths.legacy_component,
    ];
    for path in owned {
        if fs::symlink_metadata(path).is_ok() && !has_marker(path)? {
            return Err(Error::Unmanaged(path.clone()));
        }
    }
    remove_files(paths)?;
    service.reload()?;
    service.restart_unit()?;
    Ok(())
}
