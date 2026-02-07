use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

use fsid::constants::get_file_type_name;
use fsid::storage::FsidStorage;
use fsid::{is_short_format, minimal, short, standard};

/// FSID - File System Identifier
#[derive(Parser)]
#[command(name = "fsid")]
#[command(about = "FSID - A self-contained identifier for files and directories")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert a file path to its FSID
    To {
        /// Path to the file or directory
        path: String,
        /// Use short format (base36, ~45% shorter)
        #[arg(short, long)]
        short: bool,
        /// Use minimal 13-digit format (requires --storage)
        #[arg(short, long)]
        min: bool,
        /// Path to storage JSON file (required for --min, optional for others)
        #[arg(long)]
        storage: Option<PathBuf>,
    },
    /// Convert an FSID back to its file path
    From {
        /// The FSID
        fsid: String,
        /// Path to storage JSON file (required for --min FSIDs)
        #[arg(long)]
        storage: Option<PathBuf>,
    },
    /// Show detailed information about an FSID
    Info {
        /// The FSID
        fsid: String,
        /// Path to storage JSON file
        #[arg(long)]
        storage: Option<PathBuf>,
    },
    /// List all registered FSIDs
    List {
        /// Path to storage JSON file
        #[arg(long)]
        storage: PathBuf,
    },
}

/// Check if FSID is minimal format (exactly 13 digits)
fn is_minimal_format(fsid: &str) -> bool {
    fsid.len() == 13 && fsid.chars().all(|c| c.is_ascii_digit())
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::To { path, short: use_short, min, storage } => {
            // Validate --min requires --storage
            if min && storage.is_none() {
                eprintln!("Error: --min requires --storage <path>");
                std::process::exit(1);
            }

            let path_obj = Path::new(&path);
            if !path_obj.exists() && !path_obj.is_symlink() {
                eprintln!("Error: Path does not exist: {}", path);
                std::process::exit(1);
            }

            // Generate FSID based on format
            let fsid = if min {
                minimal::generate(&path)
            } else if use_short {
                short::generate(&path)
            } else {
                standard::generate(&path)
            };

            // Store if storage path provided
            if let Some(ref storage_path) = storage {
                let mut store = FsidStorage::load(storage_path);
                let canonical = std::fs::canonicalize(path_obj)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| path.clone());
                store.insert(fsid.clone(), canonical, storage_path);
            }

            println!("{}", fsid);
        }

        Commands::From { fsid, storage } => {
            let fsid_lower = fsid.to_lowercase();
            
            // Check if minimal format (requires storage)
            if is_minimal_format(&fsid) {
                if !minimal::validate(&fsid) {
                    eprintln!("Error: Invalid FSID (check digit mismatch)");
                    std::process::exit(1);
                }
                match storage {
                    Some(ref storage_path) => {
                        let store = FsidStorage::load(storage_path);
                        match store.get(&fsid) {
                            Some(path) => println!("{}", path),
                            None => {
                                eprintln!("Error: FSID not found in storage");
                                std::process::exit(1);
                            }
                        }
                    }
                    None => {
                        eprintln!("Error: Minimal FSID requires --storage <path>");
                        std::process::exit(1);
                    }
                }
            } else if is_short_format(&fsid_lower) {
                // Try storage first, then decode
                if let Some(ref storage_path) = storage {
                    let store = FsidStorage::load(storage_path);
                    if let Some(path) = store.get(&fsid_lower) {
                        println!("{}", path);
                        return;
                    }
                }
                if !short::validate(&fsid_lower) {
                    eprintln!("Error: Invalid FSID (check digit mismatch)");
                    std::process::exit(1);
                }
                match short::decode(&fsid_lower) {
                    Some(path) => println!("{}", path),
                    None => {
                        eprintln!("Error: Failed to decode FSID");
                        std::process::exit(1);
                    }
                }
            } else {
                // Standard format - try storage first, then decode
                if let Some(ref storage_path) = storage {
                    let store = FsidStorage::load(storage_path);
                    if let Some(path) = store.get(&fsid) {
                        println!("{}", path);
                        return;
                    }
                }
                if !standard::validate(&fsid) {
                    eprintln!("Error: Invalid FSID (check digit mismatch)");
                    std::process::exit(1);
                }
                match standard::decode(&fsid) {
                    Some(path) => println!("{}", path),
                    None => {
                        eprintln!("Error: Failed to decode FSID");
                        std::process::exit(1);
                    }
                }
            }
        }

        Commands::Info { fsid, storage } => {
            let fsid_lower = fsid.to_lowercase();
            let stored_path = storage.as_ref().and_then(|p| {
                let store = FsidStorage::load(p);
                store.get(&fsid).or_else(|| store.get(&fsid_lower)).cloned()
            });

            if is_minimal_format(&fsid) {
                print_minimal_info(&fsid, stored_path.as_deref());
            } else if is_short_format(&fsid_lower) {
                print_short_info(&fsid_lower, stored_path.as_deref());
            } else {
                print_standard_info(&fsid, stored_path.as_deref());
            }
        }

        Commands::List { storage } => {
            let store = FsidStorage::load(&storage);
            if store.mappings.is_empty() {
                println!("No FSIDs registered.");
            } else {
                println!("┌───────────────────────────┬────────────────────────────────────────────┐");
                println!("│           FSID            │ Path                                       │");
                println!("├───────────────────────────┼────────────────────────────────────────────┤");
                for (fsid, path) in &store.mappings {
                    let short_path = if path.len() > 40 {
                        format!("...{}", &path[path.len()-37..])
                    } else {
                        path.clone()
                    };
                    println!("│ {:25} │ {:42} │", fsid, short_path);
                }
                println!("└───────────────────────────┴────────────────────────────────────────────┘");
            }
        }
    }
}

fn print_minimal_info(fsid: &str, stored_path: Option<&str>) {
    match minimal::parse(fsid) {
        Some(info) => {
            let valid_mark = if info.valid { "✓" } else { "✗" };
            let path_display = stored_path.unwrap_or("(not in storage)");

            println!("┌─────────────────────────────────────────┐");
            println!("│        FSID Information (minimal)       │");
            println!("├─────────────────────────────────────────┤");
            println!("│ FSID:   {}                     │", fsid);
            println!("├─────────────────────────────────────────┤");
            println!("│ Prefix: {:8} ({})                 │", info.prefix_path, info.prefix_code);
            println!("│ Type:   {:14} ({})          │", get_file_type_name(info.file_type_code), info.file_type_code);
            println!("│ Mode:   {:10} ({:o})              │", info.mode_symbolic, info.mode_octal);
            println!("│ Hash:   {}                      │", info.hash);
            println!("│ Check:  {}  Valid: {}                    │", info.check_digit, valid_mark);
            println!("├─────────────────────────────────────────┤");
            println!("│ Path:   {:<31} │", &path_display[..path_display.len().min(31)]);
            if path_display.len() > 31 {
                println!("│         {:<31} │", &path_display[31..path_display.len().min(62)]);
            }
            println!("└─────────────────────────────────────────┘");
        }
        None => {
            eprintln!("Error: Invalid FSID format");
            std::process::exit(1);
        }
    }
}

fn print_short_info(fsid: &str, stored_path: Option<&str>) {
    if fsid.len() < 5 {
        eprintln!("Error: FSID too short");
        std::process::exit(1);
    }
    let prefix_code = &fsid[0..2];
    let type_mode_code = fsid.chars().nth(2).unwrap_or('z');
    let check = fsid.chars().last().unwrap();
    let valid = short::validate(fsid);
    let prefix_path = short::get_prefix_path(prefix_code).unwrap_or("/");
    let (file_type, mode, type_desc) = short::get_type_mode_info(type_mode_code);
    let decoded_path = short::decode(fsid).unwrap_or_else(|| "(decode error)".to_string());
    let path_display = stored_path.unwrap_or(&decoded_path);
    let valid_mark = if valid { "✓" } else { "✗" };

    println!("┌──────────────────────────────────────────────┐");
    println!("│         FSID Information (short)             │");
    println!("├──────────────────────────────────────────────┤");
    println!("│ FSID:   {:<36} │", fsid);
    println!("├──────────────────────────────────────────────┤");
    println!("│ Prefix: {:<8} ({})                       │", prefix_path, prefix_code);
    println!("│ Type:   {:<14} ({})                │", get_file_type_name(file_type), type_desc);
    println!("│ Mode:   {:o}                                  │", mode);
    println!("│ Check:  {}  Valid: {}                        │", check, valid_mark);
    println!("├──────────────────────────────────────────────┤");
    println!("│ Path:   {:<36} │", &path_display[..path_display.len().min(36)]);
    if path_display.len() > 36 {
        println!("│         {:<36} │", &path_display[36..path_display.len().min(72)]);
    }
    println!("└──────────────────────────────────────────────┘");
}

fn print_standard_info(fsid: &str, stored_path: Option<&str>) {
    if fsid.len() < 8 || !fsid.chars().all(|c| c.is_ascii_digit()) {
        eprintln!("Error: Invalid FSID format");
        std::process::exit(1);
    }
    let prefix_code = &fsid[0..2];
    let file_type_code = fsid[2..3].parse::<u8>().unwrap_or(0);
    let mode_code = fsid[3..4].parse::<u8>().unwrap_or(9);
    let check = &fsid[fsid.len()-2..];
    let valid = standard::validate(fsid);
    let prefix_path = standard::get_prefix_path(prefix_code).unwrap_or("/");
    let (mode_symbolic, mode_octal) = standard::get_mode_description(mode_code);
    let decoded_path = standard::decode(fsid).unwrap_or_else(|| "(decode error)".to_string());
    let path_display = stored_path.unwrap_or(&decoded_path);
    let valid_mark = if valid { "✓" } else { "✗" };

    println!("┌────────────────────────────────────────────────────┐");
    println!("│            FSID Information (standard)             │");
    println!("├────────────────────────────────────────────────────┤");
    println!("│ FSID:    {:<41} │", &fsid[..fsid.len().min(41)]);
    if fsid.len() > 41 {
        println!("│          {:<41} │", &fsid[41..]);
    }
    println!("├────────────────────────────────────────────────────┤");
    println!("│ Prefix:  {:10} ({})                          │", prefix_path, prefix_code);
    println!("│ Type:    {:16} ({})                    │", get_file_type_name(file_type_code), file_type_code);
    println!("│ Mode:    {:10} ({:o})                        │", mode_symbolic, mode_octal);
    println!("│ Check:   {:2}                                       │", check);
    println!("├────────────────────────────────────────────────────┤");
    println!("│ Valid:   {}                                        │", valid_mark);
    println!("│ Path:    {:<41} │", &path_display[..path_display.len().min(41)]);
    if path_display.len() > 41 {
        println!("│          {:<41} │", &path_display[41..]);
    }
    println!("└────────────────────────────────────────────────────┘");
}
