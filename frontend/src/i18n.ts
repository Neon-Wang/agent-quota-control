export type Locale = "zh-CN";

const zhCN = {
  appName: "Agent 配额控制台",
  appSubtitle: "用量与启动器",
  dashboard: "概览",
  tools: "工具",
  monitoring: "监控",
  settings: "设置",
  updated: "更新于",
  notRefreshed: "尚未刷新",
  refresh: "刷新",
  loading: "正在加载控制台",
  proxyStatus: "代理状态",
  chooseProjectFolder: "选择项目文件夹",
  kimiCode: "Kimi Code",
  codex: "Codex",
} as const;

export const locale: Locale = "zh-CN";
export const t = zhCN;
