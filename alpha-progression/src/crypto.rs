use base64::{engine::general_purpose::STANDARD, Engine};
use sha2::{Digest, Sha256};

const SALT: &str = app_secret!("AP_PASSWORD_SALT");

pub fn hash_password(password: &str) -> String {
    let input = format!("{}{}", password, SALT);
    let hash = Sha256::digest(input.as_bytes());
    hex::encode(hash)
}

pub fn basic_auth(uuid: &str, token: &str) -> String {
    format!("Basic {}", STANDARD.encode(format!("{}:{}", uuid, token)))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn generate_user_id() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}
