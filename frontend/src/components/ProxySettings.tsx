import { Network, Plug } from "lucide-react";
import { useState } from "react";
import { api } from "../api";
import { proxyDetailLabel } from "../proxyDisplay";
import type {
  DashboardState,
  ProxyMode,
  ProxySettings as ProxySettingsType,
  ServiceProxyConfig,
} from "../types";

interface ProxySettingsProps {
  state: DashboardState;
  onChange: (state: DashboardState) => void;
}

export function ProxySettings({ state, onChange }: ProxySettingsProps) {
  const [settings, setSettings] = useState<ProxySettingsType>(state.config.proxy);
  const [message, setMessage] = useState<string | null>(null);

  async function save() {
    onChange(await api.saveProxySettings(settings));
  }

  async function test(service: "kimi" | "codex") {
    const result = await api.testProxy(service, settings[service]);
    setMessage(`${service === "kimi" ? "Kimi Code" : "Codex"}：${proxyDetailLabel(result)}`);
  }

  function update(service: "kimi" | "codex", next: ServiceProxyConfig) {
    setSettings({ ...settings, [service]: next });
  }

  return (
    <section className="panel wide">
      <div className="panel-title">
        <Network size={15} strokeWidth={1.75} aria-hidden />
        网络代理
      </div>
      <p className="muted panel-description">
        为每项服务单独选择连接方式。自动模式会优先尝试填写的地址，再检查常用本地端口。
      </p>
      <div className="settings-grid two">
        <ServiceProxyEditor
          label="Kimi Code"
          value={settings.kimi}
          onChange={(next) => update("kimi", next)}
          onTest={() => void test("kimi")}
        />
        <ServiceProxyEditor
          label="Codex"
          value={settings.codex}
          onChange={(next) => update("codex", next)}
          onTest={() => void test("codex")}
        />
      </div>
      {message && <p className="notice">{message}</p>}
      <div className="panel-actions">
        <button
          className="primary"
          type="button"
          onClick={save}
          aria-label="保存代理设置"
        >
          应用更改
        </button>
      </div>
    </section>
  );
}

function ServiceProxyEditor({
  label,
  value,
  onChange,
  onTest,
}: {
  label: string;
  value: ServiceProxyConfig;
  onChange: (value: ServiceProxyConfig) => void;
  onTest: () => void;
}) {
  function setMode(mode: ProxyMode) {
    onChange({ ...value, mode });
  }

  return (
    <div className="proxy-editor">
      <h3 className="subhead">{label}</h3>
      <div className="segmented">
        {(["auto", "on", "off"] as ProxyMode[]).map((mode) => (
          <button
            key={mode}
            type="button"
            className={value.mode === mode ? "active" : ""}
            onClick={() => setMode(mode)}
          >
            {modeLabel(mode)}
          </button>
        ))}
      </div>
      <label className="field">
        代理 URL
        <input
          value={value.proxyUrl ?? ""}
          onChange={(event) =>
            onChange({ ...value, proxyUrl: event.currentTarget.value || null })
          }
          placeholder="http://127.0.0.1:7897"
        />
      </label>
      <button className="secondary compact" type="button" onClick={onTest}>
        <Plug size={13} strokeWidth={1.75} aria-hidden />
        测试连接
      </button>
    </div>
  );
}

function modeLabel(mode: ProxyMode): string {
  if (mode === "auto") return "自动";
  if (mode === "on") return "开启";
  return "关闭";
}
