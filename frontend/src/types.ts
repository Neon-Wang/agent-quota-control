export type ToolType = "cli" | "ide";
export type ProxyMode = "auto" | "on" | "off";
export type KimiCredentialBackend = "keychain" | "encrypted_vault";
export type SufficiencyState = "enough" | "tight" | "not_enough" | "unknown";

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

export interface ToolInfo {
  id: string;
  name: string;
  toolType: ToolType;
  installed: boolean;
  installPath?: string | null;
  launchAs?: string | null;
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

export interface AppConfig {
  version: number;
  selectedServices: string[];
  selectedTools: string[];
  firstRunCompleted: boolean;
  proxy: ProxySettings;
  credentials: CredentialSettings;
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

export interface DashboardState {
  config: AppConfig;
  tools: ToolInfo[];
  kimiQuota?: ServiceQuota | null;
  codexQuota?: ServiceQuota | null;
  kimiEstimates: TierEstimateView[];
  codexEstimates: TierEstimateView[];
  proxyStatus: ProxyStatusView;
}
