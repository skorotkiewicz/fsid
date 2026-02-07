# FSID

A self-contained identifier for files and directories.

## Install

```bash
cargo install --path .
```

## Usage

```bash
# Standard format (numeric)
fsid to /etc/passwd
# → 010012356385108566822

# Short format (~45% smaller)
fsid to /etc/passwd --short
# → 01017ssg6m1k4m

# Decode (auto-detects format)
fsid from 010012356385108566822
fsid from 01017ssg6m1k4m
# → /etc/passwd

# Show details
fsid info <fsid>
```

## Formats

| Format | Encoding | Example | Use Case |
|--------|----------|---------|----------|
| Standard | Base10 | `010012356385108566822` | Numeric-only systems |
| Short | Base36 | `01017ssg6m1k4m` | Compact storage |

See [FSID.md](FSID.md) for the full specification.

## License

MIT
