//! Standard FSID format (base10, numeric only)

use std::fs;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::path::Path;

use crate::constants::{MODES, PREFIXES_STD};

/// Get the directory prefix code and remaining path
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

/// Get the prefix path from a code
pub fn get_prefix_path(code: &str) -> Option<&'static str> {
    PREFIXES_STD.iter().find(|&&(c, _)| c == code).map(|&(_, p)| p)
}

/// Get file type code
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

/// Get permission mode code
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

/// Get permission mode description
pub fn get_mode_description(code: u8) -> (&'static str, u32) {
    if code < 9 {
        if let Some(&(_, octal, symbolic)) = MODES.iter().find(|&&(c, _, _)| c == code) {
            return (symbolic, octal);
        }
    }
    ("custom", 0)
}

/// Encode bytes to decimal string (base256 → base10)
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

/// Decode decimal string to bytes (base10 → base256)
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

/// Calculate 2-digit check code
pub fn calculate_check(digits: &str) -> String {
    let sum: u32 = digits.chars().enumerate()
        .filter_map(|(i, c)| c.to_digit(10).map(|d| if i % 2 == 0 { d } else { d * 3 }))
        .sum();
    format!("{:02}", sum % 100)
}

/// Validate FSID check digits
pub fn validate(fsid: &str) -> bool {
    if fsid.len() < 8 || !fsid.chars().all(|c| c.is_ascii_digit()) { return false; }
    let check = calculate_check(&fsid[..fsid.len()-2]);
    fsid.ends_with(&check)
}

/// Generate FSID for a path
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

/// Decode FSID back to path
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
