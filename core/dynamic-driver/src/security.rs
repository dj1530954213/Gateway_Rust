/*!
# Security Verification

Provides cryptographic verification for dynamic driver libraries.
*/

use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use sha2::{Sha256, Digest};
use ed25519_dalek::{Verifier, VerifyingKey, Signature};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use tracing::{info, warn, error, debug};

use crate::metadata::DriverMetadata;
use crate::error::DynamicDriverError;

/// Security verifier for driver libraries
pub struct SecurityVerifier {
    /// Trusted public keys for signature verification
    trusted_keys: Vec<VerifyingKey>,
    
    /// Whether to require signatures
    require_signatures: bool,
    
    /// Whether to check file hashes
    check_hashes: bool,
}

impl SecurityVerifier {
    /// Create new security verifier
    pub fn new() -> Result<Self> {
        Ok(Self {
            trusted_keys: Vec::new(),
            require_signatures: false,
            check_hashes: true,
        })
    }
    
    /// Add a trusted public key for signature verification
    pub fn add_trusted_key(&mut self, public_key: &[u8]) -> Result<()> {
        let key_array: [u8; 32] = public_key.try_into()
            .map_err(|_| DynamicDriverError::InvalidPublicKey("Key must be exactly 32 bytes".to_string()))?;
        let key = VerifyingKey::from_bytes(&key_array)
            .map_err(|e| DynamicDriverError::InvalidPublicKey(e.to_string()))?;
        
        self.trusted_keys.push(key);
        info!("Added trusted public key (total: {})", self.trusted_keys.len());
        Ok(())
    }
    
    /// Add trusted key from base64 string
    pub fn add_trusted_key_base64(&mut self, base64_key: &str) -> Result<()> {
        let key_bytes = BASE64.decode(base64_key)
            .map_err(|e| DynamicDriverError::InvalidPublicKey(format!("Base64 decode error: {}", e)))?;
        
        if key_bytes.len() != 32 {
            return Err(DynamicDriverError::InvalidPublicKey(
                format!("Invalid key length: expected 32 bytes, got {}", key_bytes.len())
            ).into());
        }
        
        self.add_trusted_key(&key_bytes)
    }
    
    /// Set whether to require signatures
    pub fn set_require_signatures(&mut self, require: bool) {
        self.require_signatures = require;
        if require {
            info!("Signature verification required");
        } else {
            warn!("Signature verification disabled");
        }
    }
    
    /// Set whether to check file hashes
    pub fn set_check_hashes(&mut self, check: bool) {
        self.check_hashes = check;
    }
    
    /// Verify a driver library
    pub fn verify_driver<P: AsRef<Path>>(
        &self, 
        library_path: P, 
        metadata: &DriverMetadata
    ) -> Result<()> {
        let library_path = library_path.as_ref();
        
        info!("Verifying driver: {:?}", library_path);
        
        // Check if file exists
        if !library_path.exists() {
            return Err(DynamicDriverError::LibraryNotFound(library_path.to_path_buf()).into());
        }
        
        // Calculate file hash if enabled
        let file_hash = if self.check_hashes {
            let hash = self.calculate_file_hash(library_path)?;
            debug!("Driver file hash: {}", hex::encode(&hash));
            Some(hash)
        } else {
            None
        };
        
        // Check signature if required
        if self.require_signatures {
            self.verify_signature(library_path, metadata, file_hash.as_ref().map(|v| &**v))?;
        } else {
            debug!("Signature verification skipped");
        }
        
        // Additional security checks
        self.perform_security_checks(library_path)?;
        
        info!("Driver verification successful: {:?}", library_path);
        Ok(())
    }
    
    /// Calculate SHA256 hash of file
    fn calculate_file_hash<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>> {
        let mut hasher = Sha256::new();
        let data = fs::read(path.as_ref())
            .with_context(|| format!("Failed to read file: {:?}", path.as_ref()))?;
        
        hasher.update(&data);
        Ok(hasher.finalize().to_vec())
    }
    
    /// Verify driver signature
    fn verify_signature<P: AsRef<Path>>(
        &self,
        library_path: P,
        metadata: &DriverMetadata,
        file_hash: Option<&[u8]>,
    ) -> Result<()> {
        let library_path = library_path.as_ref();
        
        if self.trusted_keys.is_empty() {
            return Err(DynamicDriverError::NoTrustedKeys.into());
        }
        
        // Look for signature file
        let sig_path = library_path.with_extension("sig");
        if !sig_path.exists() {
            return Err(DynamicDriverError::SignatureNotFound(sig_path).into());
        }
        
        debug!("Reading signature file: {:?}", sig_path);
        let sig_data = fs::read_to_string(&sig_path)
            .with_context(|| format!("Failed to read signature file: {:?}", sig_path))?;
        
        // Parse signature file (simple format: base64 signature)
        let signature_bytes = BASE64.decode(sig_data.trim())
            .map_err(|e| DynamicDriverError::InvalidSignature(format!("Base64 decode error: {}", e)))?;
        
        if signature_bytes.len() != 64 {
            return Err(DynamicDriverError::InvalidSignature(
                format!("Invalid signature length: expected 64 bytes, got {}", signature_bytes.len())
            ).into());
        }
        
        let signature_array: [u8; 64] = signature_bytes.try_into()
            .map_err(|_| DynamicDriverError::InvalidSignature("Signature must be exactly 64 bytes".to_string()))?;
        let signature = Signature::from_bytes(&signature_array);
        
        // Create message to verify (metadata + file hash)
        let message = self.create_signature_message(metadata, file_hash)?;
        
        // Try each trusted key
        let mut verified = false;
        for (i, key) in self.trusted_keys.iter().enumerate() {
            match key.verify(&message, &signature) {
                Ok(()) => {
                    info!("Signature verified with key #{}", i);
                    verified = true;
                    break;
                }
                Err(e) => {
                    debug!("Signature verification failed with key #{}: {}", i, e);
                }
            }
        }
        
        if !verified {
            return Err(DynamicDriverError::SignatureVerificationFailed.into());
        }
        
        Ok(())
    }
    
    /// Create message for signature verification
    fn create_signature_message(
        &self,
        metadata: &DriverMetadata,
        file_hash: Option<&[u8]>,
    ) -> Result<Vec<u8>> {
        let mut message = Vec::new();
        
        // Add metadata fields
        message.extend_from_slice(metadata.name.as_bytes());
        message.extend_from_slice(metadata.version.as_bytes());
        message.extend_from_slice(metadata.author.as_bytes());
        
        // Add protocols
        for protocol in &metadata.protocols {
            message.extend_from_slice(protocol.as_bytes());
        }
        
        // Add file hash if available
        if let Some(hash) = file_hash {
            message.extend_from_slice(hash);
        }
        
        Ok(message)
    }
    
    /// Perform additional security checks
    fn perform_security_checks<P: AsRef<Path>>(&self, library_path: P) -> Result<()> {
        let library_path = library_path.as_ref();
        
        // Check file permissions (should not be world-writable)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(library_path)?;
            let mode = metadata.permissions().mode();
            
            // Check if world-writable (bit 1)
            if mode & 0o002 != 0 {
                warn!("Driver file is world-writable: {:?}", library_path);
                return Err(DynamicDriverError::InsecurePermissions(library_path.to_path_buf()).into());
            }
        }
        
        // Check file size (basic sanity check)
        let file_size = fs::metadata(library_path)?.len();
        if file_size > 100 * 1024 * 1024 {  // 100MB limit
            warn!("Driver file is unusually large: {} bytes", file_size);
        }
        
        if file_size < 1024 {  // 1KB minimum
            return Err(DynamicDriverError::SuspiciousFile(
                "File too small to be a valid driver".to_string()
            ).into());
        }
        
        Ok(())
    }
    
    /// Generate signature for a driver (for development/testing)
    #[cfg(feature = "dev-tools")]
    pub fn sign_driver<P: AsRef<Path>>(
        &self,
        library_path: P,
        metadata: &DriverMetadata,
        private_key: &ed25519_dalek::SigningKey,
    ) -> Result<String> {
        let library_path = library_path.as_ref();
        
        // Calculate file hash
        let file_hash = self.calculate_file_hash(library_path)?;
        
        // Create signature message
        let message = self.create_signature_message(metadata, Some(&file_hash))?;
        
        // Sign the message
        let signature = private_key.sign(&message);
        
        // Return base64-encoded signature
        Ok(BASE64.encode(signature.to_bytes()))
    }
    
    /// Get verifier statistics
    pub fn stats(&self) -> SecurityStats {
        SecurityStats {
            trusted_keys: self.trusted_keys.len(),
            require_signatures: self.require_signatures,
            check_hashes: self.check_hashes,
        }
    }
}

impl Default for SecurityVerifier {
    fn default() -> Self {
        Self::new().expect("Failed to create default SecurityVerifier")
    }
}

/// Security statistics
#[derive(Debug, Clone)]
pub struct SecurityStats {
    pub trusted_keys: usize,
    pub require_signatures: bool,
    pub check_hashes: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_verifier_creation() {
        let verifier = SecurityVerifier::new();
        assert!(verifier.is_ok());
        
        let verifier = verifier.unwrap();
        let stats = verifier.stats();
        assert_eq!(stats.trusted_keys, 0);
        assert!(!stats.require_signatures);
        assert!(stats.check_hashes);
    }
    
    #[test]
    fn test_trusted_key_management() {
        let mut verifier = SecurityVerifier::new().unwrap();
        
        // Test invalid key
        let invalid_key = vec![0u8; 16]; // Wrong length
        assert!(verifier.add_trusted_key(&invalid_key).is_err());
        
        // Test valid key
        let valid_key = vec![0u8; 32];
        assert!(verifier.add_trusted_key(&valid_key).is_ok());
        
        let stats = verifier.stats();
        assert_eq!(stats.trusted_keys, 1);
    }
    
    #[test]
    fn test_base64_key_parsing() {
        let mut verifier = SecurityVerifier::new().unwrap();
        
        // Valid base64 key (32 bytes = 44 chars in base64)
        let key_base64 = BASE64.encode(&vec![0u8; 32]);
        assert!(verifier.add_trusted_key_base64(&key_base64).is_ok());
        
        // Invalid base64
        assert!(verifier.add_trusted_key_base64("invalid-base64!").is_err());
        
        // Wrong length
        let wrong_length = BASE64.encode(&vec![0u8; 16]);
        assert!(verifier.add_trusted_key_base64(&wrong_length).is_err());
    }
    
    #[test]
    fn test_file_hash() {
        let verifier = SecurityVerifier::new().unwrap();
        
        // Create temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test content").unwrap();
        
        let hash = verifier.calculate_file_hash(temp_file.path()).unwrap();
        assert_eq!(hash.len(), 32); // SHA256 produces 32 bytes
        
        // Same content should produce same hash
        let hash2 = verifier.calculate_file_hash(temp_file.path()).unwrap();
        assert_eq!(hash, hash2);
    }
    
    #[test]
    fn test_signature_message_creation() {
        let verifier = SecurityVerifier::new().unwrap();
        
        let metadata = DriverMetadata::new(
            "test-driver".to_string(),
            "1.0.0".to_string(),
            "Test driver".to_string(),
            vec!["modbus".to_string()],
            "Test Author".to_string(),
        );
        
        let file_hash = vec![0u8; 32];
        let message = verifier.create_signature_message(&metadata, Some(&file_hash)).unwrap();
        
        assert!(!message.is_empty());
        
        // Same metadata should produce same message
        let message2 = verifier.create_signature_message(&metadata, Some(&file_hash)).unwrap();
        assert_eq!(message, message2);
        
        // Different metadata should produce different message
        let different_metadata = DriverMetadata::new(
            "different-driver".to_string(),
            "1.0.0".to_string(),
            "Test driver".to_string(),
            vec!["modbus".to_string()],
            "Test Author".to_string(),
        );
        let message3 = verifier.create_signature_message(&different_metadata, Some(&file_hash)).unwrap();
        assert_ne!(message, message3);
    }
    
    #[test]
    fn test_configuration() {
        let mut verifier = SecurityVerifier::new().unwrap();
        
        verifier.set_require_signatures(true);
        verifier.set_check_hashes(false);
        
        let stats = verifier.stats();
        assert!(stats.require_signatures);
        assert!(!stats.check_hashes);
    }
}