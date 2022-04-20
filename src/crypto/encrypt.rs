use aes_gcm::{Aes256Gcm, Key, Nonce}; // Or `Aes128Gcm`
use aes_gcm::aead::{Aead, NewAead};

pub fn encrypt(nonce: &[u8], key: &[u8], bytes_to_encrypt: &[u8]) -> Result<Vec<u8>, aes_gcm::Error> {
    let key = Key::from_slice(key);
    let nonce = Nonce::from_slice(nonce);
    let aead = Aes256Gcm::new(key);
    let ciphertext = aead.encrypt(nonce, bytes_to_encrypt)?;

    Ok(ciphertext)
}