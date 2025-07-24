/*!
# Configuration Encryption

Encryption utilities for sensitive configuration values.
*/

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use ring::{aead, pbkdf2, rand::{SecureRandom, SystemRandom}};
use serde::{Deserialize, Serialize};
use base64::engine::general_purpose::STANDARD as base64_engine;
use base64::Engine;

const ALGORITHM: &aead::Algorithm = &aead::AES_256_GCM;
const PBKDF2_ALGORITHM: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA256;

/// Encrypted configuration value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedValue {
    pub ciphertext: String,
    pub nonce: String,
    pub salt: String,
    pub algorithm: String,
}

/// Configuration encryption manager
pub struct ConfigEncryption {
    key: aead::LessSafeKey,
    rng: SystemRandom,
}

impl ConfigEncryption {
    /// Create new encryption manager with password
    pub fn new_with_password(password: &str, salt: &[u8]) -> Result<Self> {
        let mut key_bytes = [0u8; 32];
        pbkdf2::derive(
            PBKDF2_ALGORITHM,
            std::num::NonZeroU32::new(100_000).unwrap(),
            salt,
            password.as_bytes(),
            &mut key_bytes,
        );

        let key = aead::UnboundKey::new(ALGORITHM, &key_bytes)
            .map_err(|_| anyhow::anyhow!("Failed to create encryption key"))?;
        let key = aead::LessSafeKey::new(key);

        Ok(Self {
            key,
            rng: SystemRandom::new(),
        })
    }

    /// Create new encryption manager with key file
    pub fn new_with_key_file(key_file_path: &Path) -> Result<Self> {
        let key_data = fs::read(key_file_path)
            .with_context(|| format!("Failed to read key file: {:?}", key_file_path))?;

        if key_data.len() != 32 {
            return Err(anyhow::anyhow!("Key file must contain exactly 32 bytes"));
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&key_data);

        let key = aead::UnboundKey::new(ALGORITHM, &key_bytes)
            .map_err(|_| anyhow::anyhow!("Failed to create encryption key"))?;
        let key = aead::LessSafeKey::new(key);

        Ok(Self {
            key,
            rng: SystemRandom::new(),
        })
    }

    /// Generate a new encryption key file
    pub fn generate_key_file(output_path: &Path) -> Result<()> {
        let rng = SystemRandom::new();
        let mut key_bytes = [0u8; 32];
        rng.fill(&mut key_bytes)
            .map_err(|_| anyhow::anyhow!("Failed to generate random key"))?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        fs::write(output_path, &key_bytes)
            .with_context(|| format!("Failed to write key file: {:?}", output_path))?;

        // Set restrictive permissions on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(output_path)?.permissions();
            perms.set_mode(0o600); // Read/write for owner only
            fs::set_permissions(output_path, perms)?;
        }

        tracing::info!("Generated encryption key file: {:?}", output_path);
        Ok(())
    }

    /// Encrypt a string value
    pub fn encrypt_string(&self, plaintext: &str) -> Result<EncryptedValue> {
        let mut nonce_bytes = [0u8; 12];
        self.rng.fill(&mut nonce_bytes)
            .map_err(|_| anyhow::anyhow!("Failed to generate nonce"))?;

        let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);
        let mut in_out = plaintext.as_bytes().to_vec();

        self.key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|_| anyhow::anyhow!("Encryption failed"))?;

        let mut salt = [0u8; 16];
        self.rng.fill(&mut salt)
            .map_err(|_| anyhow::anyhow!("Failed to generate salt"))?;

        Ok(EncryptedValue {
            ciphertext: base64_engine.encode(&in_out),
            nonce: base64_engine.encode(&nonce_bytes),
            salt: base64_engine.encode(&salt),
            algorithm: "AES-256-GCM".to_string(),
        })
    }

    /// Decrypt a string value
    pub fn decrypt_string(&self, encrypted: &EncryptedValue) -> Result<String> {
        let ciphertext = base64_engine.decode(&encrypted.ciphertext)
            .context("Failed to decode ciphertext")?;
        let nonce_bytes = base64_engine.decode(&encrypted.nonce)
            .context("Failed to decode nonce")?;

        if nonce_bytes.len() != 12 {
            return Err(anyhow::anyhow!("Invalid nonce length"));
        }

        let mut nonce_array = [0u8; 12];
        nonce_array.copy_from_slice(&nonce_bytes);
        let nonce = aead::Nonce::assume_unique_for_key(nonce_array);

        let mut in_out = ciphertext;
        let plaintext = self.key.open_in_place(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|_| anyhow::anyhow!("Decryption failed"))?;

        String::from_utf8(plaintext.to_vec())
            .context("Decrypted data is not valid UTF-8")
    }

    /// Check if a value is encrypted
    pub fn is_encrypted_value(value: &str) -> bool {
        value.starts_with("encrypted:")
    }

    /// Wrap encrypted value with prefix
    pub fn wrap_encrypted_value(encrypted: &EncryptedValue) -> Result<String> {
        let serialized = serde_json::to_string(encrypted)
            .context("Failed to serialize encrypted value")?;
        Ok(format!("encrypted:{}", base64_engine.encode(serialized)))
    }

    /// Unwrap encrypted value from prefixed string
    pub fn unwrap_encrypted_value(wrapped: &str) -> Result<EncryptedValue> {
        if !wrapped.starts_with("encrypted:") {
            return Err(anyhow::anyhow!("Value is not encrypted"));
        }

        let encoded = &wrapped[10..]; // Remove "encrypted:" prefix
        let serialized = base64_engine.decode(encoded)
            .context("Failed to decode encrypted value")?;
        let encrypted: EncryptedValue = serde_json::from_slice(&serialized)
            .context("Failed to deserialize encrypted value")?;

        Ok(encrypted)
    }

    /// Decrypt wrapped value
    pub fn decrypt_wrapped_value(&self, wrapped: &str) -> Result<String> {
        let encrypted = Self::unwrap_encrypted_value(wrapped)?;
        self.decrypt_string(&encrypted)
    }
}

/// Configuration encryption utilities
pub struct ConfigEncryptionUtils;

impl ConfigEncryptionUtils {
    /// Encrypt sensitive fields in configuration YAML
    pub fn encrypt_config_file(
        input_path: &Path,
        output_path: &Path,
        encryption: &ConfigEncryption,
        sensitive_fields: &[&str],
    ) -> Result<()> {
        let content = fs::read_to_string(input_path)
            .with_context(|| format!("Failed to read config file: {:?}", input_path))?;

        let mut yaml_value: serde_yaml::Value = serde_yaml::from_str(&content)
            .context("Failed to parse YAML")?;

        Self::encrypt_yaml_fields(&mut yaml_value, encryption, sensitive_fields)?;

        let encrypted_content = serde_yaml::to_string(&yaml_value)
            .context("Failed to serialize encrypted YAML")?;

        fs::write(output_path, encrypted_content)
            .with_context(|| format!("Failed to write encrypted config: {:?}", output_path))?;

        tracing::info!("Encrypted configuration saved to: {:?}", output_path);
        Ok(())
    }

    /// Decrypt configuration file
    pub fn decrypt_config_file(
        input_path: &Path,
        output_path: &Path,
        encryption: &ConfigEncryption,
    ) -> Result<()> {
        let content = fs::read_to_string(input_path)
            .with_context(|| format!("Failed to read encrypted config: {:?}", input_path))?;

        let mut yaml_value: serde_yaml::Value = serde_yaml::from_str(&content)
            .context("Failed to parse encrypted YAML")?;

        Self::decrypt_yaml_fields(&mut yaml_value, encryption)?;

        let decrypted_content = serde_yaml::to_string(&yaml_value)
            .context("Failed to serialize decrypted YAML")?;

        fs::write(output_path, decrypted_content)
            .with_context(|| format!("Failed to write decrypted config: {:?}", output_path))?;

        tracing::info!("Decrypted configuration saved to: {:?}", output_path);
        Ok(())
    }

    /// Recursively encrypt specified fields in YAML
    fn encrypt_yaml_fields(
        value: &mut serde_yaml::Value,
        encryption: &ConfigEncryption,
        sensitive_fields: &[&str],
    ) -> Result<()> {
        match value {
            serde_yaml::Value::Mapping(map) => {
                for (key, val) in map.iter_mut() {
                    if let serde_yaml::Value::String(key_str) = key {
                        if sensitive_fields.contains(&key_str.as_str()) {
                            if let serde_yaml::Value::String(string_val) = val {
                                if !ConfigEncryption::is_encrypted_value(string_val) {
                                    let encrypted = encryption.encrypt_string(string_val)?;
                                    let wrapped = ConfigEncryption::wrap_encrypted_value(&encrypted)?;
                                    *val = serde_yaml::Value::String(wrapped);
                                }
                            }
                        } else {
                            Self::encrypt_yaml_fields(val, encryption, sensitive_fields)?;
                        }
                    }
                }
            }
            serde_yaml::Value::Sequence(seq) => {
                for item in seq.iter_mut() {
                    Self::encrypt_yaml_fields(item, encryption, sensitive_fields)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Recursively decrypt encrypted fields in YAML
    fn decrypt_yaml_fields(
        value: &mut serde_yaml::Value,
        encryption: &ConfigEncryption,
    ) -> Result<()> {
        match value {
            serde_yaml::Value::Mapping(map) => {
                for (_, val) in map.iter_mut() {
                    Self::decrypt_yaml_fields(val, encryption)?;
                }
            }
            serde_yaml::Value::Sequence(seq) => {
                for item in seq.iter_mut() {
                    Self::decrypt_yaml_fields(item, encryption)?;
                }
            }
            serde_yaml::Value::String(string_val) => {
                if ConfigEncryption::is_encrypted_value(string_val) {
                    let decrypted = encryption.decrypt_wrapped_value(string_val)?;
                    *string_val = decrypted;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Get list of commonly sensitive configuration fields
    pub fn get_sensitive_fields() -> Vec<&'static str> {
        vec![
            "jwt_secret",
            "password",
            "secret",
            "key",
            "token",
            "api_key",
            "private_key",
            "encryption_key",
            "database_password",
            "mqtt_password",
            "admin_password",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_key_generation() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("test.key");

        ConfigEncryption::generate_key_file(&key_path).unwrap();
        assert!(key_path.exists());

        let key_data = fs::read(&key_path).unwrap();
        assert_eq!(key_data.len(), 32);
    }

    #[test]
    fn test_encryption_with_password() {
        let salt = b"test_salt_16byte";
        let encryption = ConfigEncryption::new_with_password("test_password", salt).unwrap();

        let plaintext = "sensitive_value";
        let encrypted = encryption.encrypt_string(plaintext).unwrap();
        let decrypted = encryption.decrypt_string(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
        assert_eq!(encrypted.algorithm, "AES-256-GCM");
    }

    #[test]
    fn test_encryption_with_key_file() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("test.key");

        ConfigEncryption::generate_key_file(&key_path).unwrap();
        let encryption = ConfigEncryption::new_with_key_file(&key_path).unwrap();

        let plaintext = "test_data";
        let encrypted = encryption.encrypt_string(plaintext).unwrap();
        let decrypted = encryption.decrypt_string(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_wrapped_encryption() {
        let salt = b"test_salt_16byte";
        let encryption = ConfigEncryption::new_with_password("test_password", salt).unwrap();

        let plaintext = "secret_value";
        let encrypted = encryption.encrypt_string(plaintext).unwrap();
        let wrapped = ConfigEncryption::wrap_encrypted_value(&encrypted).unwrap();

        assert!(wrapped.starts_with("encrypted:"));
        assert!(ConfigEncryption::is_encrypted_value(&wrapped));

        let decrypted = encryption.decrypt_wrapped_value(&wrapped).unwrap();
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_config_file_encryption() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("test.key");
        let input_path = temp_dir.path().join("input.yaml");
        let output_path = temp_dir.path().join("output.yaml");

        ConfigEncryption::generate_key_file(&key_path).unwrap();
        let encryption = ConfigEncryption::new_with_key_file(&key_path).unwrap();

        let config_content = r#"
general:
  instance_id: "test"
security:
  jwt_secret: "super_secret_key"
  password: "admin123"
network:
  port: 8080
"#;

        fs::write(&input_path, config_content).unwrap();

        ConfigEncryptionUtils::encrypt_config_file(
            &input_path,
            &output_path,
            &encryption,
            &["jwt_secret", "password"],
        ).unwrap();

        let encrypted_content = fs::read_to_string(&output_path).unwrap();
        assert!(encrypted_content.contains("encrypted:"));
        assert!(!encrypted_content.contains("super_secret_key"));
        assert!(!encrypted_content.contains("admin123"));
    }

    #[test]
    fn test_sensitive_fields_list() {
        let fields = ConfigEncryptionUtils::get_sensitive_fields();
        assert!(fields.contains(&"jwt_secret"));
        assert!(fields.contains(&"password"));
        assert!(fields.contains(&"api_key"));
    }
}