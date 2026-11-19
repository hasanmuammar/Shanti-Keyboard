use std::path::{Path, PathBuf};
use std::process::ExitCode;

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

    eprintln!("This Rust migration build contains the CLI shell only; the Qt GUI remains the active application.");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    run()
}

#[cfg(test)]
mod tests {
    use super::*;

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
