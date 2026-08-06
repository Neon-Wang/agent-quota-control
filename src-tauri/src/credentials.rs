use crate::types::{CredentialRef, KimiCredentialBackend};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CodexAccountCredentials {
    pub access_token: String,
    pub account_id: Option<String>,
    pub is_stale: bool,
}

pub(crate) enum AccountCredentials {
    KimiApiKey(String),
    Codex(CodexAccountCredentials),
}

pub(crate) const ACCOUNT_SECRET_SERVICE: &str = "agent-quota-control-accounts";

fn managed_keychain_location(credential_ref: &CredentialRef) -> Option<(&'static str, &str)> {
    match credential_ref {
        CredentialRef::KimiKeychain { account } | CredentialRef::CodexKeychain { account } => {
            Some((ACCOUNT_SECRET_SERVICE, account.as_str()))
        }
        CredentialRef::LegacyKimi | CredentialRef::LiveCodex | CredentialRef::KimiVault { .. } => {
            None
        }
    }
}

#[derive(Deserialize)]
struct CodexAuthDocument {
    auth_mode: Option<String>,
    tokens: Option<CodexAuthTokens>,
    last_refresh: Option<String>,
}

#[derive(Deserialize)]
struct CodexAuthTokens {
    access_token: Option<String>,
    account_id: Option<String>,
}

pub(crate) fn account_keychain_name(service: &str, account_id: &str) -> String {
    format!("account:{}:{}", service.trim(), account_id.trim())
}

pub(crate) fn parse_codex_import(content: &str) -> Result<CodexAccountCredentials, String> {
    let auth: CodexAuthDocument =
        serde_json::from_str(content).map_err(|_| "Codex 登录文件格式无效".to_string())?;
    if auth.auth_mode.as_deref() != Some("chatgpt") {
        return Err("当前 Codex 登录不是 ChatGPT OAuth 账号".to_string());
    }
    let tokens = auth
        .tokens
        .ok_or_else(|| "Codex 登录缺少 OAuth token".to_string())?;
    let access_token = tokens
        .access_token
        .filter(|token| !token.trim().is_empty())
        .ok_or_else(|| "Codex 登录缺少 access token".to_string())?;
    let is_stale = auth.last_refresh.as_ref().is_some_and(|last_refresh| {
        chrono::DateTime::parse_from_rfc3339(last_refresh)
            .ok()
            .and_then(|refreshed_at| {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .ok()
                    .map(|now| {
                        now.as_secs()
                            .saturating_sub(refreshed_at.timestamp().max(0) as u64)
                    })
            })
            .is_some_and(|age| age > 8 * 24 * 3600)
    });
    Ok(CodexAccountCredentials {
        access_token,
        account_id: tokens.account_id,
        is_stale,
    })
}

pub(crate) fn store_kimi_account_api_key(
    account_id: &str,
    api_key: &str,
    backend: &KimiCredentialBackend,
) -> Result<CredentialRef, String> {
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err("Kimi API Key 不能为空".to_string());
    }
    let account = account_keychain_name("kimi", account_id);
    match backend {
        KimiCredentialBackend::Keychain => {
            crate::keychain::store_api_key(ACCOUNT_SECRET_SERVICE, &account, api_key)?;
            Ok(CredentialRef::KimiKeychain { account })
        }
        KimiCredentialBackend::EncryptedVault => {
            crate::vault::store_secret(&account_vault_path(account_id)?, api_key)?;
            Ok(CredentialRef::KimiVault {
                account: account_id.to_string(),
            })
        }
    }
}

pub(crate) fn import_current_codex_account(
    account_id: &str,
) -> Result<(CredentialRef, Option<String>), String> {
    let content = read_current_codex_auth_document()
        .ok_or_else(|| "未找到 Codex CLI 的 ChatGPT 登录".to_string())?;
    let credentials = parse_codex_import(&content)?;
    let keychain_account = account_keychain_name("codex", account_id);
    crate::keychain::store_api_key(ACCOUNT_SECRET_SERVICE, &keychain_account, &content)?;
    Ok((
        CredentialRef::CodexKeychain {
            account: keychain_account,
        },
        credentials.account_id,
    ))
}

pub(crate) fn load_account_credentials(
    account: &crate::types::MonitorAccount,
) -> Result<AccountCredentials, String> {
    match &account.credential_ref {
        CredentialRef::LegacyKimi => {
            load_kimi_api_key(&crate::config::load_config().credentials.kimi_backend)?
                .map(AccountCredentials::KimiApiKey)
                .ok_or_else(|| "Kimi API Key 未配置".to_string())
        }
        CredentialRef::LiveCodex => read_current_codex_auth_document()
            .ok_or_else(|| "未找到 Codex CLI 的 ChatGPT 登录".to_string())
            .and_then(|content| parse_codex_import(&content))
            .map(AccountCredentials::Codex),
        CredentialRef::KimiKeychain { .. } => load_managed_secret(&account.credential_ref)?
            .map(AccountCredentials::KimiApiKey)
            .ok_or_else(|| "Kimi API Key 未配置".to_string()),
        CredentialRef::KimiVault { account } => {
            crate::vault::load_secret(&account_vault_path(account)?)?
                .map(AccountCredentials::KimiApiKey)
                .ok_or_else(|| "Kimi API Key 未配置".to_string())
        }
        CredentialRef::CodexKeychain { .. } => load_managed_secret(&account.credential_ref)?
            .ok_or_else(|| "Codex 登录已被移除".to_string())
            .and_then(|content| parse_codex_import(&content))
            .map(AccountCredentials::Codex),
    }
}

pub(crate) fn delete_managed_account_credential(
    credential_ref: &CredentialRef,
) -> Result<(), String> {
    if let Some((service, account)) = managed_keychain_location(credential_ref) {
        return crate::keychain::delete_api_key(service, account);
    }
    if let CredentialRef::KimiVault { account } = credential_ref {
        return crate::vault::clear_secret(&account_vault_path(account)?);
    }
    Ok(())
}

fn load_managed_secret(credential_ref: &CredentialRef) -> Result<Option<String>, String> {
    let (service, account) = managed_keychain_location(credential_ref)
        .ok_or_else(|| "凭据引用不是托管的 Keychain 条目".to_string())?;
    crate::keychain::load_api_key(service, account)
}

fn account_vault_path(account_id: &str) -> Result<std::path::PathBuf, String> {
    if account_id.is_empty()
        || !account_id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
    {
        return Err("账号标识不能用于凭据路径".to_string());
    }
    Ok(crate::config::config_dir().join(format!("credentials-{account_id}.vault.json")))
}

fn read_current_codex_auth_document() -> Option<String> {
    let keychain_output = std::process::Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            crate::keychain::CODEX_SERVICE,
            "-w",
        ])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|content| content.trim().to_string())
        .filter(|content| !content.is_empty());
    if keychain_output.is_some() {
        return keychain_output;
    }
    let path = dirs::home_dir()?.join(".codex").join("auth.json");
    std::fs::read_to_string(path).ok()
}

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

#[cfg(test)]
mod account_tests {
    use super::*;
    use crate::types::CredentialRef;

    #[test]
    fn account_keychain_names_are_isolated_by_service_and_account_id() {
        assert_eq!(
            account_keychain_name("kimi", "account-1"),
            "account:kimi:account-1"
        );
        assert_eq!(
            account_keychain_name("codex", "account-1"),
            "account:codex:account-1"
        );
    }

    #[test]
    fn codex_import_requires_chatgpt_auth_and_returns_the_account_id() {
        let imported = parse_codex_import(
            r#"{
                "auth_mode": "chatgpt",
                "tokens": {
                    "access_token": "secret-access-token",
                    "account_id": "acct-work"
                },
                "last_refresh": "2099-07-21T08:00:00Z"
            }"#,
        )
        .expect("chatgpt auth should import");

        assert_eq!(imported.account_id.as_deref(), Some("acct-work"));
        assert_eq!(imported.access_token, "secret-access-token");
        assert!(!imported.is_stale);
    }

    #[test]
    fn codex_import_rejects_api_key_auth() {
        let error =
            parse_codex_import(r#"{"auth_mode":"apikey","tokens":{"access_token":"secret"}}"#)
                .expect_err("API key auth has no quota endpoint identity");

        assert_eq!(error, "当前 Codex 登录不是 ChatGPT OAuth 账号");
    }

    #[test]
    fn managed_credentials_resolve_to_the_app_owned_keychain_service() {
        let kimi = CredentialRef::KimiKeychain {
            account: "account:kimi:one".to_string(),
        };
        let codex = CredentialRef::CodexKeychain {
            account: "account:codex:two".to_string(),
        };

        assert_eq!(
            managed_keychain_location(&kimi),
            Some((ACCOUNT_SECRET_SERVICE, "account:kimi:one"))
        );
        assert_eq!(
            managed_keychain_location(&codex),
            Some((ACCOUNT_SECRET_SERVICE, "account:codex:two"))
        );
        assert_eq!(managed_keychain_location(&CredentialRef::LegacyKimi), None);
        assert_eq!(managed_keychain_location(&CredentialRef::LiveCodex), None);
    }
}
