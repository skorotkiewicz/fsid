# FSID - File System Identifier

A self-contained identifier for files and directories.

## Three Formats

| Format | Encoding | Length | Storage | Reversible |
|--------|----------|--------|---------|------------|
| **Standard** | Base10 | 20-40 digits | Optional | ✓ |
| **Short** | Base36 | 7-25 chars | Optional | ✓ |
| **Minimal** | Base10 | 13 digits | Required | ✗ (hash-based) |

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

---

## Short Format

```
PP T NNNNNN...N C
│  │ │          └── 1-char check (base36)
│  │ └───────────── encoded path (base36)
│  └─────────────── type+mode combined (1 char)
└────────────────── prefix (base36, 0a-0v)
```

---

## Minimal Format (13 digits)

```
PP T M HHHHHHHH C
│  │ │ │        └── 1-digit check
│  │ │ └─────────── path hash (8 digits)
│  │ └───────────── permission mode (0-9)
│  └─────────────── file type (0-7)
└────────────────── prefix (00-25)
```

**Note:** Hash-based, requires storage for decoding.

---

## Prefixes

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

Short format also supports: `0a`-`0v` for additional paths.

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

## Permission Modes

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

## Examples

| Path | Standard | Short | Minimal |
|------|----------|-------|---------|
| `/etc/passwd` | `010012356385108566822` | `01017ssg6m1k4m` | `0100893316018` |
| `/usr/bin/ls` | `19012776382` | `0j1lf7d` | `1901691270446` |

---

## Storage

When using `--storage`, mappings are saved to a JSON file:

```json
{
  "mappings": {
    "0100893316018": "/etc/passwd",
    "01017ssg6m1k4m": "/etc/passwd"
  }
}
```
