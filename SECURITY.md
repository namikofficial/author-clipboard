# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.3.x   | ✅ Yes    |
| < 0.3   | ❌ No     |

## Reporting Vulnerabilities

If you discover a security vulnerability, please report it responsibly:

1. **Do NOT open a public issue**
2. Email: [Create a private security advisory](https://github.com/namikofficial/author-clipboard/security/advisories/new) on GitHub
3. Include: description, reproduction steps, potential impact
4. Expected response time: 48 hours

## Threat Model

### What We Protect Against

| Threat | Mitigation | Status |
|--------|-----------|--------|
| Sensitive data in clipboard history | Auto-detection of passwords, API keys, tokens, SSH keys, connection strings | ✅ Implemented |
| Credential patterns in URIs | Detection of `://user:pass@host` patterns | ✅ Implemented |
| IPC socket hijacking | Socket in `$XDG_RUNTIME_DIR` with 0700 fallback directory, never `/tmp` | ✅ Implemented |
| Unauthorized clipboard access | Requires compositor support for `wlr-data-control` protocol | ✅ By design |
| Data at rest exposure | AES-256-GCM encryption for sensitive items (opt-in via `encrypt_sensitive`) | ✅ Implemented |
| Encryption key exposure | Key file stored with 0600 permissions in data directory | ✅ Implemented |
| Clipboard data after screen lock | Optional clear-on-lock via `clear_on_lock` config | ✅ Implemented |
| Stale sensitive data | Configurable TTL with automatic cleanup | ✅ Implemented |

### What We Do NOT Protect Against

- **Root access**: A root user can read all user files including the encryption key
- **Memory dumps**: Clipboard content exists in process memory; we do not use `mlock` or secure memory
- **Compositor-level access**: The Wayland compositor itself has full clipboard access by design
- **Key management**: The encryption key is stored as a file — not in a hardware security module or OS keyring (planned for future)
- **Side-channel attacks**: No constant-time comparison or timing-attack mitigations
- **Clipboard content before daemon start**: Only content copied after the daemon starts is managed

### Sensitive Content Detection

The following patterns are detected and flagged as sensitive:

- **Passwords**: Common password field patterns
- **API keys/tokens**: Bearer tokens, API key prefixes (sk-, pk-, ghp_, etc.)
- **SSH private keys**: `-----BEGIN * PRIVATE KEY-----`
- **AWS credentials**: `AKIA` prefix patterns
- **Connection strings**: Database URIs with embedded credentials (`://user:pass@host`)
- **JWT tokens**: `eyJ` base64-encoded JSON tokens
- **Generic secrets**: High-entropy strings that appear to be secrets

### Encryption Details

When `encrypt_sensitive: true` is set in config:

- **Algorithm**: AES-256-GCM (authenticated encryption)
- **Key**: 256-bit random key, generated on first use
- **Nonce**: 12-byte random nonce per encryption operation
- **Storage**: Nonce prepended to ciphertext, then base64-encoded
- **Key file**: `~/.local/share/author-clipboard/.encryption_key` (mode 0600)

### COSMIC_DATA_CONTROL_ENABLED

This application requires `COSMIC_DATA_CONTROL_ENABLED=1` on COSMIC desktop. This environment variable enables the `zwlr_data_control_manager_v1` Wayland protocol, which allows clipboard manager applications to monitor and set clipboard content.

**Security implication**: Enabling this grants any application with Wayland access the ability to read clipboard content. This is the same access model used by clipboard managers on X11 and other platforms.

## Security Best Practices for Users

1. **Enable encryption**: Set `encrypt_sensitive: true` in your config
2. **Enable clear-on-lock**: Set `clear_on_lock: true` to clear sensitive items when screen locks
3. **Review TTL settings**: Lower `ttl_seconds` for shorter history retention
4. **Check sensitive detection**: Run `author-clipboard-ctl list --sensitive` to review flagged items
5. **Restrict data directory permissions**: Ensure `~/.local/share/author-clipboard/` is mode 0700

## Dependencies

Security-relevant dependencies:

| Crate | Purpose | Version Policy |
|-------|---------|---------------|
| `aes-gcm` | Authenticated encryption | Latest stable |
| `rusqlite` | SQLite with bundled library | Latest stable |
| `wayland-client` | Wayland protocol bindings | Latest stable |
| `base64` | Key encoding | Latest stable |
| `rand` | Cryptographic random generation | Latest stable |

## Audit History

| Date | Scope | Findings | Status |
|------|-------|----------|--------|
| 2025 | Full repo audit | URI credential detection gap, IPC `/tmp` fallback | ✅ Fixed in v0.3.1 |
