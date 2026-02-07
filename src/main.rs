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

// ============================================================================
// SHARED CONSTANTS
// ============================================================================

/// Base36 alphabet (0-9, a-z) for short format
const BASE36: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";

/// Standard directory prefix mappings (decimal, 00-25)
const PREFIXES_STD: &[(&str, &str)] = &[
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

/// Short directory prefix mappings (base36, more options)
const PREFIXES_SHORT: &[(&str, &str)] = &[
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
    ("0a", "/boot/"),
    ("0b", "/dev/"),
    ("0c", "/proc/"),
    ("0d", "/sys/"),
    ("0e", "/run/"),
    ("0f", "/mnt/"),
    ("0g", "/media/"),
    ("0h", "/root/"),
    ("0i", "/sbin/"),
    ("0j", "/usr/bin/"),
    ("0k", "/usr/lib/"),
    ("0l", "/usr/share/"),
    ("0m", "/usr/local/"),
    ("0n", "/var/log/"),
    ("0o", "/var/lib/"),
    ("0p", "/var/cache/"),
    ("0q", "/usr/lib64/"),
    ("0r", "/usr/include/"),
    ("0s", "/var/run/"),
    ("0t", "/var/tmp/"),
    ("0u", "/usr/local/bin/"),
    ("0v", "/usr/local/lib/"),
];

/// Permission mode mappings for standard format
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

/// Combined type + mode codes for short format
const TYPE_MODES: &[(char, u8, u32, &str)] = &[
    ('0', 0, 0o644, "file:644"),
    ('1', 0, 0o755, "file:755"),
    ('2', 0, 0o600, "file:600"),
    ('3', 0, 0o700, "file:700"),
    ('4', 0, 0o664, "file:664"),
    ('5', 0, 0o775, "file:775"),
    ('6', 0, 0o666, "file:666"),
    ('7', 0, 0o777, "file:777"),
    ('8', 1, 0o755, "dir:755"),
    ('9', 1, 0o700, "dir:700"),
    ('a', 1, 0o775, "dir:775"),
    ('b', 1, 0o777, "dir:777"),
    ('c', 2, 0o777, "symlink"),
    ('d', 4, 0o755, "socket"),
    ('e', 5, 0o644, "fifo"),
    ('f', 6, 0o660, "block"),
    ('g', 7, 0o666, "char"),
    ('z', 0, 0, "other"),
];

// ============================================================================
// STANDARD FORMAT (Base10)
// ============================================================================

mod standard {
    use super::*;

    pub fn get_prefix_and_remainder(path: &str) -> (&'static str, &str) {
        let mut best_match = ("00", "/", "");
        for &(code, prefix) in PREFIXES_STD.iter().skip(1) {
            if path.starts_with(prefix) && prefix.len() > best_match.1.len() {
                best_match = (code, prefix, &path[prefix.len()..]);
            }
        }
        if best_match.0 == "00" && path.starts_with('/') {
            return ("00", &path[1..]);
        }
        (best_match.0, best_match.2)
    }

    pub fn get_prefix_path(code: &str) -> Option<&'static str> {
        PREFIXES_STD.iter().find(|&&(c, _)| c == code).map(|&(_, p)| p)
    }

    pub fn get_file_type_code(path: &Path) -> u8 {
        if path.is_symlink() { return 2; }
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

    pub fn get_mode_code(path: &Path) -> u8 {
        let mode = match fs::metadata(path) {
            Ok(meta) => meta.permissions().mode() & 0o777,
            Err(_) => return 9,
        };
        for &(code, octal, _) in MODES {
            if mode == octal { return code; }
        }
        9
    }

    pub fn bytes_to_decimal(bytes: &[u8]) -> String {
        if bytes.is_empty() { return "0".to_string(); }
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
        if result.is_empty() { "0".to_string() }
        else { result.reverse(); String::from_utf8(result).unwrap() }
    }

    pub fn decimal_to_bytes(decimal: &str) -> Vec<u8> {
        if decimal == "0" || decimal.is_empty() { return Vec::new(); }
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

    pub fn calculate_check(digits: &str) -> String {
        let sum: u32 = digits.chars().enumerate()
            .filter_map(|(i, c)| c.to_digit(10).map(|d| if i % 2 == 0 { d } else { d * 3 }))
            .sum();
        format!("{:02}", sum % 100)
    }

    pub fn validate(fsid: &str) -> bool {
        if fsid.len() < 8 || !fsid.chars().all(|c| c.is_ascii_digit()) { return false; }
        let check = calculate_check(&fsid[..fsid.len()-2]);
        fsid.ends_with(&check)
    }

    pub fn generate(path: &str) -> String {
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

    pub fn decode(fsid: &str) -> Option<String> {
        if fsid.len() < 8 { return None; }
        let prefix_code = &fsid[0..2];
        let encoded_path = &fsid[4..fsid.len()-2];
        let prefix_path = get_prefix_path(prefix_code)?;
        let path_bytes = decimal_to_bytes(encoded_path);
        let relative_path = String::from_utf8(path_bytes).ok()?;
        if prefix_path == "/" { Some(format!("/{}", relative_path)) }
        else { Some(format!("{}{}", prefix_path, relative_path)) }
    }

    pub fn get_mode_description(code: u8) -> (&'static str, u32) {
        if code < 9 {
            if let Some(&(_, octal, symbolic)) = MODES.iter().find(|&&(c, _, _)| c == code) {
                return (symbolic, octal);
            }
        }
        ("custom", 0)
    }
}

// ============================================================================
// SHORT FORMAT (Base36)
// ============================================================================

mod short {
    use super::*;

    pub fn get_prefix_and_remainder(path: &str) -> (&'static str, &str) {
        let mut best_match = ("00", "/", "");
        for &(code, prefix) in PREFIXES_SHORT.iter().skip(1) {
            if path.starts_with(prefix) && prefix.len() > best_match.1.len() {
                best_match = (code, prefix, &path[prefix.len()..]);
            }
        }
        if best_match.0 == "00" && path.starts_with('/') {
            return ("00", &path[1..]);
        }
        (best_match.0, best_match.2)
    }

    pub fn get_prefix_path(code: &str) -> Option<&'static str> {
        PREFIXES_SHORT.iter().find(|&&(c, _)| c == code).map(|&(_, p)| p)
    }

    pub fn get_type_mode_code(path: &Path) -> char {
        let is_symlink = path.is_symlink();
        let (file_type, mode) = match fs::metadata(path) {
            Ok(meta) => {
                let ft = meta.file_type();
                let m = meta.permissions().mode() & 0o777;
                let t = if is_symlink { 2 }
                    else if ft.is_dir() { 1 }
                    else if ft.is_file() { 0 }
                    else if ft.is_socket() { 4 }
                    else if ft.is_fifo() { 5 }
                    else if ft.is_block_device() { 6 }
                    else if ft.is_char_device() { 7 }
                    else { 0 };
                (t, m)
            }
            Err(_) => if is_symlink { (2, 0o777) } else { (0, 0o644) }
        };
        for &(code, t, m, _) in TYPE_MODES {
            if t == file_type && m == mode { return code; }
        }
        for &(code, t, _, _) in TYPE_MODES {
            if t == file_type { return code; }
        }
        'z'
    }

    pub fn bytes_to_base36(bytes: &[u8]) -> String {
        if bytes.is_empty() { return "0".to_string(); }
        let mut result = Vec::new();
        let mut temp = bytes.to_vec();
        while !temp.is_empty() && !(temp.len() == 1 && temp[0] == 0) {
            let mut remainder = 0u16;
            let mut new_temp = Vec::new();
            for &byte in &temp {
                let value = remainder * 256 + byte as u16;
                let quotient = value / 36;
                remainder = value % 36;
                if !new_temp.is_empty() || quotient > 0 {
                    new_temp.push(quotient as u8);
                }
            }
            result.push(BASE36[remainder as usize]);
            temp = new_temp;
        }
        if result.is_empty() { "0".to_string() }
        else { result.reverse(); String::from_utf8(result).unwrap() }
    }

    pub fn base36_to_bytes(s: &str) -> Vec<u8> {
        if s == "0" || s.is_empty() { return Vec::new(); }
        let mut digits: Vec<u8> = s.bytes().map(|b| {
            if b >= b'0' && b <= b'9' { b - b'0' }
            else if b >= b'a' && b <= b'z' { b - b'a' + 10 }
            else { 0 }
        }).collect();
        let mut result = Vec::new();
        while !digits.is_empty() && !(digits.len() == 1 && digits[0] == 0) {
            let mut remainder = 0u16;
            let mut new_digits = Vec::new();
            for &digit in &digits {
                let value = remainder * 36 + digit as u16;
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

    pub fn calculate_check(s: &str) -> char {
        let sum: u32 = s.bytes().enumerate().map(|(i, b)| {
            let val = if b >= b'0' && b <= b'9' { b - b'0' }
                      else if b >= b'a' && b <= b'z' { b - b'a' + 10 }
                      else { 0 };
            if i % 2 == 0 { val as u32 } else { (val as u32) * 3 }
        }).sum();
        BASE36[(sum % 36) as usize] as char
    }

    pub fn validate(fsid: &str) -> bool {
        if fsid.len() < 5 { return false; }
        let check = calculate_check(&fsid[..fsid.len()-1]);
        fsid.chars().last() == Some(check)
    }

    pub fn generate(path: &str) -> String {
        let path_obj = Path::new(path);
        let canonical = fs::canonicalize(path_obj)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.to_string());
        let (prefix, remainder) = get_prefix_and_remainder(&canonical);
        let type_mode = get_type_mode_code(path_obj);
        let encoded_path = bytes_to_base36(remainder.as_bytes());
        let partial = format!("{}{}{}", prefix, type_mode, encoded_path);
        let check = calculate_check(&partial);
        format!("{}{}", partial, check)
    }

    pub fn decode(fsid: &str) -> Option<String> {
        if fsid.len() < 5 { return None; }
        let prefix_code = &fsid[0..2];
        let encoded_path = &fsid[3..fsid.len()-1];
        let prefix_path = get_prefix_path(prefix_code)?;
        let path_bytes = base36_to_bytes(encoded_path);
        let relative_path = String::from_utf8(path_bytes).ok()?;
        if prefix_path == "/" { Some(format!("/{}", relative_path)) }
        else { Some(format!("{}{}", prefix_path, relative_path)) }
    }

    pub fn get_type_mode_info(code: char) -> (u8, u32, &'static str) {
        for &(c, t, m, desc) in TYPE_MODES {
            if c == code { return (t, m, desc); }
        }
        (0, 0, "unknown")
    }
}

// ============================================================================
// AUTO-DETECT FORMAT
// ============================================================================

fn is_short_format(fsid: &str) -> bool {
    // Short format uses base36 (has letters a-z)
    fsid.chars().any(|c| c.is_ascii_lowercase())
}

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

// ============================================================================
// MAIN
// ============================================================================

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::To { path, short } => {
            let path_obj = Path::new(&path);
            if !path_obj.exists() && !path_obj.is_symlink() {
                eprintln!("Error: Path does not exist: {}", path);
                std::process::exit(1);
            }
            if short {
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
                // Short format info
                if fsid_lower.len() < 5 {
                    eprintln!("Error: FSID too short");
                    std::process::exit(1);
                }
                let prefix_code = &fsid_lower[0..2];
                let type_mode_code = fsid_lower.chars().nth(2).unwrap_or('z');
                let check = fsid_lower.chars().last().unwrap();
                let valid = short::validate(&fsid_lower);
                let prefix_path = short::get_prefix_path(prefix_code).unwrap_or("/");
                let (file_type, mode, type_desc) = short::get_type_mode_info(type_mode_code);
                let decoded_path = short::decode(&fsid_lower).unwrap_or_else(|| "(decode error)".to_string());
                let valid_mark = if valid { "✓" } else { "✗" };

                println!("┌──────────────────────────────────────────────┐");
                println!("│         FSID Information (short)             │");
                println!("├──────────────────────────────────────────────┤");
                println!("│ FSID:   {:<36} │", &fsid_lower);
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
            } else {
                // Standard format info
                if fsid.len() < 8 || !fsid.chars().all(|c| c.is_ascii_digit()) {
                    eprintln!("Error: Invalid FSID format");
                    std::process::exit(1);
                }
                let prefix_code = &fsid[0..2];
                let file_type_code = fsid[2..3].parse::<u8>().unwrap_or(0);
                let mode_code = fsid[3..4].parse::<u8>().unwrap_or(9);
                let check = &fsid[fsid.len()-2..];
                let valid = standard::validate(&fsid);
                let prefix_path = standard::get_prefix_path(prefix_code).unwrap_or("/");
                let (mode_symbolic, mode_octal) = standard::get_mode_description(mode_code);
                let decoded_path = standard::decode(&fsid).unwrap_or_else(|| "(decode error)".to_string());
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
        }
    }
}
