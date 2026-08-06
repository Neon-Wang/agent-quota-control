export type Locale = "zh-CN";

const zhCN = {
  appName: "Agent 配额控制台",
  appSubtitle: "用量与状态栏",
  dashboard: "概览",
  monitoring: "监控",
  settings: "设置",
  refresh: "刷新",
  loading: "正在加载控制台",
  proxyStatus: "代理状态",
  kimiCode: "Kimi Code",
  codex: "Codex",
} as const;

export const locale: Locale = "zh-CN";
export const t = zhCN;
