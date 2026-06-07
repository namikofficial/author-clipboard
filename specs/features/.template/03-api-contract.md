# API Contract: {feature-name}

> IPC protocol and API definitions for this feature.

---

## IPC Commands

### new_command

**Request**:
```json
{
  "cmd": "new_command",
  "args": {
    "param1": "value",
    "param2": 123
  }
}
```

**Response (success)**:
```json
{
  "ok": true,
  "data": {
    "result": "value"
  },
  "error": null
}
```

**Response (error)**:
```json
{
  "ok": false,
  "data": null,
  "error": "ERROR_CODE"
}
```

### Error Codes

| Code | Meaning |
|------|---------|
| `INVALID_ARG` | Invalid argument provided |
| `NOT_FOUND` | Item not found |
| `PERMISSION_DENIED` | Operation not permitted |
| `INTERNAL_ERROR` | Internal error |

---

## CLI Changes

### New Subcommand

```bash
author-clipboard-ctl new-command --flag <value>
```

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--flag` | string | | |

---

**Last Updated**: {date}