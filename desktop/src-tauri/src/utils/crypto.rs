use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use keyring::Entry;
use std::sync::OnceLock;

const KEYRING_SERVICE: &str = "piggyback";
const KEYRING_USER: &str = "voice_key";

static VOICE_KEY: OnceLock<[u8; 32]> = OnceLock::new();

pub fn get_voice_key() -> &'static [u8; 32] {
    VOICE_KEY
        .get()
        .expect("[crypto] voice key not initialised — call init_voice_key() at startup")
}

/// Load or generate the voice embedding encryption key and cache it.
/// Called once from init_db(), safe to call again, will no-op.
pub fn init_voice_key() -> Result<(), String> {
    let key = get_or_create_key()?;
    VOICE_KEY
        .set(key)
        .map_err(|_| "[crypto] voice key already initialised".to_string())
}

/// Retrieve existing key from keychain or generate and store a new one.
fn get_or_create_key() -> Result<[u8; 32], String> {
    let entry = Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| format!("[crypto] keyring entry failed: {e}"))?;

    match entry.get_password() {
        Ok(hex) => {
            let bytes =
                hex::decode(&hex).map_err(|e| format!("[crypto] key decode failed: {e}"))?;
            if bytes.len() != 32 {
                return Err(format!("[crypto] key wrong length: {} bytes", bytes.len()));
            }
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            eprintln!("[crypto] loaded key from keychain");
            Ok(key)
        }
        Err(_) => {
            let key: [u8; 32] = rand::random();
            entry
                .set_password(&hex::encode(key))
                .map_err(|e| format!("[crypto] keyring store failed: {e}"))?;
            eprintln!("[crypto] generated and stored new key");
            Ok(key)
        }
    }
}

/// Encrypt raw bytes -> returns nonce (12 bytes) prepended to ciphertext.
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let mut out = nonce.to_vec();
    out.extend_from_slice(
        &cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| format!("[crypto] encrypt failed: {e}"))?,
    );
    Ok(out)
}

/// Decrypt -> expects nonce (12 bytes) + ciphertext as produced by encrypt().
pub fn decrypt(key: &[u8; 32], data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < 12 {
        return Err("[crypto] data too short to contain nonce".to_string());
    }
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
        .map_err(|e| format!("[crypto] decrypt failed: {e}"))
}
