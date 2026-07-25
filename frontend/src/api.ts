import { invoke } from "@tauri-apps/api/core";
import type {
  DashboardState,
  KimiCredentialBackend,
  ProxySettings,
  ProxyTestResult,
  ServiceProxyConfig,
} from "./types";

export const api = {
  getDashboardState: () => invoke<DashboardState>("get_dashboard_state"),
  refreshUsage: () => invoke<DashboardState>("refresh_usage"),
  setSelectedTools: (toolIds: string[]) =>
    invoke<DashboardState>("set_selected_tools", { toolIds }),
  setSelectedServices: (serviceIds: string[]) =>
    invoke<DashboardState>("set_selected_services", { serviceIds }),
  setStatusBarServices: (serviceIds: string[]) =>
    invoke<DashboardState>("set_status_bar_services", { serviceIds }),
  saveProxySettings: (settings: ProxySettings) =>
    invoke<DashboardState>("save_proxy_settings", { settings }),
  testProxy: (service: string, config: ServiceProxyConfig) =>
    invoke<ProxyTestResult>("test_proxy", { service, config }),
  saveKimiApiKey: (apiKey: string, backend: KimiCredentialBackend) =>
    invoke<DashboardState>("save_kimi_api_key", { apiKey, backend }),
  clearKimiApiKey: (backend: KimiCredentialBackend) =>
    invoke<DashboardState>("clear_kimi_api_key", { backend }),
  addKimiAccount: (
    displayName: string,
    apiKey: string,
    backend: KimiCredentialBackend,
  ) =>
    invoke<DashboardState>("add_kimi_account", {
      displayName,
      apiKey,
      backend,
    }),
  importCodexAccount: (displayName: string) =>
    invoke<DashboardState>("import_codex_account", { displayName }),
  renameAccount: (accountId: string, displayName: string) =>
    invoke<DashboardState>("rename_account", { accountId, displayName }),
  removeAccount: (accountId: string) =>
    invoke<DashboardState>("remove_account", { accountId }),
  launchTool: (toolId: string, projectDir?: string | null) =>
    invoke<void>("launch_tool", { toolId, projectDir }),
  revealConfigDir: () => invoke<void>("reveal_config_dir"),
};
