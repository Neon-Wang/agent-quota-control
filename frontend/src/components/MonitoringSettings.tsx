import { Activity, PanelTop } from "lucide-react";
import { api } from "../api";
import type { DashboardState, StatusBarDisplayConfig } from "../types";
import { ToggleSwitch } from "./ToggleSwitch";

interface MonitoringSettingsProps {
  state: DashboardState;
  onChange: (state: DashboardState) => void;
}

const services = [
  {
    id: "kimi",
    name: "Kimi Code",
    description: "获取 Kimi Code 的用量和频限状态。",
  },
  {
    id: "codex",
    name: "Codex",
    description: "通过 Codex 登录信息获取用量状态。",
  },
] as const;

const displayOptions: ReadonlyArray<{
  key: keyof StatusBarDisplayConfig;
  label: string;
}> = [
  { key: "showIcon", label: "服务图标" },
  { key: "showPercentage", label: "百分比" },
  { key: "showStateText", label: "状态文字" },
];

export function MonitoringSettings({ state, onChange }: MonitoringSettingsProps) {
  async function updateServiceList(
    service: string,
    enabled: boolean,
    currentServices: string[],
    save: (serviceIds: string[]) => Promise<DashboardState>,
  ) {
    const next = new Set(currentServices);
    if (enabled) next.add(service);
    else next.delete(service);
    onChange(await save([...next].sort()));
  }

  async function updateDisplay(
    key: keyof StatusBarDisplayConfig,
    enabled: boolean,
  ) {
    onChange(
      await api.setStatusBarDisplay({
        ...state.config.statusBarDisplay,
        [key]: enabled,
      }),
    );
  }

  return (
    <>
      <section className="panel">
        <div className="panel-title">
          <Activity size={15} strokeWidth={1.75} aria-hidden />
          监控服务
        </div>
        {services.map((service) => {
          const monitored = state.config.selectedServices.includes(service.id);
          return (
            <div className="switch-row" key={service.id}>
              <span>
                <strong>{service.name}</strong>
                <small>{service.description}</small>
              </span>
              <div className="service-switches">
                <label>
                  <span>监控</span>
                  <ToggleSwitch
                    aria-label={`监控 ${service.name}`}
                    checked={monitored}
                    onChange={(enabled) =>
                      void updateServiceList(
                        service.id,
                        enabled,
                        state.config.selectedServices,
                        api.setSelectedServices,
                      )
                    }
                  />
                </label>
                <label>
                  <span>状态栏</span>
                  <ToggleSwitch
                    aria-label={`在状态栏显示 ${service.name}`}
                    disabled={!monitored}
                    checked={state.config.statusBarServices.includes(service.id)}
                    onChange={(enabled) =>
                      void updateServiceList(
                        service.id,
                        enabled,
                        state.config.statusBarServices,
                        api.setStatusBarServices,
                      )
                    }
                  />
                </label>
              </div>
            </div>
          );
        })}
      </section>

      <section className="panel">
        <div className="panel-title">
          <PanelTop size={15} strokeWidth={1.75} aria-hidden />
          状态栏样式
        </div>
        <p className="muted">三个元素可以独立开关并自由组合。</p>
        {displayOptions.map(({ key, label }) => (
          <div className="switch-row" key={key}>
            <span>
              <strong>{label}</strong>
            </span>
            <ToggleSwitch
              aria-label={`状态栏显示${label}`}
              checked={state.config.statusBarDisplay[key]}
              onChange={(enabled) => void updateDisplay(key, enabled)}
            />
          </div>
        ))}
      </section>
    </>
  );
}
