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
  saveProxySettings: (settings: ProxySettings) =>
    invoke<DashboardState>("save_proxy_settings", { settings }),
  testProxy: (service: string, config: ServiceProxyConfig) =>
    invoke<ProxyTestResult>("test_proxy", { service, config }),
  saveKimiApiKey: (apiKey: string, backend: KimiCredentialBackend) =>
    invoke<DashboardState>("save_kimi_api_key", { apiKey, backend }),
  clearKimiApiKey: (backend: KimiCredentialBackend) =>
    invoke<DashboardState>("clear_kimi_api_key", { backend }),
  launchTool: (toolId: string, projectDir?: string | null) =>
    invoke<void>("launch_tool", { toolId, projectDir }),
  revealConfigDir: () => invoke<void>("reveal_config_dir"),
};
