# FSID

**ISBN for your filesystem.** A numeric identifier for files — no storage required.

## Install

```bash
cargo install --path .
```

## Usage

```bash
# Encode path → FSID
fsid to /etc/passwd
# → 010012356385108566822

# Decode FSID → path
fsid from 010012356385108566822
# → /etc/passwd

# Show details
fsid info 010012356385108566822
```

## Structure

```
01 0 0 12356385108566 822
│  │ │ │              └── check (2 digits)
│  │ │ └───────────────── encoded path
│  │ └─────────────────── permissions (644)
│  └───────────────────── type (file)
└──────────────────────── prefix (/etc/)
```

The path is encoded directly into the FSID — fully reversible, no database needed.

See [FSID.md](FSID.md) for the full specification.

## License

MIT
