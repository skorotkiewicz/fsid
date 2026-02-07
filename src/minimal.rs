//! Minimal FSID format (13 digits, requires storage)

use std::fs;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::path::Path;

use crate::constants::{MODES, PREFIXES_STD};

/// Get the directory prefix code
fn get_prefix_code(path: &str) -> &'static str {
    let mut best_match = ("00", "/");
    for &(code, prefix) in PREFIXES_STD.iter().skip(1) {
        if path.starts_with(prefix) && prefix.len() > best_match.1.len() {
            best_match = (code, prefix);
        }
    }
    best_match.0
}

/// Get the prefix path from a code
pub fn get_prefix_path(code: &str) -> Option<&'static str> {
    PREFIXES_STD.iter().find(|&&(c, _)| c == code).map(|&(_, p)| p)
}

/// Get file type code
fn get_file_type_code(path: &Path) -> u8 {
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

/// Get permission mode code
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
pub fn get_mode_description(code: u8) -> (&'static str, u32) {
    if code < 9 {
        if let Some(&(_, octal, symbolic)) = MODES.iter().find(|&&(c, _, _)| c == code) {
            return (symbolic, octal);
        }
    }
    ("custom", 0)
}

/// Generate 8-digit path hash using djb2 algorithm
fn generate_path_hash(path: &str) -> String {
    let mut hash: u64 = 5381;
    for byte in path.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    format!("{:08}", hash % 100_000_000)
}

/// Calculate check digit (ISBN-13 style)
fn calculate_check_digit(digits: &str) -> char {
    let sum: u32 = digits.chars().enumerate()
        .filter_map(|(i, c)| c.to_digit(10).map(|d| if i % 2 == 0 { d } else { d * 3 }))
        .sum();
    let check = (10 - (sum % 10)) % 10;
    char::from_digit(check, 10).unwrap()
}

/// Validate 13-digit FSID
pub fn validate(fsid: &str) -> bool {
    if fsid.len() != 13 || !fsid.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let check = calculate_check_digit(&fsid[..12]);
    fsid.chars().last() == Some(check)
}

/// Generate 13-digit FSID for a path
pub fn generate(path: &str) -> String {
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

/// Minimal FSID info
pub struct MinInfo {
    pub prefix_code: String,
    pub prefix_path: String,
    pub file_type_code: u8,
    pub mode_code: u8,
    pub mode_symbolic: String,
    pub mode_octal: u32,
    pub hash: String,
    pub check_digit: char,
    pub valid: bool,
}

/// Parse 13-digit FSID
pub fn parse(fsid: &str) -> Option<MinInfo> {
    if fsid.len() != 13 { return None; }

    let prefix_code = &fsid[0..2];
    let file_type_code = fsid[2..3].parse::<u8>().ok()?;
    let mode_code = fsid[3..4].parse::<u8>().ok()?;
    let hash = &fsid[4..12];
    let check_digit = fsid.chars().last()?;

    let prefix_path = get_prefix_path(prefix_code).unwrap_or("/");
    let (mode_symbolic, mode_octal) = get_mode_description(mode_code);

    Some(MinInfo {
        prefix_code: prefix_code.to_string(),
        prefix_path: prefix_path.to_string(),
        file_type_code,
        mode_code,
        mode_symbolic: mode_symbolic.to_string(),
        mode_octal,
        hash: hash.to_string(),
        check_digit,
        valid: validate(fsid),
    })
}
