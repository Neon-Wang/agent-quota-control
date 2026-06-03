use crate::types::KimiCredentialBackend;

pub fn vault_path() -> std::path::PathBuf {
    crate::config::config_dir().join("credentials.vault.json")
}

pub fn store_kimi_api_key(key: &str, backend: &KimiCredentialBackend) -> Result<(), String> {
    match backend {
        KimiCredentialBackend::Keychain => crate::keychain::store_api_key(
            crate::keychain::KIMI_SERVICE,
            crate::keychain::KIMI_ACCOUNT,
            key,
        ),
        KimiCredentialBackend::EncryptedVault => crate::vault::store_secret(&vault_path(), key),
    }
}

pub fn load_kimi_api_key(preferred: &KimiCredentialBackend) -> Result<Option<String>, String> {
    let preferred_value = load_from_backend(preferred)?;
    if preferred_value.is_some() {
        return Ok(preferred_value);
    }

    let fallback = match preferred {
        KimiCredentialBackend::Keychain => KimiCredentialBackend::EncryptedVault,
        KimiCredentialBackend::EncryptedVault => KimiCredentialBackend::Keychain,
    };
    load_from_backend(&fallback)
}

pub fn clear_kimi_api_key(backend: &KimiCredentialBackend) -> Result<(), String> {
    match backend {
        KimiCredentialBackend::Keychain => crate::keychain::delete_api_key(
            crate::keychain::KIMI_SERVICE,
            crate::keychain::KIMI_ACCOUNT,
        ),
        KimiCredentialBackend::EncryptedVault => crate::vault::clear_secret(&vault_path()),
    }
}

fn load_from_backend(backend: &KimiCredentialBackend) -> Result<Option<String>, String> {
    match backend {
        KimiCredentialBackend::Keychain => crate::keychain::load_api_key(
            crate::keychain::KIMI_SERVICE,
            crate::keychain::KIMI_ACCOUNT,
        ),
        KimiCredentialBackend::EncryptedVault => crate::vault::load_secret(&vault_path()),
    }
}
