# FSID

**ISBN for your filesystem.** A 13-digit identifier for files and directories.

## Install

```bash
cargo install --path .
```

## Usage

```bash
# Generate FSID
fsid to /etc/passwd
# → 0100893316018

# Lookup path
fsid from 0100893316018
# → /etc/passwd

# Details
fsid info 0100893316018

# List all
fsid list
```

## Structure

```
01 0 0 89331601 8
│  │ │ │        └─ check digit
│  │ │ └────────── path hash (8 digits)
│  │ └──────────── permissions (0-9)
│  └────────────── type (file/dir/symlink)
└───────────────── prefix (/etc/ = 01)
```

See [FSID.md](FSID.md) for the full specification.

## Storage

FSID stores its mappings in a JSON file at `~/.local/share/fsid/storage.json`.

## License

MIT
