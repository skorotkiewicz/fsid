//! Shared constants for FSID encoding

/// Base36 alphabet (0-9, a-z) for short format
pub const BASE36: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";

/// Standard directory prefix mappings (decimal, 00-25)
pub const PREFIXES_STD: &[(&str, &str)] = &[
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
pub const PREFIXES_SHORT: &[(&str, &str)] = &[
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
pub const MODES: &[(u8, u32, &str)] = &[
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
pub const TYPE_MODES: &[(char, u8, u32, &str)] = &[
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

/// Get file type name from code
pub fn get_file_type_name(code: u8) -> &'static str {
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
