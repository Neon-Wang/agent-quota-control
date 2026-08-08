import { invoke } from "@tauri-apps/api/core";
import { tryGetFakeDashboard } from "./debug/fakeDashboard";
import type {
  DashboardState,
  KimiCredentialBackend,
  ProxySettings,
  ProxyTestResult,
  ServiceProxyConfig,
  StatusBarDisplayConfig,
} from "./types";

async function dashboardOrInvoke(
  command: string,
  args?: Record<string, unknown>,
): Promise<DashboardState> {
  const fake = await tryGetFakeDashboard();
  if (fake) return fake;
  return args === undefined
    ? invoke<DashboardState>(command)
    : invoke<DashboardState>(command, args);
}

export const api = {
  getDashboardState: () => dashboardOrInvoke("get_dashboard_state"),
  refreshUsage: () => dashboardOrInvoke("refresh_usage"),
  setSelectedServices: (serviceIds: string[]) =>
    dashboardOrInvoke("set_selected_services", { serviceIds }),
  setStatusBarServices: (serviceIds: string[]) =>
    dashboardOrInvoke("set_status_bar_services", { serviceIds }),
  setStatusBarDisplay: (display: StatusBarDisplayConfig) =>
    dashboardOrInvoke("set_status_bar_display", { display }),
  saveProxySettings: (settings: ProxySettings) =>
    dashboardOrInvoke("save_proxy_settings", { settings }),
  testProxy: (service: string, config: ServiceProxyConfig) =>
    invoke<ProxyTestResult>("test_proxy", { service, config }),
  saveKimiApiKey: (apiKey: string, backend: KimiCredentialBackend) =>
    dashboardOrInvoke("save_kimi_api_key", { apiKey, backend }),
  clearKimiApiKey: (backend: KimiCredentialBackend) =>
    dashboardOrInvoke("clear_kimi_api_key", { backend }),
  addKimiAccount: (
    displayName: string,
    apiKey: string,
    backend: KimiCredentialBackend,
  ) =>
    dashboardOrInvoke("add_kimi_account", {
      displayName,
      apiKey,
      backend,
    }),
  importCodexAccount: (displayName: string) =>
    dashboardOrInvoke("import_codex_account", { displayName }),
  renameAccount: (accountId: string, displayName: string) =>
    dashboardOrInvoke("rename_account", { accountId, displayName }),
  removeAccount: (accountId: string) =>
    dashboardOrInvoke("remove_account", { accountId }),
  revealConfigDir: () => invoke<void>("reveal_config_dir"),
};
