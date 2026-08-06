import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "../App";
import type { DashboardState } from "../types";

const invokeMock = vi.fn();
const fixedNow = new Date("2026-06-04T08:15:00+08:00").getTime();

const windowFocusMock = vi.hoisted(() => {
  const listeners: Array<(event: { payload: boolean }) => void> = [];
  let focused = true;
  return {
    listeners,
    emit(nextFocused: boolean) {
      focused = nextFocused;
      for (const listener of [...listeners]) {
        listener({ payload: nextFocused });
      }
    },
    reset() {
      focused = true;
      listeners.length = 0;
    },
    api: {
      startDragging: vi.fn(),
      isFocused: vi.fn(() => Promise.resolve(focused)),
      onFocusChanged: vi.fn((handler: (event: { payload: boolean }) => void) => {
        listeners.push(handler);
        return Promise.resolve(() => {
          const index = listeners.indexOf(handler);
          if (index >= 0) listeners.splice(index, 1);
        });
      }),
    },
  };
});

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (command: string, args?: unknown) => invokeMock(command, args),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => undefined)),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => windowFocusMock.api,
}));

const dashboardState: DashboardState = {
  config: {
    version: 5,
    accounts: [
      {
        id: "legacy-kimi",
        service: "kimi",
        displayName: "Kimi 工作账号",
        providerIdentityHint: null,
        credentialRef: "legacy_kimi",
        enabled: true,
        createdAt: 0,
      },
      {
        id: "legacy-codex",
        service: "codex",
        displayName: "Codex 个人账号",
        providerIdentityHint: "acct...work",
        credentialRef: "live_codex",
        enabled: true,
        createdAt: 0,
      },
    ],
    selectedServices: ["kimi", "codex"],
    statusBarServices: ["kimi", "codex"],
    statusBarDisplay: {
      showIcon: true,
      showPercentage: true,
      showStateText: true,
    },
    firstRunCompleted: true,
    credentials: { kimiBackend: "keychain" },
    proxy: {
      kimi: {
        mode: "auto",
        proxyUrl: null,
        autoPorts: [7897, 7890],
        timeoutMs: 250,
      },
      codex: {
        mode: "auto",
        proxyUrl: null,
        autoPorts: [7897, 7890],
        timeoutMs: 250,
      },
    },
  },
  cards: [
    {
      accountId: "legacy-kimi",
      service: "kimi",
      serviceDisplayName: "Kimi Code",
      accountDisplayName: "Kimi 工作账号",
      status: "fresh",
      tiers: [
        { name: "five_hour", utilization: 12, resetsAt: "2026-06-04T10:30:00+08:00" },
        { name: "weekly_limit", utilization: 40, resetsAt: "2026-06-07T18:45:00+08:00" },
      ],
      weeklyEstimate: {
        state: "enough",
        projectedUtilization: 72,
        resetInSecs: 297_000,
        lastsForSecs: 540_000,
        slopePctPerHour: 0.4,
        trendWindowHours: 24,
        observedSpanSecs: 86_400,
        windowStartSecs: Math.floor(fixedNow / 1_000) - 4 * 86_400,
        windowEndSecs: Math.floor(fixedNow / 1_000) + 297_000,
        observedPoints: [
          {
            observedAtSecs: Math.floor(fixedNow / 1_000) - 86_400,
            utilization: 28,
          },
          {
            observedAtSecs: Math.floor(fixedNow / 1_000),
            utilization: 40,
          },
        ],
        projectedPoints: [
          {
            observedAtSecs: Math.floor(fixedNow / 1_000),
            utilization: 40,
          },
          {
            observedAtSecs: Math.floor(fixedNow / 1_000) + 297_000,
            utilization: 72,
          },
        ],
      },
      proxy: { status: "direct", proxyUrl: null, message: "Direct" },
      queriedAt: Date.now(),
      lastSuccessfulAt: Date.now(),
      errorMessage: null,
    },
    {
      accountId: "legacy-codex",
      service: "codex",
      serviceDisplayName: "Codex",
      accountDisplayName: "Codex 个人账号",
      status: "fresh",
      tiers: [
        { name: "seven_day", utilization: 100, resetsAt: null },
      ],
      weeklyEstimate: {
        state: "not_enough",
        projectedUtilization: 188,
        lastsForSecs: 93_600,
      },
      proxy: {
        status: "proxy",
        proxyUrl: "http://127.0.0.1:7897",
        message: "Proxy",
      },
      queriedAt: Date.now(),
      lastSuccessfulAt: Date.now(),
      errorMessage: null,
    },
  ],
  kimiQuota: {
    service: "kimi",
    displayName: "Kimi Code",
    success: true,
    tiers: [
      { name: "five_hour", utilization: 12, resetsAt: "2026-06-04T10:30:00+08:00" },
      { name: "weekly_limit", utilization: 40, resetsAt: "2026-06-07T18:45:00+08:00" },
    ],
    error: null,
    queriedAt: Date.now(),
    credentialValid: true,
  },
  codexQuota: {
    service: "codex",
    displayName: "Codex",
    success: true,
    tiers: [
      { name: "seven_day", utilization: 100, resetsAt: null },
    ],
    error: null,
    queriedAt: Date.now(),
    credentialValid: true,
  },
  kimiEstimates: [
    {
      tier: "weekly_limit",
      estimate: { state: "enough", projectedUtilization: 72 },
    },
  ],
  codexEstimates: [
    {
      tier: "seven_day",
      estimate: {
        state: "not_enough",
        projectedUtilization: 188,
        lastsForSecs: 93_600,
      },
    },
  ],
  proxyStatus: {
    kimi: { status: "direct", proxyUrl: null, message: "Direct" },
    codex: {
      status: "proxy",
      proxyUrl: "http://127.0.0.1:7897",
      message: "Proxy",
    },
  },
};

describe("App", () => {
  beforeEach(() => {
    vi.spyOn(Date, "now").mockReturnValue(fixedNow);
    windowFocusMock.reset();
    invokeMock.mockReset();
    invokeMock.mockImplementation((command: string) => {
      if (command === "get_dashboard_state") return Promise.resolve(dashboardState);
      if (command === "save_proxy_settings") return Promise.resolve(dashboardState);
      return Promise.resolve(dashboardState);
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("renders Kimi and Codex quota cards", async () => {
    render(<App />);

    const kimiCard = await screen.findByRole("region", { name: "Kimi 工作账号 配额" });
    const codexCard = screen.getByRole("region", { name: "Codex 个人账号 配额" });

    expect(within(kimiCard).getByText("够用")).toBeInTheDocument();
    expect(within(codexCard).getByText("不够")).toBeInTheDocument();
    expect(within(kimiCard).getByRole("heading", { name: "Kimi 工作账号" })).toBeInTheDocument();
    expect(within(kimiCard).getByText("Kimi Code")).toBeInTheDocument();
    expect(within(codexCard).getByRole("heading", { name: "Codex 个人账号" })).toBeInTheDocument();
    expect(within(codexCard).getByText("当前无 5 小时限制")).toBeInTheDocument();

    const kimiTierSlots = kimiCard.querySelectorAll(".tier-row, .tier-unavailable");
    const codexTierSlots = codexCard.querySelectorAll(".tier-row, .tier-unavailable");
    expect(kimiTierSlots).toHaveLength(2);
    expect(codexTierSlots).toHaveLength(2);
    expect(kimiTierSlots[0]).toHaveTextContent("7 天");
    expect(kimiTierSlots[1]).toHaveTextContent("5 小时");
    expect(codexTierSlots[0]).toHaveTextContent("7 天");
    expect(codexTierSlots[1]).toHaveTextContent("当前无 5 小时限制");

    expect(screen.getByText("（2 小时 15 分钟后重置）")).toBeInTheDocument();
    expect(screen.getByText(/06月07日 .* 重置/)).toBeInTheDocument();
    const proxyStatus = screen.getByLabelText("代理状态");
    expect(within(proxyStatus).getByText("Kimi")).toBeInTheDocument();
    expect(within(proxyStatus).getByText("Codex")).toBeInTheDocument();
    expect(within(proxyStatus).getByLabelText("Kimi：登录正常")).toBeInTheDocument();
    expect(within(proxyStatus).getByLabelText("Codex：登录正常")).toBeInTheDocument();
    expect(screen.getByText("本周内预计够用。")).toBeInTheDocument();
    expect(screen.getByText("预计将在 1 天 2 小时 后耗尽。")).toBeInTheDocument();
    expect(
      screen.getByRole("img", { name: /实际用量.*近期趋势预测/ }),
    ).toBeInTheDocument();
    expect(screen.getByText(/近 24 小时趋势：每小时增加 0.4%/)).toBeInTheDocument();
    expect(screen.queryByText("direct")).not.toBeInTheDocument();
    expect(screen.queryByText("unavailable")).not.toBeInTheDocument();
  });

  it("shows login failures in service status without duplicate warning icons", async () => {
    const loginExpiredState: DashboardState = {
      ...dashboardState,
      cards: dashboardState.cards.map((card) => ({
        ...card,
        status: "login_expired",
        errorMessage: "登录已失效",
      })),
    };
    invokeMock.mockImplementation((command: string) => {
      if (command === "get_dashboard_state") return Promise.resolve(loginExpiredState);
      return Promise.resolve(loginExpiredState);
    });

    render(<App />);

    expect(await screen.findByLabelText("Kimi：需要登录")).toBeInTheDocument();
    expect(screen.getByLabelText("Codex：需要登录")).toBeInTheDocument();
    const kimiCard = screen.getByRole("region", { name: "Kimi 工作账号 配额" });
    expect(kimiCard.querySelectorAll(".lucide-triangle-alert")).toHaveLength(1);
    expect(within(kimiCard).getByText("登录已失效，请重新登录后刷新。")).toBeInTheDocument();
  });

  it("does not expose the retired generic tools page", async () => {
    render(<App />);

    await screen.findByRole("button", { name: "概览" });
    expect(screen.queryByRole("button", { name: "工具" })).not.toBeInTheDocument();
  });

  it("highlights navigation on press and changes pages on release", async () => {
    render(<App />);

    const overview = await screen.findByRole("button", { name: "概览" });
    const settings = screen.getByRole("button", { name: "设置" });
    const kimiCard = screen.getByRole("region", { name: "Kimi 工作账号 配额" });

    fireEvent.pointerDown(settings, { button: 0 });
    expect(settings).toHaveClass("active");
    expect(overview).not.toHaveClass("active");
    expect(kimiCard).toBeInTheDocument();

    fireEvent.click(settings);
    expect(settings).toHaveAttribute("aria-current", "page");
    expect(screen.getByRole("button", { name: "保存代理设置" })).toBeInTheDocument();
  });

  it("activates the safe-area blur after content scrolls", async () => {
    const { container } = render(<App />);

    await screen.findByRole("button", { name: "概览" });
    const scroller = container.querySelector(".content-scroll") as HTMLDivElement;
    const edgeBlur = container.querySelector(".scroll-edge-effect");
    const controls = container.querySelector(".topbar-actions") as HTMLElement;

    expect(scroller).not.toHaveClass("scrolled");
    expect(scroller.contains(edgeBlur)).toBe(true);
    expect(within(controls).getByText("Kimi")).toBeInTheDocument();
    expect(within(controls).getByText("Codex")).toBeInTheDocument();
    expect(within(controls).getByRole("button", { name: "刷新" })).toBeInTheDocument();

    scroller.scrollTop = 24;
    fireEvent.scroll(scroller);
    expect(scroller).toHaveClass("scrolled");
  });

  it("remounts the safe-area blur layers when window focus changes", async () => {
    const { container } = render(<App />);
    await screen.findByRole("button", { name: "概览" });
    await waitFor(() => {
      expect(windowFocusMock.listeners.length).toBeGreaterThan(0);
    });
    // Let the initial isFocused() resolution settle before toggling focus.
    await waitFor(() => {
      expect(windowFocusMock.api.isFocused).toHaveBeenCalled();
    });
    await Promise.resolve();

    const before = container.querySelector(".edge-blur-soft");
    expect(before).toBeTruthy();

    act(() => {
      windowFocusMock.emit(false);
    });
    await waitFor(() => {
      expect(container.querySelector(".app-shell")).toHaveClass("window-inactive");
    });
    const afterBlur = container.querySelector(".edge-blur-soft");
    expect(afterBlur).toBeTruthy();
    expect(afterBlur).not.toBe(before);

    act(() => {
      windowFocusMock.emit(true);
    });
    await waitFor(() => {
      expect(container.querySelector(".app-shell")).toHaveClass("window-active");
    });
    const afterFocus = container.querySelector(".edge-blur-soft");
    expect(afterFocus).toBeTruthy();
    expect(afterFocus).not.toBe(afterBlur);
  });

  it("saves proxy settings from settings tab", async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.click(await screen.findByRole("button", { name: "设置" }));
    await user.click(screen.getByRole("button", { name: "保存代理设置" }));

    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith(
        "save_proxy_settings",
        expect.objectContaining({ settings: dashboardState.config.proxy }),
      ),
    );
  });

  it("lets a monitored service be hidden from the menu bar independently", async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.click(await screen.findByRole("button", { name: "监控" }));
    await user.click(screen.getByRole("switch", { name: "在状态栏显示 Codex" }));

    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("set_status_bar_services", {
        serviceIds: ["kimi"],
      }),
    );
  });

  it("independently changes every menu bar display element", async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.click(await screen.findByRole("button", { name: "监控" }));

    await user.click(screen.getByRole("switch", { name: "状态栏显示服务图标" }));
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("set_status_bar_display", {
        display: {
          showIcon: false,
          showPercentage: true,
          showStateText: true,
        },
      }),
    );

    await user.click(screen.getByRole("switch", { name: "状态栏显示百分比" }));
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("set_status_bar_display", {
        display: {
          showIcon: true,
          showPercentage: false,
          showStateText: true,
        },
      }),
    );

    await user.click(screen.getByRole("switch", { name: "状态栏显示状态文字" }));
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("set_status_bar_display", {
        display: {
          showIcon: true,
          showPercentage: true,
          showStateText: false,
        },
      }),
    );
  });

  it("adds a named Kimi account from monitoring settings", async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.click(await screen.findByRole("button", { name: "监控" }));
    await user.click(screen.getByRole("button", { name: "添加 Kimi 账号" }));
    await user.type(screen.getByRole("textbox", { name: "账号名称" }), "团队账号");
    await user.type(screen.getByLabelText("Kimi API Key"), "sk-team-secret");
    await user.click(screen.getByRole("button", { name: "保存 Kimi 账号" }));

    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("add_kimi_account", {
        displayName: "团队账号",
        apiKey: "sk-team-secret",
        backend: "keychain",
      }),
    );
  });

  it("runs account icon actions on release instead of press", async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.click(await screen.findByRole("button", { name: "监控" }));
    const rename = screen.getByRole("button", {
      name: "重命名 Kimi 工作账号",
    });

    fireEvent.pointerDown(rename);
    expect(screen.queryByRole("textbox", { name: "新账号名称" })).not.toBeInTheDocument();

    fireEvent.click(rename);
    const editor = screen.getByRole("textbox", { name: "新账号名称" });
    expect(editor).toBeInTheDocument();
    expect(within(rename.closest(".account-row") as HTMLElement).getByText("Kimi Code")).toBeInTheDocument();

    await user.keyboard("{Escape}");
    expect(screen.queryByRole("textbox", { name: "新账号名称" })).not.toBeInTheDocument();
  });
});
