use clap::{Parser, Subcommand};
use std::path::Path;

use fsid::constants::get_file_type_name;
use fsid::{is_short_format, short, standard};

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
    },
    /// Convert an FSID back to its file path
    From {
        /// The FSID
        fsid: String,
    },
    /// Show detailed information about an FSID
    Info {
        /// The FSID
        fsid: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::To { path, short: use_short } => {
            let path_obj = Path::new(&path);
            if !path_obj.exists() && !path_obj.is_symlink() {
                eprintln!("Error: Path does not exist: {}", path);
                std::process::exit(1);
            }
            if use_short {
                println!("{}", short::generate(&path));
            } else {
                println!("{}", standard::generate(&path));
            }
        }

        Commands::From { fsid } => {
            let fsid_lower = fsid.to_lowercase();
            if is_short_format(&fsid_lower) {
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

        Commands::Info { fsid } => {
            let fsid_lower = fsid.to_lowercase();
            if is_short_format(&fsid_lower) {
                print_short_info(&fsid_lower);
            } else {
                print_standard_info(&fsid);
            }
        }
    }
}

fn print_short_info(fsid: &str) {
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
    println!("│ Path:   {:<36} │", &decoded_path[..decoded_path.len().min(36)]);
    if decoded_path.len() > 36 {
        println!("│         {:<36} │", &decoded_path[36..decoded_path.len().min(72)]);
    }
    println!("└──────────────────────────────────────────────┘");
}

fn print_standard_info(fsid: &str) {
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
    println!("│ Path:    {:<41} │", &decoded_path[..decoded_path.len().min(41)]);
    if decoded_path.len() > 41 {
        println!("│          {:<41} │", &decoded_path[41..]);
    }
    println!("└────────────────────────────────────────────────────┘");
}
