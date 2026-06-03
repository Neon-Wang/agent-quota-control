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
    let migrated = config.version < 2;
    if config.version < 2 {
        config.version = 2;
    }
    (config, migrated)
}

pub fn save_config(config: &AppConfig) {
    let dir = config_dir();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        log::error!("Failed to create config directory: {e}");
        return;
    }

    match serde_json::to_string_pretty(config) {
        Ok(content) => {
            if let Err(e) = std::fs::write(config_path(), content) {
                log::error!("Failed to write config: {e}");
            }
        }
        Err(e) => {
            log::error!("Failed to serialize config: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1_config_migrates_to_v2_defaults() {
        let (config, migrated) = parse_config(
            r#"{
                "version": 1,
                "selected_services": ["kimi", "codex"],
                "selected_tools": ["codex_cli"],
                "first_run_completed": true
            }"#,
        );

        assert!(migrated);
        assert_eq!(config.version, 2);
        assert_eq!(config.selected_tools, vec!["codex_cli"]);
        assert_eq!(config.proxy.kimi.auto_ports, vec![7897, 7890]);
        assert!(config.quota_events.weekly_saturation.is_empty());
    }
}
