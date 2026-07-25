import { Activity } from "lucide-react";
import { api } from "../api";
import type { DashboardState } from "../types";

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

  return (
    <section className="panel">
      <div className="panel-title">
        <Activity size={16} aria-hidden />
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
                <input
                  aria-label={`监控 ${service.name}`}
                  type="checkbox"
                  checked={monitored}
                  onChange={(event) =>
                    void updateServiceList(
                      service.id,
                      event.currentTarget.checked,
                      state.config.selectedServices,
                      api.setSelectedServices,
                    )
                  }
                />
              </label>
              <label>
                <span>状态栏</span>
                <input
                  aria-label={`在状态栏显示 ${service.name}`}
                  type="checkbox"
                  disabled={!monitored}
                  checked={state.config.statusBarServices.includes(service.id)}
                  onChange={(event) =>
                    void updateServiceList(
                      service.id,
                      event.currentTarget.checked,
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
  );
}
