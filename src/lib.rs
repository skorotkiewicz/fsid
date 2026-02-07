//! FSID - File System Identifier
//!
//! A self-contained identifier for files and directories.

pub mod constants;
pub mod short;
pub mod standard;

/// Check if FSID is in short format (contains lowercase letters)
pub fn is_short_format(fsid: &str) -> bool {
    fsid.chars().any(|c| c.is_ascii_lowercase())
}
