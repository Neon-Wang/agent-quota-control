import { Network, Plug } from "lucide-react";
import { useState } from "react";
import { api } from "../api";
import { useTranslations } from "../i18n";
import type { Translator } from "../i18n/translate";
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
  const t = useTranslations("settings");
  const common = useTranslations("common");
  const dashboard = useTranslations("dashboard");
  const [settings, setSettings] = useState<ProxySettingsType>(state.config.proxy);
  const [message, setMessage] = useState<string | null>(null);

  async function save() {
    onChange(await api.saveProxySettings(settings));
  }

  async function test(service: "kimi" | "codex") {
    const result = await api.testProxy(service, settings[service]);
    const name = service === "kimi" ? common("kimi_code") : common("codex");
    setMessage(`${name}：${proxyDetailLabel(result, dashboard)}`);
  }

  function update(service: "kimi" | "codex", next: ServiceProxyConfig) {
    setSettings({ ...settings, [service]: next });
  }

  return (
    <section className="panel wide">
      <div className="panel-title">
        <Network size={15} strokeWidth={1.75} aria-hidden />
        {t("network_proxy")}
      </div>
      <p className="muted panel-description">{t("proxy_auto_hint")}</p>
      <div className="settings-grid two">
        <ServiceProxyEditor
          label={common("kimi_code")}
          value={settings.kimi}
          onChange={(next) => update("kimi", next)}
          onTest={() => void test("kimi")}
          t={t}
        />
        <ServiceProxyEditor
          label={common("codex")}
          value={settings.codex}
          onChange={(next) => update("codex", next)}
          onTest={() => void test("codex")}
          t={t}
        />
      </div>
      {message && <p className="notice">{message}</p>}
      <div className="panel-actions">
        <button
          className="primary"
          type="button"
          onClick={save}
          aria-label={t("save_proxy_settings")}
        >
          {t("apply_changes")}
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
  t,
}: {
  label: string;
  value: ServiceProxyConfig;
  onChange: (value: ServiceProxyConfig) => void;
  onTest: () => void;
  t: Translator<"settings">;
}) {
  const [pressedMode, setPressedMode] = useState<ProxyMode | null>(null);

  function setMode(mode: ProxyMode) {
    onChange({ ...value, mode });
  }

  return (
    <div className="proxy-editor">
      <h3 className="subhead">{label}</h3>
      <div
        className="segmented"
        role="group"
        aria-label={t("proxy_mode", { service: label })}
      >
        {(["auto", "on", "off"] as ProxyMode[]).map((mode) => (
          <button
            key={mode}
            type="button"
            className={
              pressedMode === mode || (pressedMode === null && value.mode === mode)
                ? "active"
                : ""
            }
            aria-pressed={value.mode === mode}
            onPointerDown={() => setPressedMode(mode)}
            onPointerLeave={() => {
              if (pressedMode === mode) setPressedMode(null);
            }}
            onPointerCancel={() => setPressedMode(null)}
            onContextMenu={() => setPressedMode(null)}
            onClick={() => {
              setMode(mode);
              setPressedMode(null);
            }}
          >
            {modeLabel(mode, t)}
          </button>
        ))}
      </div>
      <div className="proxy-url-field">
        <span className="proxy-url-label">{t("proxy_url")}</span>
        <div className="proxy-url-row">
          <input
            value={value.proxyUrl ?? ""}
            onChange={(event) =>
              onChange({ ...value, proxyUrl: event.currentTarget.value || null })
            }
            placeholder="http://127.0.0.1:7897"
            aria-label={t("proxy_url_for", { service: label })}
          />
          <button className="secondary compact" type="button" onClick={onTest}>
            <Plug size={13} strokeWidth={1.75} aria-hidden />
            {t("test_connection")}
          </button>
        </div>
      </div>
    </div>
  );
}

function modeLabel(mode: ProxyMode, t: Translator<"settings">): string {
  if (mode === "auto") return t("mode_auto");
  if (mode === "on") return t("mode_on");
  return t("mode_off");
}
