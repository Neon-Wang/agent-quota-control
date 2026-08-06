use serde::{Deserialize, Serialize};

// ── Usage types (adapted from cc-switch services/subscription.rs) ──

/// 单个限速窗口（5小时 / 7天）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slope_pct_per_hour: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trend_window_hours: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_span_secs: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_start_secs: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_end_secs: Option<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_points: Vec<UsageChartPoint>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub projected_points: Vec<UsageChartPoint>,
}

impl Default for QuotaEstimate {
    fn default() -> Self {
        Self {
            state: SufficiencyState::Unknown,
            projected_utilization: None,
            reset_in_secs: None,
            lasts_for_secs: None,
            exhausted_at_secs: None,
            exhausted_before_reset_secs: None,
            slope_pct_per_hour: None,
            trend_window_hours: None,
            observed_span_secs: None,
            window_start_secs: None,
            window_end_secs: None,
            observed_points: Vec::new(),
            projected_points: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UsageSample {
    pub service: String,
    pub tier: String,
    pub reset_at: String,
    pub observed_at_secs: i64,
    pub utilization: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UsageChartPoint {
    pub observed_at_secs: i64,
    pub utilization: f64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardState {
    pub config: AppConfig,
    pub cards: Vec<crate::widget_snapshot::CardSnapshot>,
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
    #[serde(default = "default_accounts")]
    pub accounts: Vec<MonitorAccount>,
    #[serde(alias = "selected_services")]
    pub selected_services: Vec<String>,
    #[serde(default = "default_status_bar_services")]
    pub status_bar_services: Vec<String>,
    #[serde(default)]
    pub status_bar_display: StatusBarDisplayConfig,
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
            version: 5,
            accounts: default_accounts(),
            selected_services: vec!["kimi".to_string(), "codex".to_string()],
            status_bar_services: default_status_bar_services(),
            status_bar_display: StatusBarDisplayConfig::default(),
            first_run_completed: false,
            proxy: ProxySettings::default(),
            credentials: CredentialSettings::default(),
            quota_events: QuotaEventStore::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StatusBarDisplayConfig {
    pub show_icon: bool,
    pub show_percentage: bool,
    pub show_state_text: bool,
}

impl Default for StatusBarDisplayConfig {
    fn default() -> Self {
        Self {
            show_icon: true,
            show_percentage: true,
            show_state_text: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceKind {
    Kimi,
    Codex,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CredentialRef {
    LegacyKimi,
    LiveCodex,
    KimiKeychain { account: String },
    KimiVault { account: String },
    CodexKeychain { account: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MonitorAccount {
    pub id: String,
    pub service: ServiceKind,
    pub display_name: String,
    pub provider_identity_hint: Option<String>,
    pub credential_ref: CredentialRef,
    pub enabled: bool,
    pub created_at: i64,
}

fn default_accounts() -> Vec<MonitorAccount> {
    vec![
        MonitorAccount {
            id: "legacy-kimi".to_string(),
            service: ServiceKind::Kimi,
            display_name: "Kimi Code".to_string(),
            provider_identity_hint: None,
            credential_ref: CredentialRef::LegacyKimi,
            enabled: true,
            created_at: 0,
        },
        MonitorAccount {
            id: "legacy-codex".to_string(),
            service: ServiceKind::Codex,
            display_name: "Codex".to_string(),
            provider_identity_hint: None,
            credential_ref: CredentialRef::LiveCodex,
            enabled: true,
            created_at: 0,
        },
    ]
}

fn default_status_bar_services() -> Vec<String> {
    vec!["kimi".to_string(), "codex".to_string()]
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
