import { useEffect, useMemo, useState } from "react";
import type { PointerEvent } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  Activity,
  BarChart3,
  KeyRound,
  Loader2,
  RefreshCw,
  Settings,
  Terminal,
  Wrench,
} from "lucide-react";
import { api } from "./api";
import { CredentialSettings } from "./components/CredentialSettings";
import { LaunchPanel } from "./components/LaunchPanel";
import { MonitoringSettings } from "./components/MonitoringSettings";
import { ProxySettings } from "./components/ProxySettings";
import { QuotaCard } from "./components/QuotaCard";
import { ToolList } from "./components/ToolList";
import codexIcon from "./assets/codex.png";
import kimiIcon from "./assets/kimi.png";
import { t } from "./i18n";
import { proxyBadgeLabel } from "./proxyDisplay";
import type { DashboardState, ToolInfo } from "./types";

type View = "dashboard" | "tools" | "monitoring" | "settings";

const navItems: Array<{ id: View; label: string; icon: typeof BarChart3 }> = [
  { id: "dashboard", label: t.dashboard, icon: BarChart3 },
  { id: "tools", label: t.tools, icon: Wrench },
  { id: "monitoring", label: t.monitoring, icon: Activity },
  { id: "settings", label: t.settings, icon: Settings },
];

export function App() {
  const [view, setView] = useState<View>("dashboard");
  const [state, setState] = useState<DashboardState | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const selectedTools = useMemo(() => {
    if (!state) return [];
    return state.tools.filter((tool) =>
      state.config.selectedTools.includes(tool.id),
    );
  }, [state]);

  const availableTools = useMemo(() => {
    if (!state) return [];
    return state.tools.filter(
      (tool) => !state.config.selectedTools.includes(tool.id),
    );
  }, [state]);

  const lastUpdated = useMemo(() => {
    const timestamps = [
      state?.kimiQuota?.queriedAt,
      state?.codexQuota?.queriedAt,
    ].filter((value): value is number => typeof value === "number");
    if (timestamps.length === 0) return t.notRefreshed;
    return new Date(Math.max(...timestamps)).toLocaleTimeString();
  }, [state]);

  useEffect(() => {
    void loadState();
    const unlisten = listen<DashboardState>("dashboard://updated", (event) => {
      setState(event.payload);
      setLoading(false);
    });
    const interval = window.setInterval(() => {
      void api.getDashboardState().then(setState).catch(() => undefined);
    }, 30_000);
    return () => {
      window.clearInterval(interval);
      void unlisten.then((dispose) => dispose());
    };
  }, []);

  async function loadState() {
    setLoading(true);
    setError(null);
    try {
      setState(await api.getDashboardState());
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }

  async function refreshUsage() {
    setError(null);
    try {
      setState(await api.refreshUsage());
    } catch (err) {
      setError(String(err));
    }
  }

  async function updateTools(toolIds: string[]) {
    setState(await api.setSelectedTools(toolIds));
  }

  async function launchTool(tool: ToolInfo) {
    let projectDir: string | null = null;
    if (tool.toolType === "cli") {
      const selected = await open({
        directory: true,
        multiple: false,
        title: `${t.chooseProjectFolder}: ${tool.name}`,
      });
      if (typeof selected !== "string") return;
      projectDir = selected;
    }
    await api.launchTool(tool.id, projectDir);
  }

  function beginWindowDrag(event: PointerEvent<HTMLElement>) {
    if (event.button !== 0) return;
    const target = event.target as HTMLElement;
    if (target.closest("[data-tauri-no-drag], button, input, select, textarea, a")) {
      return;
    }
    void getCurrentWindow().startDragging();
  }

  if (loading && !state) {
    return (
      <main className="app-loading" aria-busy="true">
        <Loader2 size={18} aria-hidden className="spin" />
        {t.loading}
      </main>
    );
  }

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand" data-tauri-drag-region onPointerDown={beginWindowDrag}>
          <div>
            <h1>{t.appName}</h1>
            <p>{t.appSubtitle}</p>
          </div>
        </div>
        <nav aria-label="主导航" data-tauri-no-drag>
          {navItems.map((item) => {
            const Icon = item.icon;
            return (
              <button
                key={item.id}
                className={view === item.id ? "nav-item active" : "nav-item"}
                onClick={() => setView(item.id)}
                type="button"
                data-tauri-no-drag
              >
                <Icon size={16} aria-hidden />
                {item.label}
              </button>
            );
          })}
        </nav>
      </aside>

      <section className="content">
        <header className="topbar" data-tauri-drag-region onPointerDown={beginWindowDrag}>
          <div>
            <h2>{navItems.find((item) => item.id === view)?.label}</h2>
            <p className="eyebrow">{t.updated} {lastUpdated}</p>
          </div>
          <div className="topbar-actions" data-tauri-no-drag>
            {state && (
              <div className="proxy-pills" aria-label={t.proxyStatus}>
                <span>{proxyBadgeLabel("Kimi", state.proxyStatus.kimi)}</span>
                <span>{proxyBadgeLabel("Codex", state.proxyStatus.codex)}</span>
              </div>
            )}
            <button className="primary" type="button" onClick={refreshUsage} data-tauri-no-drag>
              <RefreshCw size={15} aria-hidden />
              {t.refresh}
            </button>
          </div>
        </header>

        {error && <div className="error-box">{error}</div>}

        {state && view === "dashboard" && (
          <div className="dashboard-grid">
            <QuotaCard
              title="Kimi Code"
              iconSrc={kimiIcon}
              quota={state.kimiQuota}
              estimates={state.kimiEstimates}
              proxy={state.proxyStatus.kimi}
            />
            <QuotaCard
              title="Codex"
              iconSrc={codexIcon}
              quota={state.codexQuota}
              estimates={state.codexEstimates}
              proxy={state.proxyStatus.codex}
            />
          </div>
        )}

        {state && view === "tools" && (
          <div className="stack">
            <LaunchPanel tools={selectedTools} onLaunch={launchTool} />
            <ToolList
              selectedTools={selectedTools}
              availableTools={availableTools}
              selectedToolIds={state.config.selectedTools}
              onChange={updateTools}
              onLaunch={launchTool}
            />
          </div>
        )}

        {state && view === "monitoring" && (
          <div className="settings-grid">
            <MonitoringSettings state={state} onChange={setState} />
            <CredentialSettings state={state} onChange={setState} />
            <section className="panel">
              <div className="panel-title">
                <Terminal size={16} aria-hidden />
                Codex 登录状态
              </div>
              <p className="muted">
                Codex 凭据会从 Codex CLI 的 Keychain 或
                ~/.codex/auth.json 读取。如果无法获取用量，请先运行
                <code>codex login</code>。
              </p>
            </section>
          </div>
        )}

        {state && view === "settings" && (
          <div className="settings-grid">
            <ProxySettings state={state} onChange={setState} />
            <section className="panel">
              <div className="panel-title">
                <KeyRound size={16} aria-hidden />
                配置目录
              </div>
              <p className="muted">
                打开本地配置目录，用于备份或检查代理、工具选择和用量事件。
              </p>
              <button className="secondary" type="button" onClick={api.revealConfigDir}>
                打开配置目录
              </button>
            </section>
          </div>
        )}
      </section>
    </main>
  );
}
