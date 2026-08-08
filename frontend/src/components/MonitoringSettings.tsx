import { Activity, PanelTop } from "lucide-react";
import { api } from "../api";
import { useTranslations } from "../i18n";
import type { DashboardState, StatusBarDisplayConfig } from "../types";
import { ToggleSwitch } from "./ToggleSwitch";

interface MonitoringSettingsProps {
  state: DashboardState;
  onChange: (state: DashboardState) => void;
}

const services = [
  { id: "kimi", nameKey: "kimi_code" as const },
  { id: "codex", nameKey: "codex" as const },
] as const;

const displayOptions: ReadonlyArray<{
  key: keyof StatusBarDisplayConfig;
  labelKey: "show_icon" | "show_percentage" | "show_state_text";
}> = [
  { key: "showIcon", labelKey: "show_icon" },
  { key: "showPercentage", labelKey: "show_percentage" },
  { key: "showStateText", labelKey: "show_state_text" },
];

export function MonitoringSettings({ state, onChange }: MonitoringSettingsProps) {
  const t = useTranslations("monitoring");
  const common = useTranslations("common");

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
          {t("services_title")}
        </div>
        {services.map((service) => {
          const name = common(service.nameKey);
          const monitored = state.config.selectedServices.includes(service.id);
          return (
            <div className="switch-row" key={service.id}>
              <span>
                <strong>{name}</strong>
              </span>
              <div className="service-switches">
                <label>
                  <span>{t("monitor")}</span>
                  <ToggleSwitch
                    aria-label={t("monitor_service", { service: name })}
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
                  <span>{t("menu_bar")}</span>
                  <ToggleSwitch
                    aria-label={t("show_in_menu_bar", { service: name })}
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
          {t("menu_bar_style")}
        </div>
        {displayOptions.map(({ key, labelKey }) => {
          const label = t(labelKey);
          return (
            <div className="switch-row" key={key}>
              <span>
                <strong>{label}</strong>
              </span>
              <ToggleSwitch
                aria-label={t("menu_bar_toggle", { label })}
                checked={state.config.statusBarDisplay[key]}
                onChange={(enabled) => void updateDisplay(key, enabled)}
              />
            </div>
          );
        })}
      </section>
    </>
  );
}
