use sha2::{Sha256, Digest};
use ed25519_dalek::{Signature, PublicKey};

/// Hash message using SHA-256
pub fn hash_message(message: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(message);
    hasher.finalize().into()
}

/// Verify Ed25519 signature
pub fn verify_signature(
    message: &[u8],
    signature: &[u8],
    public_key: &[u8],
) -> bool {
    // Convert bytes to signature and public key
    if let (Ok(signature), Ok(public_key)) = (
        Signature::from_bytes(signature),
        PublicKey::from_bytes(public_key),
    ) {
        // Verify signature
        public_key.verify_strict(message, &signature).is_ok()
    } else {
        false
    }
}

/// Generate random bytes
pub fn random_bytes(length: usize) -> Vec<u8> {
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    let mut bytes = vec![0u8; length];
    rng.fill_bytes(&mut bytes);
    bytes
}

/// Derive key using PBKDF2
pub fn derive_key(password: &[u8], salt: &[u8], iterations: u32) -> [u8; 32] {
    use pbkdf2::pbkdf2_hmac;
    use sha2::Sha256;
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password, salt, iterations, &mut key);
    key
}

/// Encrypt data using AES-GCM
pub fn encrypt(
    data: &[u8],
    key: &[u8; 32],
    nonce: &[u8; 12],
) -> Result<Vec<u8>, aes_gcm::Error> {
    use aes_gcm::{
        aead::{Aead, NewAead},
        Aes256Gcm, Nonce,
    };
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(nonce);
    cipher.encrypt(nonce, data)
}

/// Decrypt data using AES-GCM
pub fn decrypt(
    ciphertext: &[u8],
    key: &[u8; 32],
    nonce: &[u8; 12],
) -> Result<Vec<u8>, aes_gcm::Error> {
    use aes_gcm::{
        aead::{Aead, NewAead},
        Aes256Gcm, Nonce,
    };
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(nonce);
    cipher.decrypt(nonce, ciphertext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_message() {
        let message = b"Hello, world!";
        let hash = hash_message(message);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_encryption() {
        let data = b"Secret message";
        let key = random_bytes(32).try_into().unwrap();
        let nonce = random_bytes(12).try_into().unwrap();

        let ciphertext = encrypt(data, &key, &nonce).unwrap();
        let plaintext = decrypt(&ciphertext, &key, &nonce).unwrap();

        assert_eq!(plaintext, data);
    }
}