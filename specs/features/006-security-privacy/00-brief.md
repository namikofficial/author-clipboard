# Feature Brief: Security & Privacy

> Sensitive content detection, encryption at rest, incognito mode, and screen lock clearing.

---

## Problem Statement

Clipboard often contains sensitive data (passwords, API keys, tokens). This data should be protected and users need controls to prevent capture of sensitive content.

## Proposed Solution

Pattern-based sensitive content detection, AES-256-GCM encryption for flagged items, incognito mode to pause capture, and screen lock detection to clear sensitive items.

## Goals

- Auto-detect passwords, API keys, tokens, SSH keys, URI credentials
- Encrypt sensitive items at rest when enabled
- Allow pausing capture with incognito mode
- Clear sensitive items when screen locks
- Audit logging without raw sensitive data

---

**Created**: Phase 7 Complete
**Status**: Implemented v0.5.0