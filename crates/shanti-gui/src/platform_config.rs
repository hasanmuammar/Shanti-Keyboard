use std::process::Command;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopEnvironment {
    Gnome,
    Other,
}

pub fn detect_desktop_environment() -> DesktopEnvironment {
    let desktops = [
        std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default(),
        std::env::var("DESKTOP_SESSION").unwrap_or_default(),
        std::env::var("GDMSESSION").unwrap_or_default(),
        std::env::var("XDG_SESSION_DESKTOP").unwrap_or_default(),
    ]
    .join(";")
    .to_lowercase();

    if desktops.contains("gnome") {
        DesktopEnvironment::Gnome
    } else {
        DesktopEnvironment::Other
    }
}

pub fn desktop_environment_to_string(de: DesktopEnvironment) -> &'static str {
    match de {
        DesktopEnvironment::Gnome => "GNOME",
        DesktopEnvironment::Other => "Unknown",
    }
}

pub fn is_app_indicator_enabled() -> bool {
    let output = Command::new("gnome-extensions")
        .args(["list", "--enabled"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            text.lines()
                .any(|l| l.contains("appindicatorsupport@rgcjonas.gmail.com")
                    || l.contains("ubuntu-appindicators@ubuntu.com"))
        }
        _ => false,
    }
}

pub fn should_show_tray_icon() -> bool {
    detect_desktop_environment() != DesktopEnvironment::Gnome || is_app_indicator_enabled()
}

fn read_dconf_key(key: &str) -> String {
    let output = Command::new("dconf").args(["read", key]).output();
    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }
        _ => String::new(),
    }
}

fn write_dconf_setting(key: &str, value: &str) -> bool {
    let output = Command::new("dconf")
        .args(["write", key, value])
        .output();
    matches!(output, Ok(out) if out.status.success())
}

fn parse_input_sources(raw: &str) -> Vec<String> {
    let raw = raw.replace('[', "").replace(']', "");
    let mut sources = Vec::new();
    let mut remaining = raw.as_str();
    while let Some(idx) = remaining.find("),") {
        sources.push(remaining[..=idx].trim().to_string());
        remaining = &remaining[idx + 2..];
    }
    if !remaining.trim().is_empty() {
        sources.push(remaining.trim().to_string());
    }
    sources
}

pub fn setup_gnome_input_source() {
    const KEY: &str = "/org/gnome/desktop/input-sources/sources";
    let sources = parse_input_sources(&read_dconf_key(KEY));
    if sources.iter().any(|s| s == "('ibus', 'Shanti')") {
        return;
    }

    let first_source = sources.first().cloned().unwrap_or_else(|| "('xkb', 'us')".to_string());
    let mut new_sources = sources;
    new_sources.push("('ibus', 'Shanti')".to_string());

    let joined = new_sources.join(", ");
    write_dconf_setting(KEY, &format!("[{}]", joined));
    write_dconf_setting(
        "/org/gnome/desktop/input-sources/mru-sources",
        &format!("[{}]", first_source),
    );
}

pub fn setup_input_sources() {
    if detect_desktop_environment() == DesktopEnvironment::Gnome {
        setup_gnome_input_source();
    }
}
