# Requirements: Security & Privacy

---

## User Stories

### US-001: Sensitive Content Detection
**As a** user
**I want to** have sensitive content automatically detected
**So that** I don't accidentally store passwords or keys

**Acceptance Criteria**:
- Given I copy a password field value, when it contains common patterns, then it is flagged as sensitive
- Given I copy `ghp_xxxxx`, when it matches GitHub token pattern, then it is flagged as sensitive
- Given I copy `-----BEGIN OPENSSH PRIVATE KEY-----`, then it is flagged as sensitive

### US-002: Encryption at Rest
**As a** user
**I want to** have sensitive items encrypted
**So that** even if someone accesses my database, they can't read secrets

**Acceptance Criteria**:
- Given `encrypt_sensitive: true`, when a sensitive item is stored, then it is encrypted with AES-256-GCM
- Given I restore an encrypted item, when decryption is attempted, then content is returned correctly

### US-003: Incognito Mode
**As a** user
**I want to** temporarily pause clipboard capture
**So that** I can do sensitive work without recording

**Acceptance Criteria**:
- Given `.incognito` file exists, when content is copied, then it is not stored
- Given I remove `.incognito`, when content is copied, then it is stored normally

### US-004: Clear on Screen Lock
**As a** user
**I want to** have sensitive items cleared when my screen locks
**So that** no sensitive data is left accessible

**Acceptance Criteria**:
- Given `clear_on_lock: true`, when `loginctl lock-session` is run, then sensitive items are deleted
- Given screen locks via D-Bus `org.freedesktop.ScreenSaver`, then sensitive items are deleted

---

## Sensitive Detection Patterns

| Pattern | Examples |
|---------|----------|
| Password fields | `password: xxx`, `pass: xxx` |
| API keys | `sk-xxx`, `pk-xxx`, `ghp_xxx`, `AKIAxxx` |
| SSH keys | `-----BEGIN * PRIVATE KEY-----` |
| JWT tokens | `eyJxxx` (base64 JSON) |
| URI credentials | `://user:pass@host` |
| Generic secrets | High-entropy strings |

---

## Encryption Details

| Aspect | Value |
|--------|-------|
| Algorithm | AES-256-GCM |
| Key | 256-bit random, generated on first use |
| Nonce | 12-byte random per encryption |
| Storage | Nonce + ciphertext, base64 encoded |
| Key file | `<data_dir>/.encryption_key` (mode 0600) |

---

**Last Updated**: Phase 7 Complete