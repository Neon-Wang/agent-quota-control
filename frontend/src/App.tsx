import { useEffect, useLayoutEffect, useRef, useState } from "react";
import type { PointerEvent } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  Activity,
  BarChart3,
  Folder,
  Loader2,
  RefreshCw,
  Settings,
} from "lucide-react";
import { api } from "./api";
import { AccountSettings } from "./components/AccountSettings";
import { AppearanceSettings } from "./components/AppearanceSettings";
import { LanguageSettings } from "./components/LanguageSettings";
import { MonitoringSettings } from "./components/MonitoringSettings";
import { ProxySettings } from "./components/ProxySettings";
import { QuotaCard } from "./components/QuotaCard";
import appMark from "./assets/app-mark.png";
import codexIcon from "./assets/codex.png";
import kimiIcon from "./assets/kimi.png";
import { useTranslations } from "./i18n";
import type { Translator } from "./i18n/translate";
import type { DashboardState } from "./types";

type View = "dashboard" | "monitoring" | "settings";

export function App() {
  const t = useTranslations("common");
  const settingsT = useTranslations("settings");
  const [view, setView] = useState<View>("dashboard");
  const [pressedView, setPressedView] = useState<View | null>(null);
  const [state, setState] = useState<DashboardState | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [windowFocused, setWindowFocused] = useState(true);
  const [contentScrolled, setContentScrolled] = useState(false);
  const contentScrollRef = useRef<HTMLDivElement>(null);
  const scrollEdgeRef = useRef<HTMLDivElement>(null);

  const navItems: Array<{ id: View; label: string; icon: typeof BarChart3 }> = [
    { id: "dashboard", label: t("dashboard"), icon: BarChart3 },
    { id: "monitoring", label: t("monitoring"), icon: Activity },
    { id: "settings", label: t("settings"), icon: Settings },
  ];

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

  useEffect(() => {
    const appWindow = getCurrentWindow();
    let disposed = false;
    const unsubs: Array<() => void> = [];

    const setFocused = (focused: boolean) => {
      if (!disposed) setWindowFocused(focused);
    };

    void appWindow
      .isFocused()
      .then(setFocused)
      .catch(() => setFocused(document.hasFocus()));

    void appWindow
      .onFocusChanged(({ payload: focused }) => setFocused(focused))
      .then((dispose) => {
        if (disposed) {
          dispose();
          return;
        }
        unsubs.push(dispose);
      })
      .catch(() => undefined);

    const handleFocus = () => setFocused(true);
    const handleBlur = () => setFocused(false);
    window.addEventListener("focus", handleFocus);
    window.addEventListener("blur", handleBlur);
    unsubs.push(() => {
      window.removeEventListener("focus", handleFocus);
      window.removeEventListener("blur", handleBlur);
    });

    return () => {
      disposed = true;
      for (const unsub of unsubs) unsub();
    };
  }, []);

  useLayoutEffect(() => {
    const edge = scrollEdgeRef.current;
    if (!edge) return;
    const previousTransform = edge.style.transform;
    edge.style.transform = "translateZ(0)";
    void edge.offsetHeight;
    edge.style.transform = previousTransform;
  }, [windowFocused]);

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
        <Loader2 size={18} strokeWidth={1.75} aria-hidden className="spin" />
        {t("loading")}
      </main>
    );
  }

  return (
    <main className={`app-shell ${windowFocused ? "window-active" : "window-inactive"}`}>
      <aside className="sidebar">
        <div className="brand" data-tauri-drag-region onPointerDown={beginWindowDrag}>
          <img className="brand-mark" src={appMark} alt="" draggable={false} />
          <div className="brand-copy">
            <h1>{t("app_name")}</h1>
            <p>{t("app_subtitle")}</p>
          </div>
        </div>
        <nav aria-label={t("main_nav")} data-tauri-no-drag>
          {navItems.map((item) => {
            const Icon = item.icon;
            return (
              <button
                key={item.id}
                className={
                  pressedView === item.id ||
                  (pressedView === null && view === item.id)
                    ? "nav-item active"
                    : "nav-item"
                }
                aria-current={view === item.id ? "page" : undefined}
                onPointerDown={() => setPressedView(item.id)}
                onPointerLeave={() => {
                  if (pressedView === item.id) setPressedView(null);
                }}
                onPointerCancel={() => setPressedView(null)}
                onContextMenu={() => setPressedView(null)}
                onClick={() => {
                  setView(item.id);
                  setPressedView(null);
                  setContentScrolled(false);
                  if (contentScrollRef.current) {
                    contentScrollRef.current.scrollTop = 0;
                  }
                }}
                type="button"
                data-tauri-no-drag
              >
                <Icon size={15} strokeWidth={1.75} aria-hidden />
                {item.label}
              </button>
            );
          })}
        </nav>
      </aside>

      <section className="content">
        <header
          className="topbar"
          data-tauri-drag-region
          onPointerDown={beginWindowDrag}
        >
          <div className="topbar-actions" data-tauri-no-drag>
            {state && (
              <div className="proxy-pills" aria-label={t("proxy_status")}>
                <span
                  className="topbar-status"
                  aria-label={`${t("kimi")}：${serviceHealth(state.cards, "kimi", t).label}`}
                >
                  <span
                    className={`status-dot ${serviceHealth(state.cards, "kimi", t).tone}`}
                    aria-hidden
                  />
                  {t("kimi")}
                </span>
                <span
                  className="topbar-status"
                  aria-label={`${t("codex")}：${serviceHealth(state.cards, "codex", t).label}`}
                >
                  <span
                    className={`status-dot ${serviceHealth(state.cards, "codex", t).tone}`}
                    aria-hidden
                  />
                  {t("codex")}
                </span>
              </div>
            )}
            <button
              className="topbar-control prominent"
              type="button"
              onClick={refreshUsage}
              data-tauri-no-drag
            >
              <RefreshCw size={14} strokeWidth={1.75} aria-hidden />
              {t("refresh")}
            </button>
          </div>
          <div className="topbar-title">
            <h2>{navItems.find((item) => item.id === view)?.label}</h2>
          </div>
        </header>

        <div
          ref={contentScrollRef}
          className={contentScrolled ? "content-scroll scrolled" : "content-scroll"}
          onScroll={(event) =>
            setContentScrolled(event.currentTarget.scrollTop > 2)
          }
        >
          <div ref={scrollEdgeRef} className="scroll-edge-effect" aria-hidden>
            <span
              key={`soft-${windowFocused ? "on" : "off"}`}
              className="edge-blur edge-blur-soft"
            />
            <span
              key={`medium-${windowFocused ? "on" : "off"}`}
              className="edge-blur edge-blur-medium"
            />
            <span
              key={`near-${windowFocused ? "on" : "off"}`}
              className="edge-blur edge-blur-near"
            />
          </div>
          <div className="content-scroll-body">
            {error && <div className="error-box">{error}</div>}

            {state && view === "dashboard" && (
              <div className="dashboard-grid">
                {state.cards.map((card) => (
                  <QuotaCard
                    key={card.accountId}
                    card={card}
                    iconSrc={card.service === "kimi" ? kimiIcon : codexIcon}
                  />
                ))}
                {state.cards.length === 0 && (
                  <div className="dashboard-empty">
                    <h3>{t("empty_accounts_title")}</h3>
                    <p>{t("empty_accounts_body")}</p>
                  </div>
                )}
              </div>
            )}

            {state && view === "monitoring" && (
              <div className="settings-grid">
                <MonitoringSettings state={state} onChange={setState} />
                <AccountSettings state={state} onChange={setState} />
              </div>
            )}

            {state && view === "settings" && (
              <div className="settings-grid">
                <ProxySettings state={state} onChange={setState} />
                <AppearanceSettings />
                <LanguageSettings />
                <section className="panel">
                  <div className="panel-title">
                    <Folder size={15} strokeWidth={1.75} aria-hidden />
                    {settingsT("config_dir")}
                  </div>
                  <p className="muted panel-description">
                    {settingsT("config_dir_hint")}
                  </p>
                  <button className="secondary" type="button" onClick={api.revealConfigDir}>
                    {settingsT("reveal_in_finder")}
                  </button>
                </section>
              </div>
            )}
          </div>
        </div>
      </section>
    </main>
  );
}

function serviceHealth(
  cards: DashboardState["cards"],
  service: DashboardState["cards"][number]["service"],
  t: Translator<"common">,
): {
  tone: "connected" | "direct" | "warning" | "problem";
  label: string;
} {
  const serviceCards = cards.filter((card) => card.service === service);
  if (serviceCards.length === 0) {
    return { tone: "direct", label: t("health_no_account") };
  }

  const healthyCards = serviceCards.filter(
    (card) => card.status === "fresh" || card.status === "stale",
  );
  if (healthyCards.length === serviceCards.length) {
    return { tone: "connected", label: t("health_ok") };
  }
  if (serviceCards.some((card) => card.status === "login_expired")) {
    return healthyCards.length > 0
      ? { tone: "warning", label: t("health_partial_login") }
      : { tone: "problem", label: t("health_needs_login") };
  }
  if (healthyCards.length > 0) {
    return { tone: "warning", label: t("health_partial_unavailable") };
  }
  if (serviceCards.some((card) => card.status === "update_failed")) {
    return { tone: "warning", label: t("health_refresh_failed") };
  }
  return { tone: "direct", label: t("health_waiting") };
}
