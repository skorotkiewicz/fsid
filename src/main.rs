use clap::{Parser, Subcommand};
use std::fs;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::path::Path;

/// FSID - File System Identifier (like ISBN for files)
#[derive(Parser)]
#[command(name = "fsid")]
#[command(about = "FSID - A self-contained identifier for files, like ISBN for books")]
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
        /// The FSID (variable length, typically 20-40 digits)
        fsid: String,
    },
    /// Show detailed information about an FSID
    Info {
        /// The FSID
        fsid: String,
    },
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

/// Get the directory prefix code and remaining path
fn get_prefix_and_remainder(path: &str) -> (&'static str, &str) {
    // Find the most specific (longest) matching prefix
    let mut best_match = ("00", "/", "");
    for &(code, prefix) in PREFIXES.iter().skip(1) {
        if path.starts_with(prefix) && prefix.len() > best_match.1.len() {
            best_match = (code, prefix, &path[prefix.len()..]);
        }
    }
    if best_match.0 == "00" && path.starts_with('/') {
        return ("00", &path[1..]);
    }
    (best_match.0, best_match.2)
}

/// Get the prefix path from a code
fn get_prefix_path(code: &str) -> Option<&'static str> {
    PREFIXES.iter().find(|&&(c, _)| c == code).map(|&(_, p)| p)
}

/// Determine file type code (T component)
fn get_file_type_code(path: &Path) -> u8 {
    if path.is_symlink() {
        return 2;
    }
    match fs::metadata(path) {
        Ok(meta) => {
            let ft = meta.file_type();
            if ft.is_dir() { 1 }
            else if ft.is_file() { 0 }
            else if ft.is_socket() { 4 }
            else if ft.is_fifo() { 5 }
            else if ft.is_block_device() { 6 }
            else if ft.is_char_device() { 7 }
            else { 0 }
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
    for &(code, octal, _) in MODES {
        if mode == octal { return code; }
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

/// Encode bytes to decimal string (base256 → base10)
fn bytes_to_decimal(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return "0".to_string();
    }
    
    // Convert bytes to a big integer (base 256)
    // Then convert to decimal string
    let mut result = Vec::new();
    let mut temp = bytes.to_vec();
    
    while !temp.is_empty() && !(temp.len() == 1 && temp[0] == 0) {
        let mut remainder = 0u16;
        let mut new_temp = Vec::new();
        
        for &byte in &temp {
            let value = remainder * 256 + byte as u16;
            let quotient = value / 10;
            remainder = value % 10;
            
            if !new_temp.is_empty() || quotient > 0 {
                new_temp.push(quotient as u8);
            }
        }
        
        result.push((remainder as u8) + b'0');
        temp = new_temp;
    }
    
    if result.is_empty() {
        "0".to_string()
    } else {
        result.reverse();
        String::from_utf8(result).unwrap()
    }
}

/// Decode decimal string to bytes (base10 → base256)
fn decimal_to_bytes(decimal: &str) -> Vec<u8> {
    if decimal == "0" || decimal.is_empty() {
        return Vec::new();
    }
    
    let mut digits: Vec<u8> = decimal.bytes().map(|b| b - b'0').collect();
    let mut result = Vec::new();
    
    while !digits.is_empty() && !(digits.len() == 1 && digits[0] == 0) {
        let mut remainder = 0u16;
        let mut new_digits = Vec::new();
        
        for &digit in &digits {
            let value = remainder * 10 + digit as u16;
            let quotient = value / 256;
            remainder = value % 256;
            
            if !new_digits.is_empty() || quotient > 0 {
                new_digits.push(quotient as u8);
            }
        }
        
        result.push(remainder as u8);
        digits = new_digits;
    }
    
    result.reverse();
    result
}

/// Calculate 2-digit check code
fn calculate_check(digits: &str) -> String {
    let sum: u32 = digits
        .chars()
        .enumerate()
        .filter_map(|(i, c)| {
            c.to_digit(10).map(|d| if i % 2 == 0 { d } else { d * 3 })
        })
        .sum();
    format!("{:02}", sum % 100)
}

/// Validate FSID check digits
fn validate_fsid(fsid: &str) -> bool {
    if fsid.len() < 8 || !fsid.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let check = calculate_check(&fsid[..fsid.len()-2]);
    fsid.ends_with(&check)
}

/// Generate FSID for a path (no storage needed!)
fn generate_fsid(path: &str) -> String {
    let path_obj = Path::new(path);
    let canonical = fs::canonicalize(path_obj)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string());

    let (prefix, remainder) = get_prefix_and_remainder(&canonical);
    let file_type = get_file_type_code(path_obj);
    let mode = get_mode_code(path_obj);
    let encoded_path = bytes_to_decimal(remainder.as_bytes());

    let partial = format!("{}{}{}{}", prefix, file_type, mode, encoded_path);
    let check = calculate_check(&partial);

    format!("{}{}", partial, check)
}

/// Decode FSID back to path (no storage needed!)
fn decode_fsid(fsid: &str) -> Option<String> {
    if fsid.len() < 8 {
        return None;
    }
    
    let prefix_code = &fsid[0..2];
    let encoded_path = &fsid[4..fsid.len()-2];
    
    let prefix_path = get_prefix_path(prefix_code)?;
    let path_bytes = decimal_to_bytes(encoded_path);
    let relative_path = String::from_utf8(path_bytes).ok()?;
    
    if prefix_path == "/" {
        Some(format!("/{}", relative_path))
    } else {
        Some(format!("{}{}", prefix_path, relative_path))
    }
}

/// Parse FSID components
struct FsidInfo {
    prefix_code: String,
    prefix_path: String,
    file_type_code: u8,
    file_type_name: String,
    mode_symbolic: String,
    mode_octal: u32,
    _encoded_path: String,
    decoded_path: String,
    check: String,
    valid: bool,
}

fn parse_fsid(fsid: &str) -> Option<FsidInfo> {
    if fsid.len() < 8 {
        return None;
    }

    let prefix_code = &fsid[0..2];
    let file_type_code = fsid[2..3].parse::<u8>().ok()?;
    let mode_code = fsid[3..4].parse::<u8>().ok()?;
    let encoded_path = &fsid[4..fsid.len()-2];
    let check = &fsid[fsid.len()-2..];

    let prefix_path = get_prefix_path(prefix_code).unwrap_or("/");
    let (mode_symbolic, mode_octal) = get_mode_description(mode_code);
    let decoded_path = decode_fsid(fsid).unwrap_or_else(|| "(decode error)".to_string());

    Some(FsidInfo {
        prefix_code: prefix_code.to_string(),
        prefix_path: prefix_path.to_string(),
        file_type_code,
        file_type_name: get_file_type_name(file_type_code).to_string(),
        mode_symbolic: mode_symbolic.to_string(),
        mode_octal,
        _encoded_path: encoded_path.to_string(),
        decoded_path,
        check: check.to_string(),
        valid: validate_fsid(fsid),
    })
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::To { path } => {
            let path_obj = Path::new(&path);
            if !path_obj.exists() && !path_obj.is_symlink() {
                eprintln!("Error: Path does not exist: {}", path);
                std::process::exit(1);
            }
            println!("{}", generate_fsid(&path));
        }

        Commands::From { fsid } => {
            if !validate_fsid(&fsid) {
                eprintln!("Error: Invalid FSID (check digit mismatch)");
                std::process::exit(1);
            }
            match decode_fsid(&fsid) {
                Some(path) => println!("{}", path),
                None => {
                    eprintln!("Error: Failed to decode FSID");
                    std::process::exit(1);
                }
            }
        }

        Commands::Info { fsid } => {
            match parse_fsid(&fsid) {
                Some(info) => {
                    let valid_mark = if info.valid { "✓" } else { "✗" };
                    println!("┌────────────────────────────────────────────────────┐");
                    println!("│                 FSID Information                   │");
                    println!("├────────────────────────────────────────────────────┤");
                    println!("│ FSID:    {:<41} │", &fsid[..fsid.len().min(41)]);
                    if fsid.len() > 41 {
                        println!("│          {:<41} │", &fsid[41..]);
                    }
                    println!("├────────────────────────────────────────────────────┤");
                    println!("│ Prefix:  {:10} ({})                          │", info.prefix_path, info.prefix_code);
                    println!("│ Type:    {:16} ({})                    │", info.file_type_name, info.file_type_code);
                    println!("│ Mode:    {:10} ({:o})                        │", info.mode_symbolic, info.mode_octal);
                    println!("│ Check:   {:2}                                       │", info.check);
                    println!("├────────────────────────────────────────────────────┤");
                    println!("│ Valid:   {}                                        │", valid_mark);
                    println!("│ Path:    {:<41} │", &info.decoded_path[..info.decoded_path.len().min(41)]);
                    if info.decoded_path.len() > 41 {
                        println!("│          {:<41} │", &info.decoded_path[41..]);
                    }
                    println!("└────────────────────────────────────────────────────┘");
                }
                None => {
                    eprintln!("Error: Invalid FSID format");
                    std::process::exit(1);
                }
            }
        }
    }
}
