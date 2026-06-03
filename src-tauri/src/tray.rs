use crate::types::{DashboardState, ServiceQuota, SufficiencyState, TierEstimateView, ToolType};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{ActivationPolicy, AppHandle, Manager};

pub const KIMI_TRAY_ID: &str = "agent-quota-control-kimi";
pub const CODEX_TRAY_ID: &str = "agent-quota-control-codex";

pub fn create_tray(app: &AppHandle, dashboard: &DashboardState) -> Result<(), String> {
    create_service_tray(
        app,
        KIMI_TRAY_ID,
        include_bytes!("../icons/kimi_tray.png"),
        service_title(&dashboard.kimi_quota, &dashboard.kimi_estimates),
        dashboard,
    )?;
    create_service_tray(
        app,
        CODEX_TRAY_ID,
        include_bytes!("../icons/codex_tray.png"),
        service_title(&dashboard.codex_quota, &dashboard.codex_estimates),
        dashboard,
    )?;
    Ok(())
}

fn create_service_tray(
    app: &AppHandle,
    tray_id: &str,
    icon_bytes: &[u8],
    title: String,
    dashboard: &DashboardState,
) -> Result<(), String> {
    let menu = build_menu(app, dashboard)?;
    let icon = tauri::image::Image::from_bytes(icon_bytes)
        .map_err(|e| format!("Failed to load tray icon: {e}"))?;
    TrayIconBuilder::with_id(tray_id)
        .icon(icon)
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
            } else if let Some(tool_id) = id.strip_prefix("tool:") {
                let tool_id = tool_id.to_string();
                tauri::async_runtime::spawn(async move {
                    if let Err(error) = crate::commands::launch_tool(tool_id, None).await {
                        log::warn!("Tray tool launch failed: {error}");
                    }
                });
            } else if id.starts_with("cli_tool:") {
                show_dashboard(app);
            } else if id == "quit" {
                app.exit(0);
            }
        })
        .build(app)
        .map_err(|e| format!("Failed to create tray: {e}"))?;
    Ok(())
}

pub fn update_tray(app: &AppHandle, dashboard: &DashboardState) -> Result<(), String> {
    let menu = build_menu(app, dashboard)?;
    if let Some(tray) = app.tray_by_id(KIMI_TRAY_ID) {
        tray.set_menu(Some(menu))
            .map_err(|e| format!("Failed to update tray menu: {e}"))?;
        let _ = tray.set_title(Some(&service_title(
            &dashboard.kimi_quota,
            &dashboard.kimi_estimates,
        )));
    }

    let menu = build_menu(app, dashboard)?;
    if let Some(tray) = app.tray_by_id(CODEX_TRAY_ID) {
        tray.set_menu(Some(menu))
            .map_err(|e| format!("Failed to update tray menu: {e}"))?;
        let _ = tray.set_title(Some(&service_title(
            &dashboard.codex_quota,
            &dashboard.codex_estimates,
        )));
    }
    Ok(())
}

fn build_menu(app: &AppHandle, dashboard: &DashboardState) -> Result<Menu<tauri::Wry>, String> {
    let open = MenuItem::with_id(app, "open_dashboard", "打开控制台", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let kimi = MenuItem::with_id(
        app,
        "kimi_status",
        format!(
            "Kimi Code: {}",
            service_summary(&dashboard.kimi_quota, &dashboard.kimi_estimates)
        ),
        false,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;
    let codex = MenuItem::with_id(
        app,
        "codex_status",
        format!(
            "Codex: {}",
            service_summary(&dashboard.codex_quota, &dashboard.codex_estimates)
        ),
        false,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;
    let refresh = MenuItem::with_id(app, "refresh_usage", "刷新用量", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let selected_tools = selected_tools_submenu(app, dashboard)?;
    let quit =
        MenuItem::with_id(app, "quit", "退出", true, None::<&str>).map_err(|e| e.to_string())?;
    let sep1 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let sep2 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let sep3 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;

    Menu::with_items(
        app,
        &[
            &open,
            &sep1,
            &kimi,
            &codex,
            &refresh,
            &sep2,
            &selected_tools,
            &sep3,
            &quit,
        ],
    )
    .map_err(|e| e.to_string())
}

fn selected_tools_submenu(
    app: &AppHandle,
    dashboard: &DashboardState,
) -> Result<Submenu<tauri::Wry>, String> {
    let selected = dashboard
        .tools
        .iter()
        .filter(|tool| dashboard.config.selected_tools.contains(&tool.id))
        .collect::<Vec<_>>();
    if selected.is_empty() {
        let empty = MenuItem::with_id(
            app,
            "no_selected_tools",
            "没有已选择工具",
            false,
            None::<&str>,
        )
        .map_err(|e| e.to_string())?;
        return Submenu::with_items(app, "已选择工具", true, &[&empty]).map_err(|e| e.to_string());
    }

    let mut owned_items = Vec::new();
    for tool in selected {
        let label = match tool.tool_type {
            ToolType::IDE => format!("应用：{}", tool.name),
            ToolType::CLI => format!("CLI：{}", tool.name),
        };
        let item_id = match tool.tool_type {
            ToolType::IDE => format!("tool:{}", tool.id),
            ToolType::CLI => format!("cli_tool:{}", tool.id),
        };
        let item = MenuItem::with_id(app, item_id, label, true, None::<&str>)
            .map_err(|e| e.to_string())?;
        owned_items.push(item);
    }
    let refs = owned_items
        .iter()
        .map(|item| item as &dyn tauri::menu::IsMenuItem<tauri::Wry>)
        .collect::<Vec<_>>();
    Submenu::with_items(app, "已选择工具", true, &refs).map_err(|e| e.to_string())
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

fn service_title(quota: &Option<ServiceQuota>, estimates: &[TierEstimateView]) -> String {
    let h = tier_pct(quota, "five_hour")
        .map(|pct| format!("h{pct:.0}%"))
        .unwrap_or_else(|| "h--%".to_string());
    format!("{h} {}", weekly_state(estimates))
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
            },
        }
    }

    #[test]
    fn tray_title_uses_weekly_estimator_state_not_five_hour_or_raw_weekly_threshold() {
        let quota = Some(ServiceQuota {
            service: "codex".to_string(),
            display_name: "Codex".to_string(),
            success: true,
            tiers: vec![
                QuotaTier {
                    name: "five_hour".to_string(),
                    utilization: 10.0,
                    resets_at: None,
                    used: None,
                    limit: None,
                    remaining: None,
                },
                QuotaTier {
                    name: "seven_day".to_string(),
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
                &[estimate("seven_day", SufficiencyState::NotEnough)]
            ),
            "h10% 不够"
        );
    }
}
