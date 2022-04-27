use rand_core::{OsRng, RngCore};
use sha2::{Sha256, Digest};

pub fn hash_content(content: Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(content);

    hasher.finalize().as_slice().to_vec()
}

pub fn generate_salt() -> Vec<u8> {
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);

    salt.to_vec()
}

pub fn hash_password_with_salt(content: Vec<u8>, salt: Vec<u8>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hasher.update(&salt);

    let hash_content = format!("{}$.{:x}", hex::encode(&salt), hasher.finalize());
    
    hash_content
}

pub fn verify_password(password: String, hash_content: String) -> bool {
    let hash_content_split: Vec<&str> = hash_content.split("$.").collect();
    let salt = hex::decode(hash_content_split[0]).unwrap();

    let _hash_content = hash_password_with_salt(password.as_bytes().to_vec(), salt);

    hash_content == _hash_content
}