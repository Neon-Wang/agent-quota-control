use crate::types::AppConfig;
use std::path::PathBuf;

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("kimi-code-status")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    if !path.exists() {
        return AppConfig::default();
    }

    match std::fs::read_to_string(&path) {
        Ok(content) => {
            let (config, migrated) = parse_config(&content);
            if migrated {
                save_config(&config);
            }
            config
        }
        Err(e) => {
            log::warn!("Failed to read config file, using defaults: {e}");
            AppConfig::default()
        }
    }
}

fn parse_config(content: &str) -> (AppConfig, bool) {
    let mut config = serde_json::from_str::<AppConfig>(content).unwrap_or_else(|e| {
        log::warn!("Failed to parse config, using defaults: {e}");
        AppConfig::default()
    });
    let migrated = config.version < 5;
    if config.version < 5 {
        config.version = 5;
    }
    (config, migrated)
}

pub fn save_config(config: &AppConfig) {
    if let Err(error) = save_config_checked(config) {
        log::error!("{error}");
    }
}

pub fn save_config_checked(config: &AppConfig) -> Result<(), String> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|error| format!("Failed to create config directory: {error}"))?;
    let content = serde_json::to_string_pretty(config)
        .map_err(|error| format!("Failed to serialize config: {error}"))?;
    std::fs::write(config_path(), content)
        .map_err(|error| format!("Failed to write config: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_config_migrates_to_status_bar_defaults() {
        let (config, migrated) = parse_config(
            r#"{
                "version": 1,
                "selected_services": ["kimi", "codex"],
                "selected_tools": ["codex_cli"],
                "first_run_completed": true
            }"#,
        );

        assert!(migrated);
        assert_eq!(config.version, 5);
        assert_eq!(config.status_bar_services, vec!["kimi", "codex"]);
        assert_eq!(config.accounts.len(), 2);
        assert_eq!(config.accounts[0].id, "legacy-kimi");
        assert_eq!(config.accounts[0].service, crate::types::ServiceKind::Kimi);
        assert_eq!(config.accounts[1].id, "legacy-codex");
        assert_eq!(config.accounts[1].service, crate::types::ServiceKind::Codex);
        assert_eq!(config.proxy.kimi.auto_ports, vec![7897, 7890]);
        assert!(config.quota_events.weekly_saturation.is_empty());
    }

    #[test]
    fn version_four_preserves_an_intentionally_empty_account_list() {
        let (config, migrated) = parse_config(
            r#"{
                "version": 4,
                "selectedServices": [],
                "statusBarServices": [],
                "selectedTools": [],
                "firstRunCompleted": true,
                "accounts": []
            }"#,
        );

        assert!(migrated);
        assert_eq!(config.version, 5);
        assert!(config.accounts.is_empty());
        assert!(config.status_bar_display.show_icon);
        assert!(config.status_bar_display.show_percentage);
        assert!(config.status_bar_display.show_state_text);
    }

    #[test]
    fn version_four_migration_drops_legacy_tool_selection() {
        let (config, migrated) = parse_config(
            r#"{
                "version": 4,
                "selectedServices": ["codex"],
                "statusBarServices": ["codex"],
                "selectedTools": ["cursor", "codex_cli"],
                "firstRunCompleted": true,
                "accounts": []
            }"#,
        );

        assert!(migrated);
        let serialized = serde_json::to_string(&config).expect("config serializes");
        assert!(!serialized.contains("selectedTools"));
        assert!(!serialized.contains("cursor"));
    }
}
