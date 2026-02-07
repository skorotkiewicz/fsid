# FSID - File System Identifier

A self-contained identifier for files and directories.

## Two Formats

| Format | Encoding | Length | Check |
|--------|----------|--------|-------|
| **Standard** | Base10 (0-9) | 20-40 digits | 2 digits |
| **Short** | Base36 (0-9, a-z) | 7-25 chars | 1 char |

---

## Standard Format

```
PP T M NNNNNNNN...NNN CC
│  │ │ │              └── 2-digit check
│  │ │ └───────────────── encoded path (base10)
│  │ └─────────────────── permission mode (0-9)
│  └───────────────────── file type (0-7)
└──────────────────────── prefix (00-25)
```

### Prefixes (Standard)

| Code | Path | Code | Path |
|------|------|------|------|
| `00` | `/` | `13` | `/sys/` |
| `01` | `/etc/` | `14` | `/run/` |
| `02` | `/bin/` | `15` | `/mnt/` |
| `03` | `/usr/` | `16` | `/media/` |
| `04` | `/var/` | `17` | `/root/` |
| `05` | `/home/` | `18` | `/sbin/` |
| `06` | `/tmp/` | `19` | `/usr/bin/` |
| `07` | `/opt/` | `20` | `/usr/lib/` |
| `08` | `/lib/` | `21` | `/usr/share/` |
| `09` | `/srv/` | `22` | `/usr/local/` |
| `10` | `/boot/` | `23` | `/var/log/` |
| `11` | `/dev/` | `24` | `/var/lib/` |
| `12` | `/proc/` | `25` | `/var/cache/` |

### Permission Modes

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

## Short Format

```
PP T NNNNNN...N C
│  │ │          └── 1-char check (base36)
│  │ └───────────── encoded path (base36)
│  └─────────────── type+mode combined (1 char)
└────────────────── prefix (base36, 0a-0v)
```

### Prefixes (Short, additional)

| Code | Path | Code | Path |
|------|------|------|------|
| `0a` | `/boot/` | `0n` | `/var/log/` |
| `0b` | `/dev/` | `0o` | `/var/lib/` |
| `0j` | `/usr/bin/` | `0u` | `/usr/local/bin/` |
| `0k` | `/usr/lib/` | `0v` | `/usr/local/lib/` |

### Type+Mode Codes

| Code | Type | Mode | Code | Type | Mode |
|------|------|------|------|------|------|
| `0` | file | 644 | `8` | dir | 755 |
| `1` | file | 755 | `9` | dir | 700 |
| `2` | file | 600 | `a` | dir | 775 |
| `3` | file | 700 | `c` | symlink | — |
| `z` | other | — | | | |

---

## File Types

| Code | Type |
|------|------|
| `0` | Regular file |
| `1` | Directory |
| `2` | Symbolic link |
| `4` | Socket |
| `5` | Named pipe (FIFO) |
| `6` | Block device |
| `7` | Character device |

---

## Path Encoding

1. Remove matching prefix from path
2. Convert remaining bytes to big integer (base256)
3. Convert to target base (10 or 36)

Fully reversible — no information lost.

---

## Examples

| Path | Standard | Short |
|------|----------|-------|
| `/etc/passwd` | `010012356385108566822` | `01017ssg6m1k4m` |
| `/usr/bin/ls` | `19012776382` | `0j1lf7d` |
| `/var/log/pacman.log` | `230053070154474625815992304716` | `0n02ef4ljd3yhuunkvba` |
