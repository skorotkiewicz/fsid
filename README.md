# FSID

A self-contained identifier for files and directories.

## Install

```bash
cargo install --path .
```

## Usage

### Basic (self-contained, no storage)

```bash
# Standard format (numeric, longer)
fsid to /etc/passwd
# → 010012356385108566822

# Short format (base36, ~45% shorter)
fsid to /etc/passwd --short
# → 01017ssg6m1k4m

# Decode (auto-detects format)
fsid from 010012356385108566822
fsid from 01017ssg6m1k4m
# → /etc/passwd
```

### With Storage

```bash
# Store mappings for later lookup
fsid to /etc/passwd --storage ./db.json
fsid to /var/log/syslog --short --storage ./db.json

# Minimal 13-digit (requires storage)
fsid to /etc/passwd --min --storage ./db.json
# → 0100893316018

# Lookup from storage
fsid from 0100893316018 --storage ./db.json
# → /etc/passwd

# List all stored FSIDs
fsid list --storage ./db.json
```

## Formats

| Format | Encoding | Length | Storage | Example |
|--------|----------|--------|---------|---------|
| Standard | Base10 | 20-40 | Optional | `010012356385108566822` |
| Short | Base36 | 7-25 | Optional | `01017ssg6m1k4m` |
| Minimal | Base10 | 13 | **Required** | `0100893316018` |

See [FSID.md](FSID.md) for the full specification.

## License

MIT
