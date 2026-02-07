//! Short FSID format (base36, compact)

use std::fs;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::path::Path;

use crate::constants::{BASE36, PREFIXES_SHORT, TYPE_MODES};

/// Get the directory prefix code and remaining path
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

/// Get the prefix path from a code
pub fn get_prefix_path(code: &str) -> Option<&'static str> {
    PREFIXES_SHORT.iter().find(|&&(c, _)| c == code).map(|&(_, p)| p)
}

/// Get combined type+mode code
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

/// Get type+mode info from code
pub fn get_type_mode_info(code: char) -> (u8, u32, &'static str) {
    for &(c, t, m, desc) in TYPE_MODES {
        if c == code { return (t, m, desc); }
    }
    (0, 0, "unknown")
}

/// Encode bytes to base36 string
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

/// Decode base36 string to bytes
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

/// Calculate single-char check (base36)
pub fn calculate_check(s: &str) -> char {
    let sum: u32 = s.bytes().enumerate().map(|(i, b)| {
        let val = if b >= b'0' && b <= b'9' { b - b'0' }
                  else if b >= b'a' && b <= b'z' { b - b'a' + 10 }
                  else { 0 };
        if i % 2 == 0 { val as u32 } else { (val as u32) * 3 }
    }).sum();
    BASE36[(sum % 36) as usize] as char
}

/// Validate FSID check character
pub fn validate(fsid: &str) -> bool {
    if fsid.len() < 5 { return false; }
    let check = calculate_check(&fsid[..fsid.len()-1]);
    fsid.chars().last() == Some(check)
}

/// Generate FSID for a path
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

/// Decode FSID back to path
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
