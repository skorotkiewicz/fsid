# FSID

**ISBN for your filesystem.** Compact, reversible file identifiers.

## Install

```bash
cargo install --path .
```

## Usage

```bash
fsid to /etc/passwd     # → 01017ssg6m1k4m
fsid from 01017ssg6m1k4m  # → /etc/passwd
fsid info 01017ssg6m1k4m  # detailed breakdown
```

## Structure

```
01 0 17ssg6m1k4 m
│  │ │          └── check (1 char)
│  │ └───────────── encoded path (base36)
│  └─────────────── type+mode (file:644)
└────────────────── prefix (/etc/)
```

Paths are encoded directly — no database needed.

See [FSID.md](FSID.md) for full specification.

## License

MIT
