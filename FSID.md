# FSID - File System Identifier

**FSID** is a 13-character identifier for files and directories, inspired by ISBN for books.

## Structure

```
PP T M HHHHHHHH C
│  │ │ │        └── Check digit (validation)
│  │ │ └─────────── Path hash (8 digits, unique within prefix)
│  │ └───────────── Permission mode (0-9)
│  └─────────────── File type (0-7)
└────────────────── Directory prefix (00-99)
```

**Total: 13 digits** (same as ISBN-13)

---

## Directory Prefixes (PP)

| Prefix | Path |
|--------|------|
| `00` | `/` (root, unclassified) |
| `01` | `/etc/` |
| `02` | `/bin/` |
| `03` | `/usr/` |
| `04` | `/var/` |
| `05` | `/home/` |
| `06` | `/tmp/` |
| `07` | `/opt/` |
| `08` | `/lib/` |
| `09` | `/srv/` |
| `10` | `/boot/` |
| `11` | `/dev/` |
| `12` | `/proc/` |
| `13` | `/sys/` |
| `14` | `/run/` |
| `15` | `/mnt/` |
| `16` | `/media/` |
| `17` | `/root/` |
| `18` | `/sbin/` |
| `19` | `/usr/bin/` |
| `20` | `/usr/lib/` |
| `21` | `/usr/share/` |
| `22` | `/usr/local/` |
| `23` | `/var/log/` |
| `24` | `/var/lib/` |
| `25` | `/var/cache/` |
| `26-99` | Reserved / Custom |

---

## File Types (T)

| Code | Type |
|------|------|
| `0` | Regular file |
| `1` | Directory |
| `2` | Symbolic link |
| `3` | Hard link |
| `4` | Socket |
| `5` | Named pipe (FIFO) |
| `6` | Block device |
| `7` | Character device |

---

## Permission Modes (M)

Common Unix permissions encoded as single digit:

| Code | Symbolic | Octal | Description |
|------|----------|-------|-------------|
| `0` | `-rw-r--r--` | 644 | Standard file |
| `1` | `-rwxr-xr-x` | 755 | Executable |
| `2` | `-rw-------` | 600 | Private file |
| `3` | `-rwx------` | 700 | Private executable |
| `4` | `-rw-rw-r--` | 664 | Group writable |
| `5` | `-rwxrwxr-x` | 775 | Group executable |
| `6` | `drwxr-xr-x` | 755 | Standard directory |
| `7` | `drwx------` | 700 | Private directory |
| `8` | `lrwxrwxrwx` | 777 | Symlink (always 777) |
| `9` | `*` | Other | Custom/other permissions |

---

## Path Hash (HHHHHHHH)

8-digit hash derived from the full path using a deterministic hash function.
This ensures uniqueness within each directory prefix category.

---

## Check Digit (C)

Calculated using a weighted sum algorithm (similar to ISBN):
- Multiply alternating digits by 1 and 3
- Sum all results
- Check digit = (10 - (sum mod 10)) mod 10

---

## Examples

| FSID | Path | Decoded |
|------|------|---------|
| `0100123456785` | `/etc/passwd` | prefix=01(/etc), type=0(file), mode=0(644), hash=12345678, check=5 |
| `0510987654323` | `/home/user/.bashrc` | prefix=05(/home), type=1(dir), mode=0(644), hash=98765432, check=3 |
| `2320111111117` | `/var/log/syslog` | prefix=23(/var/log), type=0(file), mode=2(600), hash=01111111, check=7 |

---

## Usage

```bash
# Convert path to FSID
$ fsid to /etc/passwd
0100123456785

# Convert FSID to path
$ fsid from 0100123456785
/etc/passwd

# Show detailed info
$ fsid info 0100123456785
FSID:   0100123456785
Path:   /etc/passwd
Prefix: /etc/ (01)
Type:   Regular file (0)
Mode:   -rw-r--r-- (644)
Valid:  ✓
```
