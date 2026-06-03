use crate::providers::{codex::CodexProvider, kimi::KimiProvider, UsageProvider};
use crate::types::{
    DashboardState, KimiCredentialBackend, ProxySettings, ProxyTestResult, ServiceProxyConfig,
    TierEstimateView,
};
use std::sync::{Arc, Mutex};
use tauri::Emitter;

#[derive(Default)]
pub struct AppRuntimeState {
    pub kimi_quota: Option<crate::types::ServiceQuota>,
    pub codex_quota: Option<crate::types::ServiceQuota>,
}

pub type SharedRuntimeState = Arc<Mutex<AppRuntimeState>>;

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
    emit_dashboard_update(&app, &dashboard);
    Ok(dashboard)
}

#[tauri::command]
pub async fn clear_kimi_api_key(
    backend: KimiCredentialBackend,
    state: tauri::State<'_, SharedRuntimeState>,
) -> Result<DashboardState, String> {
    crate::credentials::clear_kimi_api_key(&backend)?;
    dashboard_state(&state)
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
    let kimi_enabled = config
        .selected_services
        .iter()
        .any(|service| service == "kimi");
    let codex_enabled = config
        .selected_services
        .iter()
        .any(|service| service == "codex");

    let kimi_quota = if kimi_enabled {
        Some(KimiProvider::new().query().await)
    } else {
        None
    };
    let codex_quota = if codex_enabled {
        Some(CodexProvider::new().query().await)
    } else {
        None
    };

    if let Some(quota) = &kimi_quota {
        crate::estimator::record_weekly_saturation_events(
            quota,
            &mut config.quota_events.weekly_saturation,
            now,
        );
    }
    if let Some(quota) = &codex_quota {
        crate::estimator::record_weekly_saturation_events(
            quota,
            &mut config.quota_events.weekly_saturation,
            now,
        );
    }
    crate::config::save_config(&config);

    let mut guard = state
        .lock()
        .map_err(|error| format!("Failed to lock runtime state: {error}"))?;
    guard.kimi_quota = kimi_quota;
    guard.codex_quota = codex_quota;
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

    Ok(DashboardState {
        kimi_estimates: estimates_for("kimi", &kimi_quota, &config),
        codex_estimates: estimates_for("codex", &codex_quota, &config),
        proxy_status: crate::types::ProxyStatusView {
            kimi: crate::proxy::test_proxy_config(&config.proxy.kimi),
            codex: crate::proxy::test_proxy_config(&config.proxy.codex),
        },
        config,
        tools,
        kimi_quota,
        codex_quota,
    })
}

pub fn emit_dashboard_update(app: &tauri::AppHandle, dashboard: &DashboardState) {
    if let Err(error) = app.emit("dashboard://updated", dashboard) {
        log::warn!("Failed to emit dashboard update: {error}");
    }
}

fn estimates_for(
    service: &str,
    quota: &Option<crate::types::ServiceQuota>,
    config: &crate::types::AppConfig,
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
                estimate: crate::estimator::estimate_tier_with_saturation(tier, now, event),
            }
        })
        .collect()
}
