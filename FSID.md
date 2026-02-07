# FSID - File System Identifier

**FSID** is a variable-length numeric identifier for files and directories, inspired by ISBN for books.

## Key Feature: **No Storage Required**

FSID encodes the **complete path** into the identifier itself. The path can be fully reconstructed from just the FSID — no database, no lookup table.

## Structure

```
PP T M NNNNNNNN...NNN CC
│  │ │ │              └── 2-digit check code
│  │ │ └───────────────── encoded path (base256 → base10)
│  │ └─────────────────── permission mode (0-9)
│  └───────────────────── file type (0-7)
└──────────────────────── directory prefix (00-99)
```

**Typical length: 20-40 digits** (depends on path length)

---

## Directory Prefixes (PP)

The prefix removes common path prefixes to shorten the FSID:

| Prefix | Path |
|--------|------|
| `00` | `/` (root) |
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

| Code | Symbolic | Octal |
|------|----------|-------|
| `0` | `-rw-r--r--` | 644 |
| `1` | `-rwxr-xr-x` | 755 |
| `2` | `-rw-------` | 600 |
| `3` | `-rwx------` | 700 |
| `4` | `-rw-rw-r--` | 664 |
| `5` | `-rwxrwxr-x` | 775 |
| `6` | `drwxr-xr-x` | 755 |
| `7` | `drwx------` | 700 |
| `8` | `lrwxrwxrwx` | 777 |
| `9` | custom | — |

---

## Path Encoding

The remaining path (after prefix removal) is encoded as:
1. Convert path string to bytes (UTF-8)
2. Interpret bytes as base-256 big integer
3. Convert to base-10 (decimal digits)

This encoding is **fully reversible** — no information is lost.

---

## Check Code (CC)

Two-digit check code calculated using weighted sum:
- Alternating digits multiplied by 1 and 3
- Sum modulo 100

---

## Examples

| FSID | Path | Length |
|------|------|--------|
| `010012356385108566822` | `/etc/passwd` | 21 digits |
| `19012776382` | `/usr/bin/ls` | 11 digits |
| `230053070154474625815992304716` | `/var/log/pacman.log` | 30 digits |

---

## Usage

```bash
# Encode path → FSID
$ fsid to /etc/passwd
010012356385108566822

# Decode FSID → path
$ fsid from 010012356385108566822
/etc/passwd

# Show details
$ fsid info 010012356385108566822
```
