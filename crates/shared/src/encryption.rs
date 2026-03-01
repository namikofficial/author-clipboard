//! Encryption at rest for sensitive clipboard items
//!
//! Uses AES-256-GCM for authenticated encryption. The encryption key is derived
//! from a machine-specific identifier stored in the data directory.

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::Engine;
use rand::RngCore;
use std::path::Path;
use tracing::debug;

const KEY_FILE: &str = ".encryption_key";
const NONCE_SIZE: usize = 12;

/// Manages encryption/decryption of clipboard content.
pub struct EncryptionManager {
    cipher: Aes256Gcm,
}

impl EncryptionManager {
    /// Create or load an encryption manager.
    /// Loads the key from the data directory, or generates a new one if not present.
    pub fn new(data_dir: &Path) -> Result<Self, String> {
        let key_path = data_dir.join(KEY_FILE);
        let key_bytes = if key_path.exists() {
            let encoded = std::fs::read_to_string(&key_path)
                .map_err(|e| format!("Failed to read encryption key: {e}"))?;
            base64::engine::general_purpose::STANDARD
                .decode(encoded.trim())
                .map_err(|e| format!("Failed to decode encryption key: {e}"))?
        } else {
            // Generate a new key
            let mut key = vec![0u8; 32]; // 256 bits
            OsRng.fill_bytes(&mut key);

            // Ensure directory exists
            if let Some(parent) = key_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create key directory: {e}"))?;
            }

            // Save key with restrictive permissions
            let encoded = base64::engine::general_purpose::STANDARD.encode(&key);
            std::fs::write(&key_path, &encoded)
                .map_err(|e| format!("Failed to write encryption key: {e}"))?;

            // Set file permissions to owner-only on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = std::fs::Permissions::from_mode(0o600);
                let _ = std::fs::set_permissions(&key_path, perms);
            }

            debug!("Generated new encryption key");
            key
        };

        if key_bytes.len() != 32 {
            return Err(format!(
                "Invalid key length: expected 32 bytes, got {}",
                key_bytes.len()
            ));
        }

        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        Ok(Self { cipher })
    }

    /// Encrypt plaintext content. Returns base64-encoded ciphertext (nonce prepended).
    pub fn encrypt(&self, plaintext: &str) -> Result<String, String> {
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| format!("Encryption failed: {e}"))?;

        // Prepend nonce to ciphertext
        let mut combined = nonce_bytes.to_vec();
        combined.extend_from_slice(&ciphertext);

        Ok(base64::engine::general_purpose::STANDARD.encode(combined))
    }

    /// Decrypt base64-encoded ciphertext (nonce prepended). Returns plaintext.
    pub fn decrypt(&self, encrypted: &str) -> Result<String, String> {
        let combined = base64::engine::general_purpose::STANDARD
            .decode(encrypted)
            .map_err(|e| format!("Failed to decode ciphertext: {e}"))?;

        if combined.len() < NONCE_SIZE {
            return Err("Ciphertext too short".to_string());
        }

        let (nonce_bytes, ciphertext) = combined.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {e}"))?;

        String::from_utf8(plaintext).map_err(|e| format!("Invalid UTF-8 after decryption: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let mgr = EncryptionManager::new(tmp.path()).unwrap();

        let plaintext = "Hello, World! 🌍 This is sensitive data.";
        let encrypted = mgr.encrypt(plaintext).unwrap();

        // Encrypted should be different from plaintext
        assert_ne!(encrypted, plaintext);

        // Decrypt should return original
        let decrypted = mgr.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_key_persistence() {
        let tmp = TempDir::new().unwrap();

        // Create manager and encrypt
        let mgr1 = EncryptionManager::new(tmp.path()).unwrap();
        let encrypted = mgr1.encrypt("test data").unwrap();

        // Create new manager from same dir — should load same key
        let mgr2 = EncryptionManager::new(tmp.path()).unwrap();
        let decrypted = mgr2.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, "test data");
    }

    #[test]
    fn test_different_keys_fail() {
        let tmp1 = TempDir::new().unwrap();
        let tmp2 = TempDir::new().unwrap();

        let mgr1 = EncryptionManager::new(tmp1.path()).unwrap();
        let mgr2 = EncryptionManager::new(tmp2.path()).unwrap();

        let encrypted = mgr1.encrypt("secret").unwrap();
        // Different key should fail to decrypt
        assert!(mgr2.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_encrypt_empty_string() {
        let tmp = TempDir::new().unwrap();
        let mgr = EncryptionManager::new(tmp.path()).unwrap();

        let encrypted = mgr.encrypt("").unwrap();
        let decrypted = mgr.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, "");
    }
}
