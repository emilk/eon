//! Eon formatter.
//!
//! Formats Eon files according to the Eon syntax.
//! See <https://github.com/emilk/eon> for more.

use std::{fs, path::Path, process};

use clap::{Arg, Command};
use ignore::WalkBuilder;

fn main() {
    let matches = Command::new("Eon formatter")
        .about("Format Eon files")
        .arg(
            Arg::new("files")
                .help("Files or directories to format")
                .num_args(1..)
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("check")
                .long("check")
                .help("Check if files are formatted without modifying them")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("extension")
                .long("ext")
                .short('e')
                .help("File extension to process")
                .default_value("eon")
                .value_name("EXT"),
        )
        .get_matches();

    let paths: Vec<&str> = matches
        .get_many::<String>("files")
        .expect("Missing file paths")
        .map(|s| s.as_str())
        .collect();
    let check_mode = matches.get_flag("check");
    let extension = matches
        .get_one::<String>("extension")
        .expect("Missing extension")
        .as_str();

    let mut exit_code = 0;

    let mut file_paths = Vec::new();

    for path_str in paths {
        let path = Path::new(path_str);

        if path.is_file() {
            file_paths.push(path.to_path_buf());
        } else if path.is_dir() {
            let walker = WalkBuilder::new(path)
                .git_ignore(true)
                .git_exclude(true)
                .git_global(true)
                .hidden(false)
                .build();

            for entry in walker {
                match entry {
                    Ok(entry) => {
                        let entry_path = entry.path();
                        if entry_path.is_file() && has_extension(entry_path, extension) {
                            file_paths.push(path.to_path_buf());
                        }
                    }
                    Err(err) => {
                        eprintln!("Error walking directory: {err}");
                        exit_code = 1;
                    }
                }
            }
        } else {
            eprintln!("Path does not exist: {}", path.display());
            exit_code = 1;
        }
    }

    let mut num_files_changed = 0;

    for path in &file_paths {
        match process_file(path, check_mode) {
            Ok(false) => {}
            Ok(true) => {
                num_files_changed += 1;
                if check_mode {
                    eprintln!("Would format: {}", path.display());
                    exit_code = 1;
                } else {
                    eprintln!("Formatted: {}", path.display());
                }
            }
            Err(e) => {
                eprintln!("Error processing file {}: {}", path.display(), e);
                exit_code = 1;
            }
        }
    }

    let num_files_found = file_paths.len();

    if check_mode {
        if num_files_changed > 0 {
            eprintln!("{num_files_changed}/{num_files_found} file(s) would be reformatted");
            exit_code = 1;
        } else {
            eprintln!("All {num_files_found} file(s) are correctly formatted");
        }
    } else {
        eprintln!(
            "Formatted {} file(s), {} file(s) left unchanged",
            num_files_changed,
            num_files_found - num_files_changed
        );
    }

    #[allow(clippy::exit, clippy::allow_attributes)]
    process::exit(exit_code);
}

fn has_extension(entry_path: &Path, extension: &str) -> bool {
    if let Some(ext) = entry_path.extension() {
        ext == extension
    } else {
        false
    }
}

fn process_file(path: &Path, check_mode: bool) -> Result<bool, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let options = eon_syntax::FormatOptions::default();
    let formatted = eon_syntax::reformat(&content, &options)?;

    let needs_formatting = content != formatted;

    if needs_formatting && !check_mode {
        fs::write(path, formatted)?;
    }

    Ok(needs_formatting)
}
