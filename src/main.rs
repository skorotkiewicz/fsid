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
            let w = 50; // inner width

            println!("┌{}┐", "─".repeat(w));
            println!("│{:^w$}│", "FSID Information (minimal)");
            println!("├{}┤", "─".repeat(w));
            println!("│ FSID:   {:<w2$} │", fsid, w2 = w - 10);
            println!("├{}┤", "─".repeat(w));
            let prefix_line = format!("{} ({})", info.prefix_path, info.prefix_code);
            let type_line = format!("{} ({})", get_file_type_name(info.file_type_code), info.file_type_code);
            let mode_line = format!("{} ({:o})", info.mode_symbolic, info.mode_octal);
            let check_line = format!("{}   Valid: {}", info.check_digit, valid_mark);
            println!("│ Prefix: {:<w2$} │", prefix_line, w2 = w - 10);
            println!("│ Type:   {:<w2$} │", type_line, w2 = w - 10);
            println!("│ Mode:   {:<w2$} │", mode_line, w2 = w - 10);
            println!("│ Hash:   {:<w2$} │", info.hash, w2 = w - 10);
            println!("│ Check:  {:<w2$} │", check_line, w2 = w - 10);
            println!("├{}┤", "─".repeat(w));
            println!("│ Path:   {:<w2$} │", &path_display[..path_display.len().min(w - 10)], w2 = w - 10);
            if path_display.len() > w - 10 {
                println!("│         {:<w2$} │", &path_display[w - 10..path_display.len().min((w - 10) * 2)], w2 = w - 10);
            }
            println!("└{}┘", "─".repeat(w));
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
    let w = 50;

    println!("┌{}┐", "─".repeat(w));
    println!("│{:^w$}│", "FSID Information (short)");
    println!("├{}┤", "─".repeat(w));
    println!("│ FSID:   {:<w2$} │", fsid, w2 = w - 10);
    println!("├{}┤", "─".repeat(w));
    let prefix_line = format!("{} ({})", prefix_path, prefix_code);
    let type_line = format!("{} ({})", get_file_type_name(file_type), type_desc);
    let mode_line = format!("{:o}", mode);
    let check_line = format!("{}   Valid: {}", check, valid_mark);
    println!("│ Prefix: {:<w2$} │", prefix_line, w2 = w - 10);
    println!("│ Type:   {:<w2$} │", type_line, w2 = w - 10);
    println!("│ Mode:   {:<w2$} │", mode_line, w2 = w - 10);
    println!("│ Check:  {:<w2$} │", check_line, w2 = w - 10);
    println!("├{}┤", "─".repeat(w));
    println!("│ Path:   {:<w2$} │", &path_display[..path_display.len().min(w - 10)], w2 = w - 10);
    if path_display.len() > w - 10 {
        println!("│         {:<w2$} │", &path_display[w - 10..path_display.len().min((w - 10) * 2)], w2 = w - 10);
    }
    println!("└{}┘", "─".repeat(w));
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
    let w = 50;

    println!("┌{}┐", "─".repeat(w));
    println!("│{:^w$}│", "FSID Information (standard)");
    println!("├{}┤", "─".repeat(w));
    println!("│ FSID:   {:<w2$} │", &fsid[..fsid.len().min(w - 10)], w2 = w - 10);
    if fsid.len() > w - 10 {
        println!("│         {:<w2$} │", &fsid[w - 10..fsid.len().min((w - 10) * 2)], w2 = w - 10);
    }
    println!("├{}┤", "─".repeat(w));
    let prefix_line = format!("{} ({})", prefix_path, prefix_code);
    let type_line = format!("{} ({})", get_file_type_name(file_type_code), file_type_code);
    let mode_line = format!("{} ({:o})", mode_symbolic, mode_octal);
    let check_line = format!("{}   Valid: {}", check, valid_mark);
    println!("│ Prefix: {:<w2$} │", prefix_line, w2 = w - 10);
    println!("│ Type:   {:<w2$} │", type_line, w2 = w - 10);
    println!("│ Mode:   {:<w2$} │", mode_line, w2 = w - 10);
    println!("│ Check:  {:<w2$} │", check_line, w2 = w - 10);
    println!("├{}┤", "─".repeat(w));
    println!("│ Path:   {:<w2$} │", &path_display[..path_display.len().min(w - 10)], w2 = w - 10);
    if path_display.len() > w - 10 {
        println!("│         {:<w2$} │", &path_display[w - 10..path_display.len().min((w - 10) * 2)], w2 = w - 10);
    }
    println!("└{}┘", "─".repeat(w));
}
