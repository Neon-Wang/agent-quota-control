use crate::credentials::AccountCredentials;
use crate::providers::{codex::CodexProvider, kimi::KimiProvider};
use crate::types::{
    DashboardState, KimiCredentialBackend, MonitorAccount, ProxySettings, ProxyTestResult,
    ServiceKind, ServiceProxyConfig, TierEstimateView,
};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use tauri::Emitter;

#[derive(Default)]
pub struct AppRuntimeState {
    pub kimi_quota: Option<crate::types::ServiceQuota>,
    pub codex_quota: Option<crate::types::ServiceQuota>,
    pub quotas: BTreeMap<String, crate::types::ServiceQuota>,
    pub last_successful_quotas: BTreeMap<String, crate::types::ServiceQuota>,
}

pub type SharedRuntimeState = Arc<Mutex<AppRuntimeState>>;

fn is_account_monitored(account: &MonitorAccount, selected_services: &[String]) -> bool {
    account.enabled
        && selected_services
            .iter()
            .any(|service| service == crate::widget_snapshot::service_id(account.service))
}

fn credential_failure_quota(account: &MonitorAccount) -> crate::types::ServiceQuota {
    crate::types::ServiceQuota {
        service: crate::widget_snapshot::service_id(account.service).to_string(),
        display_name: match account.service {
            ServiceKind::Kimi => "Kimi Code".to_string(),
            ServiceKind::Codex => "Codex".to_string(),
        },
        success: false,
        tiers: Vec::new(),
        error: Some("账号凭据不可用".to_string()),
        queried_at: Some(crate::estimator::now_unix_secs() * 1000),
        credential_valid: false,
    }
}

#[tauri::command]
pub async fn get_dashboard_state(
    state: tauri::State<'_, SharedRuntimeState>,
) -> Result<DashboardState, String> {
    dashboard_state(&state)
}

#[tauri::command]
pub async fn refresh_usage(
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedRuntimeState>,
) -> Result<DashboardState, String> {
    refresh_usage_inner(&state).await?;
    let dashboard = dashboard_state(&state)?;
    crate::tray::update_tray(&app, &dashboard)?;
    publish_widget_snapshot(&dashboard);
    emit_dashboard_update(&app, &dashboard);
    Ok(dashboard)
}

#[tauri::command]
pub async fn set_selected_tools(
    tool_ids: Vec<String>,
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedRuntimeState>,
) -> Result<DashboardState, String> {
    let mut config = crate::config::load_config();
    config.selected_tools = tool_ids;
    crate::config::save_config(&config);
    let dashboard = dashboard_state(&state)?;
    crate::tray::update_tray(&app, &dashboard)?;
    publish_widget_snapshot(&dashboard);
    emit_dashboard_update(&app, &dashboard);
    Ok(dashboard)
}

#[tauri::command]
pub async fn set_selected_services(
    service_ids: Vec<String>,
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedRuntimeState>,
) -> Result<DashboardState, String> {
    let mut config = crate::config::load_config();
    config.selected_services = service_ids;
    crate::config::save_config(&config);
    refresh_usage_inner(&state).await?;
    let dashboard = dashboard_state(&state)?;
    crate::tray::update_tray(&app, &dashboard)?;
    publish_widget_snapshot(&dashboard);
    emit_dashboard_update(&app, &dashboard);
    Ok(dashboard)
}

#[tauri::command]
pub async fn set_status_bar_services(
    service_ids: Vec<String>,
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedRuntimeState>,
) -> Result<DashboardState, String> {
    let mut config = crate::config::load_config();
    config.status_bar_services = service_ids;
    crate::config::save_config(&config);
    let dashboard = dashboard_state(&state)?;
    crate::tray::update_tray(&app, &dashboard)?;
    publish_widget_snapshot(&dashboard);
    emit_dashboard_update(&app, &dashboard);
    Ok(dashboard)
}

#[tauri::command]
pub async fn save_proxy_settings(
    settings: ProxySettings,
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedRuntimeState>,
) -> Result<DashboardState, String> {
    let mut config = crate::config::load_config();
    config.proxy = settings;
    crate::config::save_config(&config);
    let dashboard = dashboard_state(&state)?;
    crate::tray::update_tray(&app, &dashboard)?;
    publish_widget_snapshot(&dashboard);
    emit_dashboard_update(&app, &dashboard);
    Ok(dashboard)
}

#[tauri::command]
pub async fn test_proxy(
    _service: String,
    config: ServiceProxyConfig,
) -> Result<ProxyTestResult, String> {
    Ok(crate::proxy::test_proxy_config(&config))
}

#[tauri::command]
pub async fn save_kimi_api_key(
    api_key: String,
    backend: KimiCredentialBackend,
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedRuntimeState>,
) -> Result<DashboardState, String> {
    crate::credentials::store_kimi_api_key(api_key.trim(), &backend)?;
    let mut config = crate::config::load_config();
    config.credentials.kimi_backend = backend;
    crate::config::save_config(&config);
    refresh_usage_inner(&state).await?;
    let dashboard = dashboard_state(&state)?;
    crate::tray::update_tray(&app, &dashboard)?;
    publish_widget_snapshot(&dashboard);
    emit_dashboard_update(&app, &dashboard);
    Ok(dashboard)
}

#[tauri::command]
pub async fn clear_kimi_api_key(
    backend: KimiCredentialBackend,
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedRuntimeState>,
) -> Result<DashboardState, String> {
    crate::credentials::clear_kimi_api_key(&backend)?;
    refresh_usage_inner(&state).await?;
    let dashboard = dashboard_state(&state)?;
    crate::tray::update_tray(&app, &dashboard)?;
    publish_widget_snapshot(&dashboard);
    emit_dashboard_update(&app, &dashboard);
    Ok(dashboard)
}

#[tauri::command]
pub async fn add_kimi_account(
    display_name: String,
    api_key: String,
    backend: KimiCredentialBackend,
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedRuntimeState>,
) -> Result<DashboardState, String> {
    let account_id = crate::accounts::new_account_id();
    let credential_ref =
        crate::credentials::store_kimi_account_api_key(&account_id, &api_key, &backend)?;
    let mut config = crate::config::load_config();
    if let Err(error) = crate::accounts::add_account_with_id(
        &mut config.accounts,
        &account_id,
        ServiceKind::Kimi,
        &display_name,
        credential_ref.clone(),
        crate::estimator::now_unix_secs(),
    ) {
        let _ = crate::credentials::delete_managed_account_credential(&credential_ref);
        return Err(error);
    }
    if let Err(error) = crate::config::save_config_checked(&config) {
        let _ = crate::credentials::delete_managed_account_credential(&credential_ref);
        return Err(error);
    }
    refresh_usage_inner(&state).await?;
    finalize_dashboard(&app, &state)
}

#[tauri::command]
pub async fn import_codex_account(
    display_name: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedRuntimeState>,
) -> Result<DashboardState, String> {
    let account_id = crate::accounts::new_account_id();
    let (credential_ref, provider_identity_hint) =
        crate::credentials::import_current_codex_account(&account_id)?;
    let mut config = crate::config::load_config();
    let account = match crate::accounts::add_account_with_id(
        &mut config.accounts,
        &account_id,
        ServiceKind::Codex,
        &display_name,
        credential_ref.clone(),
        crate::estimator::now_unix_secs(),
    ) {
        Ok(account) => account,
        Err(error) => {
            let _ = crate::credentials::delete_managed_account_credential(&credential_ref);
            return Err(error);
        }
    };
    if let Some(stored) = config
        .accounts
        .iter_mut()
        .find(|stored| stored.id == account.id)
    {
        stored.provider_identity_hint =
            provider_identity_hint.map(|identity| redact_identity(&identity));
    }
    if let Err(error) = crate::config::save_config_checked(&config) {
        let _ = crate::credentials::delete_managed_account_credential(&credential_ref);
        return Err(error);
    }
    refresh_usage_inner(&state).await?;
    finalize_dashboard(&app, &state)
}

#[tauri::command]
pub fn rename_account(
    account_id: String,
    display_name: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedRuntimeState>,
) -> Result<DashboardState, String> {
    let mut config = crate::config::load_config();
    crate::accounts::rename_account(&mut config.accounts, &account_id, &display_name)?;
    crate::config::save_config_checked(&config)?;
    finalize_dashboard(&app, &state)
}

#[tauri::command]
pub fn remove_account(
    account_id: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedRuntimeState>,
) -> Result<DashboardState, String> {
    let mut config = crate::config::load_config();
    let removed = crate::accounts::remove_account(&mut config.accounts, &account_id)?;
    crate::config::save_config_checked(&config)?;
    if let Err(error) =
        crate::credentials::delete_managed_account_credential(&removed.credential_ref)
    {
        log::warn!("Removed account but could not clean its managed credential: {error}");
    }
    if let Err(error) = crate::usage_history::remove_service(estimator_service_key(&removed)) {
        log::warn!("Removed account but could not clean its usage history: {error}");
    }
    let mut guard = state
        .lock()
        .map_err(|error| format!("Failed to lock runtime state: {error}"))?;
    guard.quotas.remove(&account_id);
    guard.last_successful_quotas.remove(&account_id);
    drop(guard);
    finalize_dashboard(&app, &state)
}

#[tauri::command]
pub async fn launch_tool(tool_id: String, project_dir: Option<String>) -> Result<(), String> {
    let tools = crate::harness::scan_tools();
    let tool = tools
        .iter()
        .find(|tool| tool.id == tool_id)
        .ok_or_else(|| format!("Tool not found: {tool_id}"))?;
    crate::launcher::launch_tool(tool, project_dir.as_deref())
}

#[tauri::command]
pub async fn reveal_config_dir() -> Result<(), String> {
    let path = crate::config::config_dir();
    std::fs::create_dir_all(&path).map_err(|e| format!("Failed to create config dir: {e}"))?;
    std::process::Command::new("open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Failed to reveal config dir: {e}"))
}

pub async fn refresh_usage_inner(state: &SharedRuntimeState) -> Result<(), String> {
    let mut config = crate::config::load_config();
    let now = crate::estimator::now_unix_secs();
    let monitored_accounts: Vec<MonitorAccount> = config
        .accounts
        .iter()
        .filter(|account| is_account_monitored(account, &config.selected_services))
        .cloned()
        .collect();
    let mut quotas = BTreeMap::new();
    for account in &monitored_accounts {
        let quota = match crate::credentials::load_account_credentials(account) {
            Ok(AccountCredentials::KimiApiKey(api_key)) => {
                KimiProvider::query_with_api_key(&api_key).await
            }
            Ok(AccountCredentials::Codex(credentials)) => {
                CodexProvider::query_with_credentials(&credentials).await
            }
            Err(error) => {
                log::warn!("Credential unavailable for account {}: {error}", account.id);
                credential_failure_quota(account)
            }
        };
        let estimator_key = estimator_service_key(account);
        crate::estimator::record_weekly_saturation_events_for(
            estimator_key,
            &quota,
            &mut config.quota_events.weekly_saturation,
            now,
        );
        if let Err(error) = crate::usage_history::record_quota_samples(estimator_key, &quota, now) {
            log::warn!(
                "Usage refreshed for account {} but history could not be saved: {error}",
                account.id
            );
        }
        quotas.insert(account.id.clone(), quota);
    }
    crate::config::save_config(&config);

    let mut guard = state
        .lock()
        .map_err(|error| format!("Failed to lock runtime state: {error}"))?;
    guard.last_successful_quotas.retain(|account_id, _| {
        config
            .accounts
            .iter()
            .any(|account| account.id == *account_id)
    });
    for (account_id, quota) in &quotas {
        if quota.success {
            guard
                .last_successful_quotas
                .insert(account_id.clone(), quota.clone());
        }
    }
    guard.kimi_quota = first_service_quota(&monitored_accounts, &quotas, ServiceKind::Kimi);
    guard.codex_quota = first_service_quota(&monitored_accounts, &quotas, ServiceKind::Codex);
    guard.quotas = quotas;
    Ok(())
}

pub fn dashboard_state(state: &SharedRuntimeState) -> Result<DashboardState, String> {
    let config = crate::config::load_config();
    let tools = crate::harness::scan_tools();
    let guard = state
        .lock()
        .map_err(|error| format!("Failed to lock runtime state: {error}"))?;
    let kimi_quota = guard.kimi_quota.clone();
    let codex_quota = guard.codex_quota.clone();
    let proxy_status = crate::types::ProxyStatusView {
        kimi: crate::proxy::test_proxy_config(&config.proxy.kimi),
        codex: crate::proxy::test_proxy_config(&config.proxy.codex),
    };
    let now_ms = crate::estimator::now_unix_secs() * 1000;
    let usage_history = crate::usage_history::load_history();
    let cards = config
        .accounts
        .iter()
        .filter(|account| is_account_monitored(account, &config.selected_services))
        .map(|account| {
            let quota = guard.quotas.get(&account.id);
            let last_successful_quota = guard.last_successful_quotas.get(&account.id);
            let estimation_quota = quota
                .filter(|current| current.success)
                .or(last_successful_quota)
                .cloned();
            let estimates = estimates_for(
                estimator_service_key(account),
                &estimation_quota,
                &config,
                &usage_history,
            );
            let weekly_estimate = estimates
                .iter()
                .find(|entry| matches!(entry.tier.as_str(), "weekly_limit" | "seven_day"))
                .map(|entry| entry.estimate.clone());
            let proxy = match account.service {
                ServiceKind::Kimi => proxy_status.kimi.clone(),
                ServiceKind::Codex => proxy_status.codex.clone(),
            };
            crate::widget_snapshot::build_card_snapshot(
                account,
                quota,
                last_successful_quota,
                weekly_estimate,
                proxy,
                now_ms,
            )
        })
        .collect();

    Ok(DashboardState {
        kimi_estimates: estimates_for("kimi", &kimi_quota, &config, &usage_history),
        codex_estimates: estimates_for("codex", &codex_quota, &config, &usage_history),
        proxy_status,
        config,
        tools,
        cards,
        kimi_quota,
        codex_quota,
    })
}

fn first_service_quota(
    accounts: &[MonitorAccount],
    quotas: &BTreeMap<String, crate::types::ServiceQuota>,
    service: ServiceKind,
) -> Option<crate::types::ServiceQuota> {
    accounts
        .iter()
        .find(|account| account.service == service)
        .and_then(|account| quotas.get(&account.id))
        .cloned()
}

fn estimator_service_key(account: &MonitorAccount) -> &str {
    match account.id.as_str() {
        "legacy-kimi" => "kimi",
        "legacy-codex" => "codex",
        _ => account.id.as_str(),
    }
}

pub fn emit_dashboard_update(app: &tauri::AppHandle, dashboard: &DashboardState) {
    if let Err(error) = app.emit("dashboard://updated", dashboard) {
        log::warn!("Failed to emit dashboard update: {error}");
    }
}

pub fn publish_widget_snapshot(dashboard: &DashboardState) {
    match crate::widget_snapshot::publish_dashboard(dashboard) {
        Ok(_) => {
            if let Err(error) = crate::widget_reload::reload_timelines() {
                log::debug!("Widget snapshot published without timeline reload: {error}");
            }
        }
        Err(error) => log::warn!("Failed to publish Widget snapshot: {error}"),
    }
}

fn finalize_dashboard(
    app: &tauri::AppHandle,
    state: &SharedRuntimeState,
) -> Result<DashboardState, String> {
    let dashboard = dashboard_state(state)?;
    crate::tray::update_tray(app, &dashboard)?;
    publish_widget_snapshot(&dashboard);
    emit_dashboard_update(app, &dashboard);
    Ok(dashboard)
}

fn redact_identity(identity: &str) -> String {
    let characters: Vec<char> = identity.trim().chars().collect();
    if characters.len() <= 8 {
        return "已导入".to_string();
    }
    let prefix: String = characters.iter().take(4).collect();
    let suffix: String = characters.iter().skip(characters.len() - 4).collect();
    format!("{prefix}...{suffix}")
}

fn estimates_for(
    service: &str,
    quota: &Option<crate::types::ServiceQuota>,
    config: &crate::types::AppConfig,
    history: &crate::usage_history::UsageHistoryDocument,
) -> Vec<TierEstimateView> {
    let now = crate::estimator::now_unix_secs();
    quota
        .as_ref()
        .filter(|quota| quota.success)
        .into_iter()
        .flat_map(|quota| quota.tiers.iter())
        .map(|tier| {
            let event = crate::estimator::matching_saturation_event(
                tier,
                service,
                &config.quota_events.weekly_saturation,
            );
            TierEstimateView {
                tier: tier.name.clone(),
                estimate: if matches!(tier.name.as_str(), "weekly_limit" | "seven_day") {
                    let samples = crate::usage_history::samples_for_tier(history, service, tier);
                    crate::estimator::estimate_recent_weekly_trend(tier, now, &samples, event)
                } else {
                    crate::estimator::estimate_tier_with_saturation(tier, now, event)
                },
            }
        })
        .collect()
}

#[cfg(test)]
mod account_runtime_tests {
    use super::*;
    use crate::types::{
        CredentialRef, MonitorAccount, QuotaTier, ServiceKind, ServiceQuota, UsageSample,
    };

    fn account(service: ServiceKind, enabled: bool) -> MonitorAccount {
        MonitorAccount {
            id: "account-test".to_string(),
            service,
            display_name: "Test".to_string(),
            provider_identity_hint: None,
            credential_ref: CredentialRef::LiveCodex,
            enabled,
            created_at: 0,
        }
    }

    #[test]
    fn monitored_accounts_require_both_account_and_service_to_be_enabled() {
        assert!(is_account_monitored(
            &account(ServiceKind::Codex, true),
            &["codex".to_string()]
        ));
        assert!(!is_account_monitored(
            &account(ServiceKind::Codex, false),
            &["codex".to_string()]
        ));
        assert!(!is_account_monitored(
            &account(ServiceKind::Codex, true),
            &["kimi".to_string()]
        ));
    }

    #[test]
    fn credential_failure_quota_is_safe_and_invalid() {
        let quota = credential_failure_quota(&account(ServiceKind::Codex, true));

        assert!(!quota.success);
        assert!(!quota.credential_valid);
        assert!(quota.tiers.is_empty());
        assert_eq!(quota.error.as_deref(), Some("账号凭据不可用"));
    }

    #[test]
    fn provider_identity_redaction_is_unicode_safe() {
        assert_eq!(redact_identity("账号标识很长ABCDE"), "账号标识...BCDE");
        assert_eq!(redact_identity("short"), "已导入");
    }

    #[test]
    fn weekly_estimates_use_matching_recent_history() {
        let now = crate::estimator::now_unix_secs();
        let reset_at = chrono::DateTime::from_timestamp(now + 86_400, 0)
            .unwrap()
            .to_rfc3339();
        let weekly = QuotaTier {
            name: "seven_day".to_string(),
            utilization: 70.0,
            resets_at: Some(reset_at.clone()),
            used: None,
            limit: None,
            remaining: None,
        };
        let quota = Some(ServiceQuota {
            service: "codex".to_string(),
            display_name: "Codex".to_string(),
            success: true,
            tiers: vec![weekly],
            error: None,
            queried_at: Some(now * 1_000),
            credential_valid: true,
        });
        let history = crate::usage_history::UsageHistoryDocument {
            version: 1,
            samples: vec![
                UsageSample {
                    service: "account-test".to_string(),
                    tier: "seven_day".to_string(),
                    reset_at: reset_at.clone(),
                    observed_at_secs: now - 7_200,
                    utilization: 30.0,
                },
                UsageSample {
                    service: "account-test".to_string(),
                    tier: "seven_day".to_string(),
                    reset_at: reset_at.clone(),
                    observed_at_secs: now - 3_600,
                    utilization: 45.0,
                },
                UsageSample {
                    service: "account-test".to_string(),
                    tier: "seven_day".to_string(),
                    reset_at,
                    observed_at_secs: now,
                    utilization: 70.0,
                },
            ],
        };

        let estimates = estimates_for(
            "account-test",
            &quota,
            &crate::types::AppConfig::default(),
            &history,
        );

        assert_eq!(estimates[0].estimate.trend_window_hours, Some(24));
        assert!(estimates[0].estimate.slope_pct_per_hour.is_some());
        assert_eq!(estimates[0].estimate.observed_points.len(), 3);
        assert_eq!(estimates[0].estimate.projected_points.len(), 2);
    }
}
