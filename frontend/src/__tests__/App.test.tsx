import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "../App";
import type { DashboardState } from "../types";

const invokeMock = vi.fn();
const fixedNow = new Date("2026-06-04T08:15:00+08:00").getTime();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (command: string, args?: unknown) => invokeMock(command, args),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => undefined)),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ startDragging: vi.fn() }),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

const dashboardState: DashboardState = {
  config: {
    version: 4,
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
    selectedTools: ["codex_cli"],
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
  tools: [
    {
      id: "codex_cli",
      name: "Codex CLI",
      toolType: "cli",
      installed: true,
      installPath: "/Users/test/.local/bin/codex",
      launchAs: null,
    },
    {
      id: "vscode",
      name: "VS Code",
      toolType: "ide",
      installed: true,
      installPath: "/Applications/Visual Studio Code.app",
      launchAs: "Visual Studio Code",
    },
  ],
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
    invokeMock.mockReset();
    invokeMock.mockImplementation((command: string) => {
      if (command === "get_dashboard_state") return Promise.resolve(dashboardState);
      if (command === "set_selected_tools") return Promise.resolve(dashboardState);
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

    expect(within(kimiCard).getByRole("heading", { name: "够" })).toBeInTheDocument();
    expect(within(codexCard).getByRole("heading", { name: "不够" })).toBeInTheDocument();
    expect(within(kimiCard).getByText("Kimi Code")).toBeInTheDocument();
    expect(within(kimiCard).getByText("Kimi 工作账号")).toBeInTheDocument();
    expect(within(codexCard).getByText("Codex 个人账号")).toBeInTheDocument();
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
    expect(screen.getByText("Kimi 当前直连")).toBeInTheDocument();
    expect(screen.getByText("Codex 代理已连接")).toBeInTheDocument();
    expect(screen.getByText("本周内预计够用。")).toBeInTheDocument();
    expect(screen.getByText("预计将在 1 天 2 小时 后耗尽。")).toBeInTheDocument();
    expect(
      screen.getByRole("img", { name: /实际用量.*近期趋势预测/ }),
    ).toBeInTheDocument();
    expect(screen.getByText(/近 24 小时趋势：每小时增加 0.4%/)).toBeInTheDocument();
    expect(screen.queryByText("direct")).not.toBeInTheDocument();
    expect(screen.queryByText("unavailable")).not.toBeInTheDocument();
  });

  it("shows selected and available tools", async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.click(await screen.findByRole("button", { name: "工具" }));
    await user.click(screen.getByRole("button", { name: /工具选择/ }));

    expect(screen.getAllByText("Codex CLI").length).toBeGreaterThan(0);
    expect(screen.getByText("VS Code")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "添加" }));

    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("set_selected_tools", {
        toolIds: ["codex_cli", "vscode"],
      }),
    );
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
    await user.click(screen.getByRole("checkbox", { name: "在状态栏显示 Codex" }));

    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("set_status_bar_services", {
        serviceIds: ["kimi"],
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
});
