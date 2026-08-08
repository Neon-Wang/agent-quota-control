import type { Translator } from "./i18n/translate";
import type { ProxyTestResult } from "./types";

type DashboardT = Translator<"dashboard">;

export function proxyBadgeLabel(
  serviceName: string,
  proxy: ProxyTestResult,
  t: DashboardT,
): string {
  return `${serviceName} ${proxyStatusLabel(proxy, t)}`;
}

export function proxyStatusLabel(proxy: ProxyTestResult, t: DashboardT): string {
  if (proxy.status === "proxy") {
    return t("proxy_connected");
  }
  if (proxy.status === "direct") {
    return t("proxy_direct");
  }
  return t("proxy_down");
}

export function proxyDetailLabel(proxy: ProxyTestResult, t: DashboardT): string {
  if (proxy.status === "proxy" && proxy.proxyUrl) {
    return t("proxy_connected_url", { url: proxy.proxyUrl });
  }
  if (proxy.status === "direct") {
    return t("proxy_direct_detail");
  }
  return proxy.message || t("proxy_down_detail");
}
