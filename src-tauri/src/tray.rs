use crate::types::{
    DashboardState, ServiceQuota, StatusBarDisplayConfig, SufficiencyState, TierEstimateView,
};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{ActivationPolicy, AppHandle, Manager};

pub const KIMI_TRAY_ID: &str = "agent-quota-control-kimi";
pub const CODEX_TRAY_ID: &str = "agent-quota-control-codex";

pub fn create_tray(app: &AppHandle, dashboard: &DashboardState) -> Result<(), String> {
    create_service_tray(
        app,
        KIMI_TRAY_ID,
        include_bytes!("../icons/kimi_tray.png"),
        service_title(
            &dashboard.kimi_quota,
            &dashboard.kimi_estimates,
            &dashboard.config.status_bar_display,
        ),
        kimi_tray_visible(dashboard),
        dashboard,
    )?;
    create_service_tray(
        app,
        CODEX_TRAY_ID,
        include_bytes!("../icons/codex_tray.png"),
        service_title(
            &dashboard.codex_quota,
            &dashboard.codex_estimates,
            &dashboard.config.status_bar_display,
        ),
        codex_tray_visible(dashboard),
        dashboard,
    )?;
    Ok(())
}

fn create_service_tray(
    app: &AppHandle,
    tray_id: &str,
    icon_bytes: &[u8],
    title: String,
    visible: bool,
    dashboard: &DashboardState,
) -> Result<(), String> {
    let menu = build_menu(app, dashboard)?;
    let mut tray_builder = TrayIconBuilder::with_id(tray_id)
        .title(title)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if matches!(event, TrayIconEvent::Click { .. }) {
                show_dashboard(tray.app_handle());
            }
        })
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            if id == "open_dashboard" {
                show_dashboard(app);
            } else if id == "refresh_usage" {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Some(state) = app.try_state::<crate::commands::SharedRuntimeState>() {
                        if let Ok(dashboard) =
                            crate::commands::refresh_usage(app.clone(), state).await
                        {
                            let _ = update_tray(&app, &dashboard);
                        }
                    }
                });
            } else if id == "launch_chatgpt_7897" {
                if let Err(error) = crate::chatgpt_launcher::launch_chatgpt_with_7897() {
                    log::warn!("ChatGPT 7897 launch failed: {error}");
                }
                if let Some(state) = app.try_state::<crate::commands::SharedRuntimeState>() {
                    if let Ok(dashboard) = crate::commands::dashboard_state(&state) {
                        let _ = update_tray(app, &dashboard);
                    }
                }
            } else if id == "quit" {
                app.exit(0);
            }
        });
    if dashboard.config.status_bar_display.show_icon {
        let icon = tauri::image::Image::from_bytes(icon_bytes)
            .map_err(|e| format!("Failed to load tray icon: {e}"))?;
        tray_builder = tray_builder.icon(icon);
    }
    let tray = tray_builder
        .build(app)
        .map_err(|e| format!("Failed to create tray: {e}"))?;
    tray.set_visible(visible)
        .map_err(|e| format!("Failed to set tray visibility: {e}"))?;
    Ok(())
}

pub fn update_tray(app: &AppHandle, dashboard: &DashboardState) -> Result<(), String> {
    let menu = build_menu(app, dashboard)?;
    if let Some(tray) = app.tray_by_id(KIMI_TRAY_ID) {
        tray.set_menu(Some(menu))
            .map_err(|e| format!("Failed to update tray menu: {e}"))?;
        let _ = tray.set_visible(kimi_tray_visible(dashboard));
        let _ = tray.set_icon(service_icon(
            include_bytes!("../icons/kimi_tray.png"),
            dashboard.config.status_bar_display.show_icon,
        )?);
        let _ = tray.set_title(Some(&service_title(
            &dashboard.kimi_quota,
            &dashboard.kimi_estimates,
            &dashboard.config.status_bar_display,
        )));
    } else if kimi_tray_visible(dashboard) {
        create_service_tray(
            app,
            KIMI_TRAY_ID,
            include_bytes!("../icons/kimi_tray.png"),
            service_title(
                &dashboard.kimi_quota,
                &dashboard.kimi_estimates,
                &dashboard.config.status_bar_display,
            ),
            true,
            dashboard,
        )?;
    }

    let menu = build_menu(app, dashboard)?;
    if let Some(tray) = app.tray_by_id(CODEX_TRAY_ID) {
        tray.set_menu(Some(menu))
            .map_err(|e| format!("Failed to update tray menu: {e}"))?;
        let _ = tray.set_visible(codex_tray_visible(dashboard));
        let _ = tray.set_icon(service_icon(
            include_bytes!("../icons/codex_tray.png"),
            dashboard.config.status_bar_display.show_icon,
        )?);
        let _ = tray.set_title(Some(&service_title(
            &dashboard.codex_quota,
            &dashboard.codex_estimates,
            &dashboard.config.status_bar_display,
        )));
    } else if codex_tray_visible(dashboard) {
        create_service_tray(
            app,
            CODEX_TRAY_ID,
            include_bytes!("../icons/codex_tray.png"),
            service_title(
                &dashboard.codex_quota,
                &dashboard.codex_estimates,
                &dashboard.config.status_bar_display,
            ),
            true,
            dashboard,
        )?;
    }
    Ok(())
}

fn build_menu(app: &AppHandle, dashboard: &DashboardState) -> Result<Menu<tauri::Wry>, String> {
    let open = MenuItem::with_id(app, "open_dashboard", "打开控制台", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let refresh = MenuItem::with_id(app, "refresh_usage", "刷新用量", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let chatgpt_installed = crate::chatgpt_launcher::is_chatgpt_installed();
    let chatgpt_running = chatgpt_installed && crate::chatgpt_launcher::is_chatgpt_running();
    let (chatgpt_label, chatgpt_enabled) = if !chatgpt_installed {
        ("未找到 ChatGPT.app", false)
    } else if chatgpt_running {
        ("ChatGPT 已运行，请先完全退出", false)
    } else {
        ("通过 7897 代理打开 ChatGPT", true)
    };
    let launch_chatgpt = MenuItem::with_id(
        app,
        "launch_chatgpt_7897",
        chatgpt_label,
        chatgpt_enabled,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;
    let quit =
        MenuItem::with_id(app, "quit", "退出", true, None::<&str>).map_err(|e| e.to_string())?;
    let sep1 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let sep2 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let sep3 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;

    let mut items: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = vec![&open, &sep1];
    let kimi_status = if kimi_tray_visible(dashboard) {
        Some(
            MenuItem::with_id(
                app,
                "kimi_status",
                format!(
                    "Kimi Code: {}",
                    service_summary(&dashboard.kimi_quota, &dashboard.kimi_estimates)
                ),
                false,
                None::<&str>,
            )
            .map_err(|e| e.to_string())?,
        )
    } else {
        None
    };
    if let Some(item) = &kimi_status {
        items.push(item);
    }
    let codex_status = if codex_tray_visible(dashboard) {
        Some(
            MenuItem::with_id(
                app,
                "codex_status",
                format!(
                    "Codex: {}",
                    service_summary(&dashboard.codex_quota, &dashboard.codex_estimates)
                ),
                false,
                None::<&str>,
            )
            .map_err(|e| e.to_string())?,
        )
    } else {
        None
    };
    if let Some(item) = &codex_status {
        items.push(item);
    }
    items.push(&refresh);
    items.push(&sep2);
    items.push(&launch_chatgpt);
    items.push(&sep3);
    items.push(&quit);

    Menu::with_items(app, &items).map_err(|e| e.to_string())
}

fn service_icon(
    icon_bytes: &'static [u8],
    show_icon: bool,
) -> Result<Option<tauri::image::Image<'static>>, String> {
    show_icon
        .then(|| {
            tauri::image::Image::from_bytes(icon_bytes)
                .map_err(|e| format!("Failed to load tray icon: {e}"))
        })
        .transpose()
}

fn show_dashboard(app: &AppHandle) {
    let _ = app.set_activation_policy(ActivationPolicy::Regular);
    let _ = app.set_dock_visibility(true);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn service_summary(quota: &Option<ServiceQuota>, estimates: &[TierEstimateView]) -> String {
    let h = tier_pct(quota, "five_hour")
        .map(|pct| format!("h{pct:.0}%"))
        .unwrap_or_else(|| "h--%".to_string());
    let w = weekly_pct(quota)
        .map(|pct| format!("w{pct:.0}%"))
        .unwrap_or_else(|| "w--%".to_string());
    format!("{h} · {w} · {}", weekly_state(estimates))
}

fn service_title(
    quota: &Option<ServiceQuota>,
    estimates: &[TierEstimateView],
    display: &StatusBarDisplayConfig,
) -> String {
    let mut parts = Vec::new();
    if display.show_percentage {
        parts.push(
            tier_pct(quota, "five_hour")
                .map(|pct| format!("h{pct:.0}%"))
                .or_else(|| weekly_pct(quota).map(|pct| format!("w{pct:.0}%")))
                .unwrap_or_else(|| "w--%".to_string()),
        );
    }
    if display.show_state_text {
        parts.push(weekly_state(estimates).to_string());
    }
    parts.join(" ")
}

fn kimi_tray_visible(dashboard: &DashboardState) -> bool {
    service_tray_enabled(dashboard, "kimi")
        && crate::credentials::load_kimi_api_key(&dashboard.config.credentials.kimi_backend)
            .ok()
            .flatten()
            .is_some()
}

fn codex_tray_visible(dashboard: &DashboardState) -> bool {
    service_tray_enabled(dashboard, "codex")
}

fn service_tray_enabled(dashboard: &DashboardState, service: &str) -> bool {
    dashboard
        .config
        .selected_services
        .iter()
        .any(|id| id == service)
        && dashboard
            .config
            .status_bar_services
            .iter()
            .any(|id| id == service)
        && status_bar_has_content(&dashboard.config.status_bar_display)
}

fn status_bar_has_content(display: &StatusBarDisplayConfig) -> bool {
    display.show_icon || display.show_percentage || display.show_state_text
}

fn weekly_state(estimates: &[TierEstimateView]) -> &'static str {
    let state = estimates
        .iter()
        .find(|entry| matches!(entry.tier.as_str(), "weekly_limit" | "seven_day"))
        .map(|entry| &entry.estimate.state);
    match state {
        Some(SufficiencyState::Enough) => "够",
        Some(SufficiencyState::Tight) => "偏紧",
        Some(SufficiencyState::NotEnough) => "不够",
        _ => "未知",
    }
}

fn weekly_pct(quota: &Option<ServiceQuota>) -> Option<f64> {
    quota
        .as_ref()
        .filter(|quota| quota.success)
        .and_then(|quota| {
            quota
                .tiers
                .iter()
                .find(|tier| matches!(tier.name.as_str(), "weekly_limit" | "seven_day"))
                .map(|tier| tier.utilization)
        })
}

fn tier_pct(quota: &Option<ServiceQuota>, name: &str) -> Option<f64> {
    quota
        .as_ref()
        .filter(|quota| quota.success)
        .and_then(|quota| {
            quota
                .tiers
                .iter()
                .find(|tier| tier.name == name)
                .map(|tier| tier.utilization)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{QuotaEstimate, QuotaTier};

    fn estimate(tier: &str, state: SufficiencyState) -> TierEstimateView {
        TierEstimateView {
            tier: tier.to_string(),
            estimate: QuotaEstimate {
                state,
                projected_utilization: None,
                reset_in_secs: None,
                lasts_for_secs: None,
                exhausted_at_secs: None,
                exhausted_before_reset_secs: None,
                ..Default::default()
            },
        }
    }

    #[test]
    fn tray_title_uses_weekly_utilization_and_estimator_state() {
        let quota = Some(ServiceQuota {
            service: "codex".to_string(),
            display_name: "Codex".to_string(),
            success: true,
            tiers: vec![QuotaTier {
                name: "seven_day".to_string(),
                utilization: 70.0,
                resets_at: None,
                used: None,
                limit: None,
                remaining: None,
            }],
            error: None,
            queried_at: None,
            credential_valid: true,
        });

        assert_eq!(
            service_title(
                &quota,
                &[estimate("seven_day", SufficiencyState::NotEnough)],
                &crate::types::StatusBarDisplayConfig::default(),
            ),
            "w70% 不够"
        );
    }

    #[test]
    fn tray_title_keeps_five_hour_utilization_when_the_service_provides_it() {
        let quota = Some(ServiceQuota {
            service: "kimi".to_string(),
            display_name: "Kimi Code".to_string(),
            success: true,
            tiers: vec![
                QuotaTier {
                    name: "five_hour".to_string(),
                    utilization: 12.0,
                    resets_at: None,
                    used: None,
                    limit: None,
                    remaining: None,
                },
                QuotaTier {
                    name: "weekly_limit".to_string(),
                    utilization: 70.0,
                    resets_at: None,
                    used: None,
                    limit: None,
                    remaining: None,
                },
            ],
            error: None,
            queried_at: None,
            credential_valid: true,
        });

        assert_eq!(
            service_title(
                &quota,
                &[estimate("weekly_limit", SufficiencyState::Enough)],
                &crate::types::StatusBarDisplayConfig::default(),
            ),
            "h12% 够"
        );
    }

    #[test]
    fn tray_title_supports_independent_percentage_and_state_switches() {
        let quota = Some(ServiceQuota {
            service: "codex".to_string(),
            display_name: "Codex".to_string(),
            success: true,
            tiers: vec![QuotaTier {
                name: "seven_day".to_string(),
                utilization: 70.0,
                resets_at: None,
                used: None,
                limit: None,
                remaining: None,
            }],
            error: None,
            queried_at: None,
            credential_valid: true,
        });
        let estimates = [estimate("seven_day", SufficiencyState::Unknown)];

        assert_eq!(
            service_title(
                &quota,
                &estimates,
                &crate::types::StatusBarDisplayConfig {
                    show_icon: true,
                    show_percentage: true,
                    show_state_text: false,
                },
            ),
            "w70%"
        );
        assert_eq!(
            service_title(
                &quota,
                &estimates,
                &crate::types::StatusBarDisplayConfig {
                    show_icon: true,
                    show_percentage: false,
                    show_state_text: true,
                },
            ),
            "未知"
        );
        assert_eq!(
            service_title(
                &quota,
                &estimates,
                &crate::types::StatusBarDisplayConfig {
                    show_icon: true,
                    show_percentage: false,
                    show_state_text: false,
                },
            ),
            ""
        );
    }

    #[test]
    fn tray_requires_at_least_one_visible_display_element() {
        assert!(status_bar_has_content(&StatusBarDisplayConfig::default()));
        assert!(!status_bar_has_content(&StatusBarDisplayConfig {
            show_icon: false,
            show_percentage: false,
            show_state_text: false,
        }));
    }
}
