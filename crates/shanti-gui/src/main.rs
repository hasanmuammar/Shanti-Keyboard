use std::path::{Path, PathBuf};
use std::process::ExitCode;

use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar};
use gtk::{Box as GtkBox, Label, Orientation};
use shanti_config::Settings;

const REQUIRED_FILES: &[&str] = &[
    "layouts/avrophonetic.json",
    "data/autocorrect.json",
    "data/dictionary.json",
    "data/suffix.json",
    "data/regex.json",
];

fn print_help() {
    println!("Shanti Keyboard\n\nUsage: shanti-gui-rust [OPTIONS]\n\nOptions:\n      --version     Print version information\n      --check-data  Check installed layouts and dictionary data\n  -h, --help        Print this help");
}

/// Human-readable snapshot of the typed store, shown in the shell until
/// the full settings panel lands. Kept pure so it can be tested headless.
fn settings_summary(settings: &Settings) -> String {
    format!(
        "Layout: {} ({})\nANSI encoding: {}\nTray info count: {}\nTop bar: {} at ({}, {})",
        settings.layout_name,
        settings.layout_path,
        if settings.ansi_encoding { "on" } else { "off" },
        settings.tray_info_count,
        if settings.top_bar_visibility {
            "shown"
        } else {
            "hidden"
        },
        settings.top_bar_position.0,
        settings.top_bar_position.1,
    )
}

fn check_data(root: &Path) -> Result<(), Vec<PathBuf>> {
    let mut missing = Vec::new();
    let layouts = root.join("layouts");
    if !layouts.is_dir() {
        missing.push(layouts);
    }
    for relative in REQUIRED_FILES {
        let path = root.join(relative);
        if !path.is_file() {
            missing.push(path);
        }
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing)
    }
}

fn run() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut check_data_requested = false;
    let mut show_version = false;
    for arg in args.by_ref() {
        match arg.as_str() {
            "--version" | "-v" => show_version = true,
            "--check-data" => check_data_requested = true,
            "--help" | "-h" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            _ => {
                eprintln!("Unknown option: {arg}\nTry --help for usage.");
                return ExitCode::from(2);
            }
        }
    }

    if show_version {
        println!(
            "Shanti Keyboard {}",
            option_env!("SHANTI_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))
        );
        return ExitCode::SUCCESS;
    }

    if check_data_requested {
        let root = shanti_paths::engine_data_dir();
        return match check_data(&root) {
            Ok(()) => {
                println!("Shanti runtime data files are present.");
                ExitCode::SUCCESS
            }
            Err(missing) => {
                for path in missing {
                    eprintln!("Missing required runtime data: {}", path.display());
                }
                ExitCode::FAILURE
            }
        };
    }

    let application = Application::builder()
        .application_id("org.shanti.ShantiGtkDev")
        .build();
    application.connect_activate(|application| {
        // First-run legacy migration and user-dir bootstrap, then the typed
        // store shared with the IBus engine. Errors fall back to defaults:
        // the shell is a control plane, not a required input-method runtime.
        shanti_paths::ensure_user_dirs();
        let settings = Settings::load().unwrap_or_default();

        let content = GtkBox::new(Orientation::Vertical, 0);
        content.append(&HeaderBar::new());

        let body = GtkBox::new(Orientation::Vertical, 12);
        body.set_margin_top(32);
        body.set_margin_bottom(32);
        body.set_margin_start(32);
        body.set_margin_end(32);

        let heading = Label::new(Some("Shanti Keyboard"));
        heading.add_css_class("title-1");
        let description = Label::new(Some(
            "GTK application shell. Settings and keyboard controls are being migrated.",
        ));
        description.set_wrap(true);

        let summary = Label::new(Some(&settings_summary(&settings)));
        summary.set_wrap(true);
        summary.set_xalign(0.0);

        body.append(&heading);
        body.append(&description);
        body.append(&summary);
        content.append(&body);

        let window = ApplicationWindow::builder()
            .application(application)
            .title("Shanti Keyboard")
            .default_width(560)
            .default_height(360)
            .content(&content)
            .build();
        window.present();
    });
    application.run().into()
}

fn main() -> ExitCode {
    run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_summary_shows_the_loaded_typed_model() {
        let settings = shanti_config::Settings::from_ini(
            "[layout]\nname=Probhat\npath=probhat\n\
             [settings]\nANSI=true\nTrayInfoCount=5\n\
             [window]\nvisibility\\TopBar=false\npositions\\TopBar=@Point(10 20)\n",
        );
        assert_eq!(
            settings_summary(&settings),
            "Layout: Probhat (probhat)\nANSI encoding: on\nTray info count: 5\nTop bar: hidden at (10, 20)"
        );
    }

    #[test]
    fn data_check_requires_layout_directory_and_runtime_files() {
        let root = std::env::temp_dir().join(format!("shanti-gui-data-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("layouts")).unwrap();
        for relative in REQUIRED_FILES {
            let path = root.join(relative);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "{}").unwrap();
        }
        assert_eq!(check_data(&root), Ok(()));
        std::fs::remove_file(root.join("data/regex.json")).unwrap();
        assert_eq!(check_data(&root), Err(vec![root.join("data/regex.json")]));
        std::fs::remove_dir_all(root).unwrap();
    }
}
