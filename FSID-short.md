# FSID - File System Identifier

A compact, reversible identifier for files. No storage required.

## Structure

```
PP T NNNNNN...N C
│  │ │          └── check (1 base36 char)
│  │ └───────────── encoded path (base36)
│  └─────────────── type + mode (1 base36 char)
└────────────────── prefix (2 base36 chars)
```

**Typical length: 7-20 characters**

---

## Encoding

- **Base36** alphabet: `0-9`, `a-z` (36 values per character)
- Path bytes → big integer (base256) → base36 string
- ~45% shorter than decimal encoding

---

## Prefixes (PP)

| Code | Path | Code | Path |
|------|------|------|------|
| `00` | `/` | `0g` | `/media/` |
| `01` | `/etc/` | `0h` | `/root/` |
| `02` | `/bin/` | `0i` | `/sbin/` |
| `03` | `/usr/` | `0j` | `/usr/bin/` |
| `04` | `/var/` | `0k` | `/usr/lib/` |
| `05` | `/home/` | `0l` | `/usr/share/` |
| `06` | `/tmp/` | `0m` | `/usr/local/` |
| `07` | `/opt/` | `0n` | `/var/log/` |
| `08` | `/lib/` | `0o` | `/var/lib/` |
| `09` | `/srv/` | `0p` | `/var/cache/` |
| `0a` | `/boot/` | `0q` | `/usr/lib64/` |
| `0b` | `/dev/` | `0r` | `/usr/include/` |
| `0c` | `/proc/` | `0s` | `/var/run/` |
| `0d` | `/sys/` | `0t` | `/var/tmp/` |
| `0e` | `/run/` | `0u` | `/usr/local/bin/` |
| `0f` | `/mnt/` | `0v` | `/usr/local/lib/` |

---

## Type + Mode (T)

Single character encodes both file type and permissions:

| Code | Type | Mode | Code | Type | Mode |
|------|------|------|------|------|------|
| `0` | file | 644 | `8` | dir | 755 |
| `1` | file | 755 | `9` | dir | 700 |
| `2` | file | 600 | `a` | dir | 775 |
| `3` | file | 700 | `b` | dir | 777 |
| `4` | file | 664 | `c` | symlink | — |
| `5` | file | 775 | `d` | socket | — |
| `6` | file | 666 | `e` | fifo | — |
| `7` | file | 777 | `f` | block | — |
| `z` | other | — | `g` | char | — |

---

## Examples

| Path | FSID | Length |
|------|------|--------|
| `/etc/passwd` | `01017ssg6m1k4m` | 14 |
| `/usr/bin/ls` | `0j1lf7d` | 7 |
| `/var/log/pacman.log` | `0n02ef4ljd3yhuunkvba` | 20 |
| `/tmp` | `00b4jjhsk` | 9 |

---

## Check Character

Weighted sum of all characters modulo 36, encoded as base36.
Catches typos and transmission errors.
