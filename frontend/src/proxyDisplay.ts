import type { ProxyTestResult } from "./types";

export function proxyBadgeLabel(serviceName: string, proxy: ProxyTestResult): string {
  return `${serviceName} ${proxyStatusLabel(proxy)}`;
}

export function proxyStatusLabel(proxy: ProxyTestResult): string {
  if (proxy.status === "proxy") {
    return "代理已连接";
  }
  if (proxy.status === "direct") {
    return "当前直连";
  }
  return "代理未连通";
}

export function proxyDetailLabel(proxy: ProxyTestResult): string {
  if (proxy.status === "proxy" && proxy.proxyUrl) {
    return `代理已连接：${proxy.proxyUrl}`;
  }
  if (proxy.status === "direct") {
    return "未检测到可用代理，当前走直连";
  }
  return proxy.message || "代理未连通，请检查地址和本地端口";
}
