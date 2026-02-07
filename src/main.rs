use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// FSID - File System Identifier (like ISBN for files)
#[derive(Parser)]
#[command(name = "fsid")]
#[command(about = "FSID - A 13-digit identifier for files, like ISBN for books")]
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
    },
    /// Convert an FSID back to its file path
    From {
        /// The 13-digit FSID
        fsid: String,
    },
    /// Show detailed information about an FSID
    Info {
        /// The 13-digit FSID
        fsid: String,
    },
    /// List all registered FSIDs
    List,
}

/// Directory prefix mappings (PP component)
const PREFIXES: &[(&str, &str)] = &[
    ("00", "/"),
    ("01", "/etc/"),
    ("02", "/bin/"),
    ("03", "/usr/"),
    ("04", "/var/"),
    ("05", "/home/"),
    ("06", "/tmp/"),
    ("07", "/opt/"),
    ("08", "/lib/"),
    ("09", "/srv/"),
    ("10", "/boot/"),
    ("11", "/dev/"),
    ("12", "/proc/"),
    ("13", "/sys/"),
    ("14", "/run/"),
    ("15", "/mnt/"),
    ("16", "/media/"),
    ("17", "/root/"),
    ("18", "/sbin/"),
    ("19", "/usr/bin/"),
    ("20", "/usr/lib/"),
    ("21", "/usr/share/"),
    ("22", "/usr/local/"),
    ("23", "/var/log/"),
    ("24", "/var/lib/"),
    ("25", "/var/cache/"),
];

/// Permission mode mappings (M component)
const MODES: &[(u8, u32, &str)] = &[
    (0, 0o644, "-rw-r--r--"),
    (1, 0o755, "-rwxr-xr-x"),
    (2, 0o600, "-rw-------"),
    (3, 0o700, "-rwx------"),
    (4, 0o664, "-rw-rw-r--"),
    (5, 0o775, "-rwxrwxr-x"),
    (6, 0o755, "drwxr-xr-x"),
    (7, 0o700, "drwx------"),
    (8, 0o777, "lrwxrwxrwx"),
];

/// Storage for FSID -> Path mappings
#[derive(Serialize, Deserialize, Default)]
struct FsidStorage {
    mappings: HashMap<String, String>,
}

impl FsidStorage {
    fn load() -> Self {
        let path = Self::storage_path();
        if path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    fn save(&self) {
        let path = Self::storage_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(self).unwrap();
        let _ = fs::write(path, content);
    }

    fn storage_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("fsid")
            .join("storage.json")
    }

    fn insert(&mut self, fsid: String, path: String) {
        self.mappings.insert(fsid, path);
        self.save();
    }

    fn get(&self, fsid: &str) -> Option<&String> {
        self.mappings.get(fsid)
    }
}

/// Get the directory prefix code for a path
fn get_prefix_code(path: &str) -> &'static str {
    // Find the most specific (longest) matching prefix
    let mut best_match = ("00", "/");
    for &(code, prefix) in PREFIXES.iter().skip(1) {
        if path.starts_with(prefix) && prefix.len() > best_match.1.len() {
            best_match = (code, prefix);
        }
    }
    best_match.0
}

/// Get the prefix path from a code
fn get_prefix_path(code: &str) -> Option<&'static str> {
    PREFIXES.iter().find(|&&(c, _)| c == code).map(|&(_, p)| p)
}

/// Determine file type code (T component)
fn get_file_type_code(path: &Path) -> u8 {
    if path.is_symlink() {
        return 2; // Symlink
    }

    match fs::metadata(path) {
        Ok(meta) => {
            let ft = meta.file_type();
            if ft.is_dir() {
                1
            } else if ft.is_file() {
                0
            } else if ft.is_socket() {
                4
            } else if ft.is_fifo() {
                5
            } else if ft.is_block_device() {
                6
            } else if ft.is_char_device() {
                7
            } else {
                0
            }
        }
        Err(_) => 0,
    }
}

/// Get file type description
fn get_file_type_name(code: u8) -> &'static str {
    match code {
        0 => "Regular file",
        1 => "Directory",
        2 => "Symbolic link",
        3 => "Hard link",
        4 => "Socket",
        5 => "Named pipe (FIFO)",
        6 => "Block device",
        7 => "Character device",
        _ => "Unknown",
    }
}

/// Get permission mode code (M component)
fn get_mode_code(path: &Path) -> u8 {
    let mode = match fs::metadata(path) {
        Ok(meta) => meta.permissions().mode() & 0o777,
        Err(_) => return 9,
    };

    // Find matching mode or return 9 (other)
    for &(code, octal, _) in MODES {
        if mode == octal {
            return code;
        }
    }
    9
}

/// Get permission mode description
fn get_mode_description(code: u8) -> (&'static str, u32) {
    if code < 9 {
        if let Some(&(_, octal, symbolic)) = MODES.iter().find(|&&(c, _, _)| c == code) {
            return (symbolic, octal);
        }
    }
    ("custom", 0)
}

/// Generate 8-digit path hash
fn generate_path_hash(path: &str) -> String {
    // Simple deterministic hash using djb2 algorithm
    let mut hash: u64 = 5381;
    for byte in path.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    // Take last 8 digits
    format!("{:08}", hash % 100_000_000)
}

/// Calculate check digit (similar to ISBN-13)
fn calculate_check_digit(digits: &str) -> char {
    let sum: u32 = digits
        .chars()
        .enumerate()
        .filter_map(|(i, c)| {
            c.to_digit(10).map(|d| {
                if i % 2 == 0 { d } else { d * 3 }
            })
        })
        .sum();

    let check = (10 - (sum % 10)) % 10;
    char::from_digit(check, 10).unwrap()
}

/// Validate FSID check digit
fn validate_fsid(fsid: &str) -> bool {
    if fsid.len() != 13 || !fsid.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let check = calculate_check_digit(&fsid[..12]);
    fsid.chars().last() == Some(check)
}

/// Generate FSID for a path
fn generate_fsid(path: &str) -> String {
    let path_obj = Path::new(path);
    let canonical = fs::canonicalize(path_obj)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string());

    let prefix = get_prefix_code(&canonical);
    let file_type = get_file_type_code(path_obj);
    let mode = get_mode_code(path_obj);
    let hash = generate_path_hash(&canonical);

    let partial = format!("{}{}{}{}", prefix, file_type, mode, hash);
    let check = calculate_check_digit(&partial);

    format!("{}{}", partial, check)
}

/// Parse FSID components
struct FsidInfo {
    prefix_code: String,
    prefix_path: String,
    file_type_code: u8,
    file_type_name: String,
    _mode_code: u8,
    mode_symbolic: String,
    mode_octal: u32,
    hash: String,
    check_digit: char,
    valid: bool,
}

fn parse_fsid(fsid: &str) -> Option<FsidInfo> {
    if fsid.len() != 13 {
        return None;
    }

    let prefix_code = &fsid[0..2];
    let file_type_code = fsid[2..3].parse::<u8>().ok()?;
    let mode_code = fsid[3..4].parse::<u8>().ok()?;
    let hash = &fsid[4..12];
    let check_digit = fsid.chars().last()?;

    let prefix_path = get_prefix_path(prefix_code).unwrap_or("/");
    let (mode_symbolic, mode_octal) = get_mode_description(mode_code);

    Some(FsidInfo {
        prefix_code: prefix_code.to_string(),
        prefix_path: prefix_path.to_string(),
        file_type_code,
        file_type_name: get_file_type_name(file_type_code).to_string(),
        _mode_code: mode_code,
        mode_symbolic: mode_symbolic.to_string(),
        mode_octal,
        hash: hash.to_string(),
        check_digit,
        valid: validate_fsid(fsid),
    })
}

fn main() {
    let cli = Cli::parse();
    let mut storage = FsidStorage::load();

    match cli.command {
        Commands::To { path } => {
            let path_obj = Path::new(&path);
            if !path_obj.exists() && !path_obj.is_symlink() {
                eprintln!("Error: Path does not exist: {}", path);
                std::process::exit(1);
            }

            let fsid = generate_fsid(&path);
            let canonical = fs::canonicalize(path_obj)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.clone());

            storage.insert(fsid.clone(), canonical);
            println!("{}", fsid);
        }

        Commands::From { fsid } => {
            if !validate_fsid(&fsid) {
                eprintln!("Error: Invalid FSID (check digit mismatch or wrong format)");
                std::process::exit(1);
            }

            match storage.get(&fsid) {
                Some(path) => println!("{}", path),
                None => {
                    eprintln!("Error: FSID not found in storage");
                    eprintln!("Hint: Use 'fsid to <path>' to register a file first");
                    std::process::exit(1);
                }
            }
        }

        Commands::Info { fsid } => {
            match parse_fsid(&fsid) {
                Some(info) => {
                    let stored_path = storage.get(&fsid).map(|s| s.as_str()).unwrap_or("(not registered)");
                    let valid_mark = if info.valid { "✓" } else { "✗" };

                    println!("┌─────────────────────────────────────┐");
                    println!("│           FSID Information          │");
                    println!("├─────────────────────────────────────┤");
                    println!("│ FSID:   {}             │", fsid);
                    println!("├─────────────────────────────────────┤");
                    println!("│ Prefix: {} ({})                     │", info.prefix_path, info.prefix_code);
                    println!("│ Type:   {} ({})          │", info.file_type_name, info.file_type_code);
                    println!("│ Mode:   {} ({:o})        │", info.mode_symbolic, info.mode_octal);
                    println!("│ Hash:   {}                  │", info.hash);
                    println!("│ Check:  {}                          │", info.check_digit);
                    println!("├─────────────────────────────────────┤");
                    println!("│ Valid:  {}                          │", valid_mark);
                    println!("│ Path:   {} │", stored_path);
                    println!("└─────────────────────────────────────┘");
                }
                None => {
                    eprintln!("Error: Invalid FSID format (expected 13 digits)");
                    std::process::exit(1);
                }
            }
        }

        Commands::List => {
            if storage.mappings.is_empty() {
                println!("No FSIDs registered yet.");
                println!("Use 'fsid to <path>' to register files.");
            } else {
                println!("┌───────────────┬────────────────────────────────────────────┐");
                println!("│     FSID      │ Path                                       │");
                println!("├───────────────┼────────────────────────────────────────────┤");
                for (fsid, path) in &storage.mappings {
                    let short_path = if path.len() > 40 {
                        format!("...{}", &path[path.len()-37..])
                    } else {
                        path.clone()
                    };
                    println!("│ {} │ {:42} │", fsid, short_path);
                }
                println!("└───────────────┴────────────────────────────────────────────┘");
            }
        }
    }
}
