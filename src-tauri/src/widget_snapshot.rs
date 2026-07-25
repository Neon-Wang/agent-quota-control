use crate::types::{
    MonitorAccount, ProxyTestResult, QuotaEstimate, QuotaTier, ServiceKind, ServiceQuota,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) const WIDGET_SCHEMA_VERSION: u32 = 1;
pub(crate) const WIDGET_SUPPORT_DIRECTORY: &str = "io.ccswitch.agent-quota-control";
const STALE_AFTER_MS: i64 = 15 * 60 * 1000;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CardStatus {
    Fresh,
    Stale,
    UpdateFailed,
    LoginExpired,
    NoData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardSnapshot {
    pub account_id: String,
    pub service: String,
    pub service_display_name: String,
    pub account_display_name: String,
    pub status: CardStatus,
    pub tiers: Vec<QuotaTier>,
    pub weekly_estimate: Option<QuotaEstimate>,
    pub proxy: ProxyTestResult,
    pub queried_at: Option<i64>,
    pub last_successful_at: Option<i64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WidgetAccount {
    pub id: String,
    pub service: String,
    pub display_name: String,
    pub provider_identity_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WidgetDocument {
    pub schema_version: u32,
    pub generated_at: i64,
    pub accounts: Vec<WidgetAccount>,
    pub cards_by_account: BTreeMap<String, CardSnapshot>,
}

impl WidgetDocument {
    pub(crate) fn empty(generated_at: i64) -> Self {
        Self {
            schema_version: WIDGET_SCHEMA_VERSION,
            generated_at,
            accounts: Vec::new(),
            cards_by_account: BTreeMap::new(),
        }
    }
}

pub(crate) fn build_widget_document(
    accounts: &[MonitorAccount],
    cards: Vec<CardSnapshot>,
    generated_at: i64,
) -> WidgetDocument {
    let card_ids: std::collections::BTreeSet<&str> =
        cards.iter().map(|card| card.account_id.as_str()).collect();
    let enabled_accounts: Vec<&MonitorAccount> = accounts
        .iter()
        .filter(|account| account.enabled && card_ids.contains(account.id.as_str()))
        .collect();
    let enabled_ids: std::collections::BTreeSet<&str> = enabled_accounts
        .iter()
        .map(|account| account.id.as_str())
        .collect();
    WidgetDocument {
        schema_version: WIDGET_SCHEMA_VERSION,
        generated_at,
        accounts: enabled_accounts
            .into_iter()
            .map(|account| WidgetAccount {
                id: account.id.clone(),
                service: service_id(account.service).to_string(),
                display_name: account.display_name.clone(),
                provider_identity_hint: account.provider_identity_hint.clone(),
            })
            .collect(),
        cards_by_account: cards
            .into_iter()
            .filter(|card| enabled_ids.contains(card.account_id.as_str()))
            .map(|card| (card.account_id.clone(), card))
            .collect(),
    }
}

pub(crate) fn publish_to_directory(
    document: &WidgetDocument,
    directory: &Path,
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(directory)
        .map_err(|error| format!("Failed to create widget snapshot directory: {error}"))?;
    let target = directory.join("widget-snapshot.json");
    let temporary = directory.join(format!(
        ".widget-snapshot-{}-{:016x}.tmp",
        std::process::id(),
        rand::random::<u64>()
    ));
    let content = serde_json::to_vec_pretty(document)
        .map_err(|error| format!("Failed to serialize widget snapshot: {error}"))?;
    let publish_result = (|| {
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)
            .map_err(|error| format!("Failed to create widget snapshot: {error}"))?;
        file.write_all(&content)
            .map_err(|error| format!("Failed to write widget snapshot: {error}"))?;
        file.sync_all()
            .map_err(|error| format!("Failed to sync widget snapshot: {error}"))?;
        std::fs::rename(&temporary, &target)
            .map_err(|error| format!("Failed to publish widget snapshot: {error}"))?;
        Ok(target.clone())
    })();
    if publish_result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    publish_result
}

fn widget_support_directory(home: &Path) -> PathBuf {
    home.join("Library")
        .join("Application Support")
        .join(WIDGET_SUPPORT_DIRECTORY)
}

pub(crate) fn shared_container_directory() -> Result<PathBuf, String> {
    if let Some(directory) = std::env::var_os("AGENT_QUOTA_WIDGET_DIR") {
        return Ok(PathBuf::from(directory));
    }
    dirs::home_dir()
        .map(|home| widget_support_directory(&home))
        .ok_or_else(|| "Unable to resolve the macOS widget data directory".to_string())
}

pub(crate) fn publish_dashboard(
    dashboard: &crate::types::DashboardState,
) -> Result<PathBuf, String> {
    let document = build_widget_document(
        &dashboard.config.accounts,
        dashboard.cards.clone(),
        crate::estimator::now_unix_secs() * 1000,
    );
    publish_to_directory(&document, &shared_container_directory()?)
}

pub(crate) fn build_card_snapshot(
    account: &MonitorAccount,
    current_quota: Option<&ServiceQuota>,
    last_successful_quota: Option<&ServiceQuota>,
    weekly_estimate: Option<QuotaEstimate>,
    proxy: ProxyTestResult,
    now_ms: i64,
) -> CardSnapshot {
    let successful_quota = current_quota
        .filter(|quota| quota.success)
        .or(last_successful_quota.filter(|quota| quota.success));
    let status = match current_quota {
        Some(quota) if quota.success => {
            let is_stale = quota
                .queried_at
                .is_none_or(|queried_at| now_ms.saturating_sub(queried_at) > STALE_AFTER_MS);
            if is_stale {
                CardStatus::Stale
            } else {
                CardStatus::Fresh
            }
        }
        Some(_) if successful_quota.is_some() => CardStatus::UpdateFailed,
        Some(quota) if !quota.credential_valid => CardStatus::LoginExpired,
        _ => CardStatus::NoData,
    };
    let error_message = match status {
        CardStatus::UpdateFailed => Some("更新失败，正在显示上次数据".to_string()),
        CardStatus::LoginExpired => Some("登录已失效".to_string()),
        CardStatus::NoData => Some("暂无用量，打开应用检查账号".to_string()),
        CardStatus::Stale => Some("数据较旧".to_string()),
        CardStatus::Fresh => None,
    };
    CardSnapshot {
        account_id: account.id.clone(),
        service: service_id(account.service).to_string(),
        service_display_name: service_display_name(account.service).to_string(),
        account_display_name: account.display_name.clone(),
        status,
        tiers: successful_quota
            .map(|quota| quota.tiers.clone())
            .unwrap_or_default(),
        weekly_estimate,
        proxy,
        queried_at: current_quota.and_then(|quota| quota.queried_at),
        last_successful_at: successful_quota.and_then(|quota| quota.queried_at),
        error_message,
    }
}

pub(crate) fn service_id(service: ServiceKind) -> &'static str {
    match service {
        ServiceKind::Kimi => "kimi",
        ServiceKind::Codex => "codex",
    }
}

fn service_display_name(service: ServiceKind) -> &'static str {
    match service {
        ServiceKind::Kimi => "Kimi Code",
        ServiceKind::Codex => "Codex",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        CredentialRef, MonitorAccount, ProxyTestResult, QuotaTier, ServiceKind, ServiceQuota,
        SufficiencyState,
    };

    const NOW_MS: i64 = 1_000_000;

    fn account() -> MonitorAccount {
        MonitorAccount {
            id: "account-work".to_string(),
            service: ServiceKind::Codex,
            display_name: "工作账号".to_string(),
            provider_identity_hint: Some("acct-...work".to_string()),
            credential_ref: CredentialRef::LiveCodex,
            enabled: true,
            created_at: 1,
        }
    }

    fn quota(success: bool, credential_valid: bool, queried_at: i64) -> ServiceQuota {
        ServiceQuota {
            service: "codex".to_string(),
            display_name: "Codex".to_string(),
            success,
            tiers: if success {
                vec![QuotaTier {
                    name: "seven_day".to_string(),
                    utilization: 42.0,
                    resets_at: Some("2026-07-25T01:00:00Z".to_string()),
                    used: None,
                    limit: None,
                    remaining: None,
                }]
            } else {
                Vec::new()
            },
            error: (!success).then(|| "provider body secret-access-token".to_string()),
            queried_at: Some(queried_at),
            credential_valid,
        }
    }

    #[test]
    fn fresh_quota_builds_the_canonical_account_card() {
        let current = quota(true, true, NOW_MS);
        let card = build_card_snapshot(
            &account(),
            Some(&current),
            None,
            Some(crate::types::QuotaEstimate {
                state: SufficiencyState::Enough,
                projected_utilization: Some(61.0),
                reset_in_secs: None,
                lasts_for_secs: None,
                exhausted_at_secs: None,
                exhausted_before_reset_secs: None,
                ..Default::default()
            }),
            ProxyTestResult {
                status: "direct".to_string(),
                proxy_url: None,
                message: "Direct".to_string(),
            },
            NOW_MS,
        );

        assert_eq!(card.account_id, "account-work");
        assert_eq!(card.account_display_name, "工作账号");
        assert_eq!(card.status, CardStatus::Fresh);
        assert_eq!(card.tiers, current.tiers);
        assert_eq!(
            card.weekly_estimate.unwrap().projected_utilization,
            Some(61.0)
        );
    }

    #[test]
    fn failed_refresh_preserves_the_last_success_without_leaking_provider_error() {
        let current = quota(false, true, NOW_MS);
        let previous = quota(true, true, NOW_MS - 60_000);
        let card = build_card_snapshot(
            &account(),
            Some(&current),
            Some(&previous),
            None,
            ProxyTestResult {
                status: "direct".to_string(),
                proxy_url: None,
                message: "Direct".to_string(),
            },
            NOW_MS,
        );

        assert_eq!(card.status, CardStatus::UpdateFailed);
        assert_eq!(card.tiers, previous.tiers);
        assert_eq!(
            card.error_message.as_deref(),
            Some("更新失败，正在显示上次数据")
        );
        assert!(!serde_json::to_string(&card)
            .unwrap()
            .contains("secret-access-token"));
    }

    #[test]
    fn invalid_credentials_without_history_have_login_expired_status() {
        let current = quota(false, false, NOW_MS);
        let card = build_card_snapshot(
            &account(),
            Some(&current),
            None,
            None,
            ProxyTestResult {
                status: "unavailable".to_string(),
                proxy_url: None,
                message: "Unavailable".to_string(),
            },
            NOW_MS,
        );

        assert_eq!(card.status, CardStatus::LoginExpired);
        assert!(card.tiers.is_empty());
    }

    #[test]
    fn successful_data_older_than_fifteen_minutes_is_stale() {
        let current = quota(true, true, NOW_MS - 15 * 60 * 1000 - 1);
        let card = build_card_snapshot(
            &account(),
            Some(&current),
            None,
            None,
            ProxyTestResult {
                status: "direct".to_string(),
                proxy_url: None,
                message: "Direct".to_string(),
            },
            NOW_MS,
        );

        assert_eq!(card.status, CardStatus::Stale);
    }

    #[test]
    fn widget_document_uses_versioned_camel_case_json() {
        let document = WidgetDocument::empty(NOW_MS);
        let json = serde_json::to_string(&document).unwrap();

        assert!(json.contains("\"schemaVersion\":1"));
        assert!(json.contains("\"generatedAt\":1000000"));
        assert!(json.contains("\"cardsByAccount\":{}"));
        assert!(!json.contains("credential"));
    }

    #[test]
    fn publishing_writes_a_decodable_redacted_snapshot() {
        let temp = tempfile::tempdir().unwrap();
        let document = WidgetDocument::empty(NOW_MS);

        let path = publish_to_directory(&document, temp.path()).expect("publish should succeed");

        assert_eq!(path.file_name().unwrap(), "widget-snapshot.json");
        let content = std::fs::read_to_string(path).unwrap();
        let decoded: WidgetDocument = serde_json::from_str(&content).unwrap();
        assert_eq!(decoded.schema_version, WIDGET_SCHEMA_VERSION);
        assert!(!content.contains("credential"));
        assert!(std::fs::read_dir(temp.path()).unwrap().all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".tmp")));
    }

    #[test]
    fn widget_snapshot_uses_the_shared_application_support_directory() {
        assert_eq!(
            widget_support_directory(Path::new("/Users/tester")),
            PathBuf::from(
                "/Users/tester/Library/Application Support/io.ccswitch.agent-quota-control"
            )
        );
    }

    #[test]
    fn document_indexes_enabled_accounts_and_cards_by_stable_id() {
        let enabled = account();
        let mut disabled = account();
        disabled.id = "account-disabled".to_string();
        disabled.enabled = false;
        let card = build_card_snapshot(
            &enabled,
            Some(&quota(true, true, NOW_MS)),
            None,
            None,
            ProxyTestResult {
                status: "direct".to_string(),
                proxy_url: None,
                message: "Direct".to_string(),
            },
            NOW_MS,
        );

        let document = build_widget_document(&[enabled, disabled], vec![card], NOW_MS);

        assert_eq!(document.accounts.len(), 1);
        assert_eq!(document.accounts[0].id, "account-work");
        assert!(document.cards_by_account.contains_key("account-work"));
        assert!(!document.cards_by_account.contains_key("account-disabled"));
    }

    #[test]
    fn document_omits_accounts_without_an_overview_card() {
        let monitored = account();
        let mut not_monitored = account();
        not_monitored.id = "account-not-monitored".to_string();
        not_monitored.display_name = "未启用监控".to_string();
        let card = build_card_snapshot(
            &monitored,
            Some(&quota(true, true, NOW_MS)),
            None,
            None,
            ProxyTestResult {
                status: "direct".to_string(),
                proxy_url: None,
                message: "Direct".to_string(),
            },
            NOW_MS,
        );

        let document = build_widget_document(&[monitored, not_monitored], vec![card], NOW_MS);

        assert_eq!(document.accounts.len(), 1);
        assert_eq!(document.accounts[0].id, "account-work");
    }
}
