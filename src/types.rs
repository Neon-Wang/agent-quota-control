use serde::{Deserialize, Serialize};

// ── Usage types (adapted from cc-switch services/subscription.rs) ──

/// 单个限速窗口（5小时 / 7天）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaTier {
    pub name: String,
    /// 使用百分比 0–100
    pub utilization: f64,
    /// ISO 8601 重置时间
    pub resets_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub used: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remaining: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SufficiencyState {
    Enough,
    Tight,
    NotEnough,
    Unknown,
}

impl SufficiencyState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Enough => "够用",
            Self::Tight => "偏紧",
            Self::NotEnough => "不够",
            Self::Unknown => "未知",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuotaEstimate {
    pub state: SufficiencyState,
    pub projected_utilization: Option<f64>,
    pub reset_in_secs: Option<i64>,
    pub lasts_for_secs: Option<i64>,
}

/// 统一用量查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceQuota {
    pub service: String,
    pub display_name: String,
    pub success: bool,
    pub tiers: Vec<QuotaTier>,
    pub error: Option<String>,
    pub queried_at: Option<i64>,
    pub credential_valid: bool,
}

impl ServiceQuota {
    pub fn empty(service: &str, display_name: &str) -> Self {
        Self {
            service: service.to_string(),
            display_name: display_name.to_string(),
            success: false,
            tiers: vec![],
            error: None,
            queried_at: None,
            credential_valid: false,
        }
    }
}

// ── Harness types ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    #[allow(clippy::upper_case_acronyms)]
    CLI,
    #[allow(clippy::upper_case_acronyms)]
    IDE,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub id: String,
    pub name: String,
    pub tool_type: ToolType,
    pub installed: bool,
    pub install_path: Option<String>,
    /// How to launch: "Cursor", "Visual Studio Code", etc. Passed to `open -a`.
    pub launch_as: Option<String>,
}

// ── Config types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: u32,
    pub selected_services: Vec<String>,
    pub selected_tools: Vec<String>,
    pub first_run_completed: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            selected_services: vec!["kimi".to_string(), "codex".to_string()],
            selected_tools: vec![],
            first_run_completed: false,
        }
    }
}

// ── Credential state ──

#[derive(Debug, Clone, PartialEq)]
pub enum CredentialState {
    Valid,
    Missing,
    Expired(String),
}
