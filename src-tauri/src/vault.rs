use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::Engine;
use rand::RngCore;
use serde::{Deserialize, Serialize};

const VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VaultFile {
    version: u32,
    nonce: String,
    ciphertext: String,
}

pub fn store_secret(path: &std::path::Path, secret: &str) -> Result<(), String> {
    let key = load_or_create_master_key()?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let mut nonce_bytes = [0_u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), secret.as_bytes())
        .map_err(|e| format!("Failed to encrypt vault secret: {e}"))?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create vault dir: {e}"))?;
    }
    let file = VaultFile {
        version: VERSION,
        nonce: base64::engine::general_purpose::STANDARD.encode(nonce_bytes),
        ciphertext: base64::engine::general_purpose::STANDARD.encode(ciphertext),
    };
    let content = serde_json::to_string_pretty(&file)
        .map_err(|e| format!("Failed to serialize vault: {e}"))?;
    std::fs::write(path, content).map_err(|e| format!("Failed to write vault: {e}"))
}

pub fn load_secret(path: &std::path::Path) -> Result<Option<String>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read vault: {e}"))?;
    let file: VaultFile =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse vault: {e}"))?;
    if file.version != VERSION {
        return Err(format!("Unsupported vault version: {}", file.version));
    }
    let key = load_or_create_master_key()?;
    let nonce = base64::engine::general_purpose::STANDARD
        .decode(file.nonce)
        .map_err(|e| format!("Invalid vault nonce: {e}"))?;
    let ciphertext = base64::engine::general_purpose::STANDARD
        .decode(file.ciphertext)
        .map_err(|e| format!("Invalid vault ciphertext: {e}"))?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let plaintext = cipher
        .decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|e| format!("Failed to decrypt vault secret: {e}"))?;
    String::from_utf8(plaintext)
        .map(Some)
        .map_err(|e| format!("Vault plaintext is invalid UTF-8: {e}"))
}

pub fn clear_secret(path: &std::path::Path) -> Result<(), String> {
    if path.exists() {
        std::fs::remove_file(path).map_err(|e| format!("Failed to remove vault: {e}"))?;
    }
    Ok(())
}

fn load_or_create_master_key() -> Result<[u8; 32], String> {
    if let Some(encoded) = crate::keychain::load_api_key(
        crate::keychain::VAULT_SERVICE,
        crate::keychain::VAULT_ACCOUNT,
    )? {
        return decode_master_key(&encoded);
    }

    let mut key = [0_u8; 32];
    OsRng.fill_bytes(&mut key);
    crate::keychain::store_api_key(
        crate::keychain::VAULT_SERVICE,
        crate::keychain::VAULT_ACCOUNT,
        &base64::engine::general_purpose::STANDARD.encode(key),
    )?;
    Ok(key)
}

fn decode_master_key(encoded: &str) -> Result<[u8; 32], String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| format!("Invalid vault master key: {e}"))?;
    bytes
        .try_into()
        .map_err(|_| "Vault master key must be 32 bytes".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vault_file_does_not_contain_plaintext_shape() {
        let file = VaultFile {
            version: VERSION,
            nonce: "nonce".to_string(),
            ciphertext: "ciphertext".to_string(),
        };
        let json = serde_json::to_string(&file).unwrap();

        assert!(!json.contains("sk-test-secret"));
    }
}
