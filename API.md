## Web API

Start an HTTP server for programmatic access:

```bash
# Start server (default: 127.0.0.1:8080)
fsid serve --storage ./catalog.json

# Custom address
fsid serve --addr 0.0.0.0:3000 --storage ./catalog.json
```

### Endpoints

All requests are `POST /` with JSON body:

```bash
# Generate FSID
curl -X POST http://localhost:8080 \
  -H "Content-Type: application/json" \
  -d '{"action": "to", "path": "/etc/passwd", "format": "min"}'
# → {"success":true,"fsid":"0100893316018"}

# Decode FSID
curl -X POST http://localhost:8080 \
  -H "Content-Type: application/json" \
  -d '{"action": "from", "fsid": "0100893316018"}'
# → {"success":true,"path":"/etc/passwd"}

# Get info
curl -X POST http://localhost:8080 \
  -H "Content-Type: application/json" \
  -d '{"action": "info", "fsid": "0100893316018"}'

# List all
curl -X POST http://localhost:8080 \
  -H "Content-Type: application/json" \
  -d '{"action": "list"}'
```

### Request Format

| Field | Required | Values |
|-------|----------|--------|
| `action` | Yes | `to`, `from`, `info`, `list` |
| `path` | For `to` | File path |
| `fsid` | For `from`, `info` | FSID string |
| `format` | Optional | `standard`, `short`, `min` |