import { Activity } from "lucide-react";
import { api } from "../api";
import type { DashboardState } from "../types";

interface MonitoringSettingsProps {
  state: DashboardState;
  onChange: (state: DashboardState) => void;
}

export function MonitoringSettings({ state, onChange }: MonitoringSettingsProps) {
  async function toggle(service: string, enabled: boolean) {
    const current = new Set(state.config.selectedServices);
    if (enabled) current.add(service);
    else current.delete(service);
    onChange(await api.setSelectedServices([...current].sort()));
  }

  return (
    <section className="panel">
      <div className="panel-title">
        <Activity size={16} aria-hidden />
        监控服务
      </div>
      <label className="switch-row">
        <span>
          <strong>Kimi Code</strong>
          <small>获取 Kimi Code 的用量和频限状态。</small>
        </span>
        <input
          type="checkbox"
          checked={state.config.selectedServices.includes("kimi")}
          onChange={(event) => void toggle("kimi", event.currentTarget.checked)}
        />
      </label>
      <label className="switch-row">
        <span>
          <strong>Codex</strong>
          <small>通过 Codex 登录信息获取用量状态。</small>
        </span>
        <input
          type="checkbox"
          checked={state.config.selectedServices.includes("codex")}
          onChange={(event) => void toggle("codex", event.currentTarget.checked)}
        />
      </label>
    </section>
  );
}
