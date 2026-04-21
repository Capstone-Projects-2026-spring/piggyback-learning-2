use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use keyring::Entry;

const KEYRING_SERVICE: &str = "piggyback";
const KEYRING_USER: &str = "voice_key";

/// Retrieve existing key from keychain or generate and store a new one
pub fn get_or_create_key() -> Result<[u8; 32], String> {
    let entry = Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| format!("[crypto] keyring entry failed: {e}"))?;

    match entry.get_password() {
        Ok(hex) => {
            // Key already exists — decode it
            let bytes =
                hex::decode(&hex).map_err(|e| format!("[crypto] key decode failed: {e}"))?;
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            eprintln!("[crypto] loaded existing key from keychain");
            Ok(key)
        }
        Err(_) => {
            // First run — generate a fresh key and persist it
            let key: [u8; 32] = rand::random();
            let hex = hex::encode(key);
            entry
                .set_password(&hex)
                .map_err(|e| format!("[crypto] keyring store failed: {e}"))?;
            eprintln!("[crypto] generated and stored new key in keychain");
            Ok(key)
        }
    }
}

/// Encrypt raw bytes — returns nonce (12 bytes) + ciphertext concatenated
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| format!("[crypto] encrypt failed: {e}"))?;

    // Prepend nonce so we can extract it on decrypt
    let mut out = nonce.to_vec();
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypt — expects nonce (12 bytes) + ciphertext as produced by encrypt()
pub fn decrypt(key: &[u8; 32], data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < 12 {
        return Err("[crypto] data too short to contain nonce".to_string());
    }
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("[crypto] decrypt failed: {e}"))
}
