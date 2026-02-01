// crates/chitin-core/src/crypto.rs

use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

use crate::error::ChitinError;

/// An ed25519 keypair for signing and verification.
pub struct Keypair {
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

impl Keypair {
    /// Generate a new random ed25519 keypair.
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        Keypair {
            signing_key,
            verifying_key,
        }
    }

    /// Get the public key bytes (32 bytes).
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }

    /// Sign a message and return the signature bytes.
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        let signature = self.signing_key.sign(message);
        signature.to_bytes().to_vec()
    }
}

/// Sign a message with the given signing key bytes.
///
/// Returns the ed25519 signature as a 64-byte vector.
pub fn sign_message(signing_key_bytes: &[u8; 32], message: &[u8]) -> Result<Vec<u8>, ChitinError> {
    let signing_key = SigningKey::from_bytes(signing_key_bytes);
    let signature = signing_key.sign(message);
    Ok(signature.to_bytes().to_vec())
}

/// Verify an ed25519 signature.
///
/// Returns `true` if the signature is valid for the given message and public key.
pub fn verify_signature(
    public_key_bytes: &[u8; 32],
    message: &[u8],
    signature_bytes: &[u8],
) -> Result<bool, ChitinError> {
    let verifying_key = VerifyingKey::from_bytes(public_key_bytes)
        .map_err(|e| ChitinError::Crypto(format!("Invalid public key: {}", e)))?;

    let signature_array: [u8; 64] = signature_bytes
        .try_into()
        .map_err(|_| ChitinError::Crypto("Signature must be exactly 64 bytes".to_string()))?;

    let signature = ed25519_dalek::Signature::from_bytes(&signature_array);

    match verifying_key.verify(message, &signature) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Compute SHA-256 hash of the given bytes.
///
/// Returns a 32-byte hash.
pub fn hash_bytes(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_sign_verify() {
        let keypair = Keypair::generate();
        let message = b"hello chitin protocol";

        let signature = keypair.sign(message);
        let pubkey = keypair.public_key_bytes();

        let valid = verify_signature(&pubkey, message, &signature).unwrap();
        assert!(valid);

        // Verify wrong message fails
        let wrong_message = b"wrong message";
        let invalid = verify_signature(&pubkey, wrong_message, &signature).unwrap();
        assert!(!invalid);
    }

    #[test]
    fn test_sign_message_function() {
        let keypair = Keypair::generate();
        let message = b"test message";

        let signing_key_bytes = keypair.signing_key.to_bytes();
        let signature = sign_message(&signing_key_bytes, message).unwrap();
        let pubkey = keypair.public_key_bytes();

        let valid = verify_signature(&pubkey, message, &signature).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_hash_bytes() {
        let data = b"reefipedia";
        let hash = hash_bytes(data);
        assert_eq!(hash.len(), 32);

        // Same input should produce same hash
        let hash2 = hash_bytes(data);
        assert_eq!(hash, hash2);

        // Different input should produce different hash
        let hash3 = hash_bytes(b"different");
        assert_ne!(hash, hash3);
    }
}
