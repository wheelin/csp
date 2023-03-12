use core::panic;
use std::{path::Path, error::Error, io::Read};

use clap::{Parser};
use serde::{Deserialize};
use tabled::{Tabled, Table, Style};

const CONFIG_PATH: &'static str = "/home/<user>/.config/csp";

#[derive(Deserialize, Clone, Tabled)]
struct SheetItem {
    shortcut: String,
    description: String,
    #[tabled(skip)]
    keywords: Vec<String>,
}

#[derive(Deserialize)]
struct Sheet {
    items: Vec<SheetItem>,
}

/// Search through shortcuts and commands of certain programs
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Software whose shortcuts to look through
    #[arg(short, long)]
    exec: String,
    /// Filter search results for a given software
    #[arg(short, long)]
    filter: Option<String>,
    /// Search in /home/$USER/.config/csp directory for cheatsheet files
    #[arg(short, long)]
    confpath: bool,
    // verbosity flag
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> std::result::Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.verbose {
        println!("Arguments: {:?}", args);
    }

    let user = match std::env::var_os("USER") {
        Some(u) => u.into_string().expect("Directory name /home/$USER/.config/ not valid."),
        None => {
            panic!("No user defined by the env variable $USER.");
        }
    };

    let config_path_str = CONFIG_PATH.replace("<user>", &user);
    let config_path = Path::new(&config_path_str);
    if args.confpath {

        if !config_path.exists() {
            println!("Configuration path for csp doesn't exist. Creating...");

            std::fs::create_dir(&config_path)?;

            for entry in walkdir::WalkDir::new("sheets") {
                match entry {
                    Ok(e) => {
                        if !e.file_type().is_dir() {
                            println!("Copying {:?} to {:?}", e.file_name(), config_path);
                            std::fs::copy(e.path(), config_path.join(e.file_name()))?;
                        }
                    },
                    Err(_) => (),
                }
            }
        }
    }

    let cs_path = if args.confpath { config_path } else { Path::new("sheets")};

    for entry in walkdir::WalkDir::new(cs_path) {
        match entry {
            Ok(e) => {
                if !e.file_type().is_dir() {
                    let file_name = match e.path().to_str() {
                        Some(f) => f,
                        None => panic!("Cannot read file name {:?} properly...", e.file_name()),
                    };
                    if file_name.contains(&args.exec) {
                        let mut file = std::fs::OpenOptions::new().read(true).open(file_name)?;
                        let mut file_content = String::new();
                        let size = file.read_to_string(&mut file_content)?;
                        if size == 0 {
                            panic!("File {} is empty...", file_name);
                        }
                        let data: Sheet = serde_json::from_str(&file_content)?;
                        let mut filtered: Vec<SheetItem> = Vec::new();

                        if let Some(filter) = args.filter.clone() {
                            for item in data.items {
                                let keywords = item.keywords.join(",");
                                if item.description.contains(&filter) || keywords.contains(&filter) {
                                    filtered.push(item.clone());
                                }
                            }
                        } else {
                            for item in data.items {
                                filtered.push(item.clone());
                            }
                        }

                        let mut table = Table::new(&filtered);
                        table.with(Style::rounded());
                        println!("{}", table.to_string());
                    }
                }
            },
            Err(_) => (),
        }
    }

    Ok(())
}
