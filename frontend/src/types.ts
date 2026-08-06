export type ProxyMode = "auto" | "on" | "off";
export type KimiCredentialBackend = "keychain" | "encrypted_vault";
export type SufficiencyState = "enough" | "tight" | "not_enough" | "unknown";
export type ServiceKind = "kimi" | "codex";
export type CardStatus =
  | "fresh"
  | "stale"
  | "update_failed"
  | "login_expired"
  | "no_data";

export interface QuotaTier {
  name: string;
  utilization: number;
  resetsAt?: string | null;
  used?: number | null;
  limit?: number | null;
  remaining?: number | null;
}

export interface QuotaEstimate {
  state: SufficiencyState;
  projectedUtilization?: number | null;
  resetInSecs?: number | null;
  lastsForSecs?: number | null;
  exhaustedAtSecs?: number | null;
  exhaustedBeforeResetSecs?: number | null;
  slopePctPerHour?: number | null;
  trendWindowHours?: number | null;
  observedSpanSecs?: number | null;
  windowStartSecs?: number | null;
  windowEndSecs?: number | null;
  observedPoints?: UsageChartPoint[];
  projectedPoints?: UsageChartPoint[];
}

export interface UsageChartPoint {
  observedAtSecs: number;
  utilization: number;
}

export interface ServiceQuota {
  service: string;
  displayName: string;
  success: boolean;
  tiers: QuotaTier[];
  error?: string | null;
  queriedAt?: number | null;
  credentialValid: boolean;
}

export interface ServiceProxyConfig {
  mode: ProxyMode;
  proxyUrl?: string | null;
  autoPorts: number[];
  timeoutMs: number;
}

export interface ProxySettings {
  kimi: ServiceProxyConfig;
  codex: ServiceProxyConfig;
}

export interface CredentialSettings {
  kimiBackend: KimiCredentialBackend;
}

export interface StatusBarDisplayConfig {
  showIcon: boolean;
  showPercentage: boolean;
  showStateText: boolean;
}

export interface AppConfig {
  version: number;
  accounts: MonitorAccount[];
  selectedServices: string[];
  statusBarServices: string[];
  statusBarDisplay: StatusBarDisplayConfig;
  firstRunCompleted: boolean;
  proxy: ProxySettings;
  credentials: CredentialSettings;
}

export interface MonitorAccount {
  id: string;
  service: ServiceKind;
  displayName: string;
  providerIdentityHint?: string | null;
  credentialRef: unknown;
  enabled: boolean;
  createdAt: number;
}

export interface TierEstimateView {
  tier: string;
  estimate: QuotaEstimate;
}

export interface ProxyTestResult {
  status: string;
  proxyUrl?: string | null;
  message: string;
}

export interface ProxyStatusView {
  kimi: ProxyTestResult;
  codex: ProxyTestResult;
}

export interface CardSnapshot {
  accountId: string;
  service: ServiceKind;
  serviceDisplayName: string;
  accountDisplayName: string;
  status: CardStatus;
  tiers: QuotaTier[];
  weeklyEstimate?: QuotaEstimate | null;
  proxy: ProxyTestResult;
  queriedAt?: number | null;
  lastSuccessfulAt?: number | null;
  errorMessage?: string | null;
}

export interface DashboardState {
  config: AppConfig;
  cards: CardSnapshot[];
  kimiQuota?: ServiceQuota | null;
  codexQuota?: ServiceQuota | null;
  kimiEstimates: TierEstimateView[];
  codexEstimates: TierEstimateView[];
  proxyStatus: ProxyStatusView;
}
