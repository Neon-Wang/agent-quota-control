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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QuotaEstimate {
    pub state: SufficiencyState,
    pub projected_utilization: Option<f64>,
    pub reset_in_secs: Option<i64>,
    pub lasts_for_secs: Option<i64>,
    pub exhausted_at_secs: Option<i64>,
    pub exhausted_before_reset_secs: Option<i64>,
}

/// 统一用量查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct ToolInfo {
    pub id: String,
    pub name: String,
    pub tool_type: ToolType,
    pub installed: bool,
    pub install_path: Option<String>,
    /// How to launch: "Cursor", "Visual Studio Code", etc. Passed to `open -a`.
    pub launch_as: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardState {
    pub config: AppConfig,
    pub tools: Vec<ToolInfo>,
    pub kimi_quota: Option<ServiceQuota>,
    pub codex_quota: Option<ServiceQuota>,
    pub kimi_estimates: Vec<TierEstimateView>,
    pub codex_estimates: Vec<TierEstimateView>,
    pub proxy_status: ProxyStatusView,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TierEstimateView {
    pub tier: String,
    pub estimate: QuotaEstimate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyStatusView {
    pub kimi: ProxyTestResult,
    pub codex: ProxyTestResult,
}

// ── Config types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub version: u32,
    #[serde(alias = "selected_services")]
    pub selected_services: Vec<String>,
    #[serde(alias = "selected_tools")]
    pub selected_tools: Vec<String>,
    #[serde(alias = "first_run_completed")]
    pub first_run_completed: bool,
    #[serde(default)]
    pub proxy: ProxySettings,
    #[serde(default)]
    pub credentials: CredentialSettings,
    #[serde(default)]
    pub quota_events: QuotaEventStore,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 2,
            selected_services: vec!["kimi".to_string(), "codex".to_string()],
            selected_tools: vec![],
            first_run_completed: false,
            proxy: ProxySettings::default(),
            credentials: CredentialSettings::default(),
            quota_events: QuotaEventStore::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProxyMode {
    Auto,
    On,
    Off,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ServiceProxyConfig {
    pub mode: ProxyMode,
    pub proxy_url: Option<String>,
    pub auto_ports: Vec<u16>,
    pub timeout_ms: u64,
}

impl Default for ServiceProxyConfig {
    fn default() -> Self {
        Self {
            mode: ProxyMode::Auto,
            proxy_url: None,
            auto_ports: vec![7897, 7890],
            timeout_ms: 250,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProxySettings {
    pub kimi: ServiceProxyConfig,
    pub codex: ServiceProxyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KimiCredentialBackend {
    Keychain,
    EncryptedVault,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CredentialSettings {
    pub kimi_backend: KimiCredentialBackend,
}

impl Default for CredentialSettings {
    fn default() -> Self {
        Self {
            kimi_backend: KimiCredentialBackend::Keychain,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QuotaEventStore {
    pub weekly_saturation: Vec<QuotaSaturationEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QuotaSaturationEvent {
    pub service: String,
    pub tier: String,
    pub reset_at: String,
    pub reached_at_secs: i64,
    pub utilization_at: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyTestResult {
    pub status: String,
    pub proxy_url: Option<String>,
    pub message: String,
}

// ── Credential state ──

#[derive(Debug, Clone, PartialEq)]
pub enum CredentialState {
    Valid,
    Missing,
    Expired(String),
}
