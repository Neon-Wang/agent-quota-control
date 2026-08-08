import { screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { renderWithI18n as render } from "../test/render";
import type { QuotaEstimate } from "../types";
import { UsageTrendChart } from "./UsageTrendChart";

const NOW = 1_700_000_000;

function trendEstimate(overrides: Partial<QuotaEstimate> = {}): QuotaEstimate {
  return {
    state: "not_enough",
    projectedUtilization: 108,
    resetInSecs: 97_200,
    lastsForSecs: 64_800,
    slopePctPerHour: 1.4,
    trendWindowHours: 24,
    observedSpanSecs: 86_400,
    windowStartSecs: NOW - 6 * 86_400,
    windowEndSecs: NOW + 97_200,
    observedPoints: [
      { observedAtSecs: NOW - 86_400, utilization: 20 },
      { observedAtSecs: NOW - 43_200, utilization: 24 },
      { observedAtSecs: NOW, utilization: 70 },
    ],
    projectedPoints: [
      { observedAtSecs: NOW, utilization: 70 },
      { observedAtSecs: NOW + 64_800, utilization: 100 },
    ],
    ...overrides,
  };
}

describe("UsageTrendChart", () => {
  it("renders a forecast-first report without web-style tags", () => {
    render(<UsageTrendChart estimate={trendEstimate()} nowSecs={NOW} />);

    expect(
      screen.getByRole("img", { name: /实际用量.*近期趋势预测/ }),
    ).toBeInTheDocument();
    expect(screen.getByText("预计 18 小时后用完")).toBeInTheDocument();
    expect(
      screen.getByText(
        "根据最近 24 小时的变化，用量大约每小时增加 1.4%。",
      ),
    ).toBeInTheDocument();
    expect(screen.queryByText(/提前/)).not.toBeInTheDocument();
    expect(document.querySelector(".trend-outcome")).toBeNull();
    expect(document.querySelector(".trend-exhaustion-point")).not.toBeNull();
    expect(
      screen.getByTestId("observed-usage-line").getAttribute("stroke"),
    ).toMatch(/^url\(#trend-stroke-/);
  });

  it("keeps past-tense copy only when exhaustion already happened", () => {
    render(
      <UsageTrendChart
        estimate={trendEstimate({
          exhaustedAtSecs: NOW - 3_600,
          exhaustedBeforeResetSecs: 86_400,
          lastsForSecs: 200_000,
          projectedPoints: [
            { observedAtSecs: NOW - 10_000, utilization: 100 },
            { observedAtSecs: NOW, utilization: 100 },
          ],
        })}
        nowSecs={NOW}
      />,
    );

    expect(screen.getByText("本周期已在重置前用完")).toBeInTheDocument();
    expect(screen.getByText("比本次重置大约提前了 1 天。")).toBeInTheDocument();
    expect(screen.queryByText(/预计 .*后用完/)).not.toBeInTheDocument();
  });

  it("labels a healthy forecast with projected reset usage", () => {
    render(
      <UsageTrendChart
        estimate={trendEstimate({
          state: "enough",
          projectedUtilization: 62,
          lastsForSecs: 200_000,
          slopePctPerHour: 12,
          trendWindowHours: 24,
          observedSpanSecs: 1_200,
          observedPoints: [
            { observedAtSecs: NOW - 1_200, utilization: 46 },
            { observedAtSecs: NOW - 900, utilization: 47 },
            { observedAtSecs: NOW - 600, utilization: 48 },
            { observedAtSecs: NOW - 300, utilization: 49 },
            { observedAtSecs: NOW, utilization: 50 },
          ],
          projectedPoints: [
            { observedAtSecs: NOW, utilization: 50 },
            { observedAtSecs: NOW + 86_400, utilization: 62 },
          ],
        })}
        nowSecs={NOW}
      />,
    );

    expect(screen.getByText("预计重置时约 62%")).toBeInTheDocument();
    expect(
      screen.getByText(
        "根据最近 20 分钟的变化，用量大约每小时增加 12%。",
      ),
    ).toBeInTheDocument();
  });

  it("shows observed history without inventing a projection while samples accumulate", () => {
    render(
      <UsageTrendChart
        estimate={trendEstimate({
          state: "unknown",
          projectedUtilization: null,
          lastsForSecs: null,
          slopePctPerHour: null,
          trendWindowHours: null,
          projectedPoints: [],
        })}
        nowSecs={NOW}
      />,
    );

    expect(screen.getByText("正在积累趋势数据")).toBeInTheDocument();
    expect(screen.getByTestId("observed-usage-line")).toBeInTheDocument();
    expect(screen.queryByTestId("projected-usage-line")).not.toBeInTheDocument();
  });

  it("keeps quiet chrome without a now label or red 100% limit line", () => {
    render(
      <UsageTrendChart
        estimate={trendEstimate({
          windowStartSecs: NOW,
          windowEndSecs: NOW + 7 * 86_400,
          observedPoints: [
            { observedAtSecs: NOW, utilization: 0 },
            { observedAtSecs: NOW + 300, utilization: 1 },
          ],
          projectedPoints: [
            { observedAtSecs: NOW + 300, utilization: 1 },
            { observedAtSecs: NOW + 28_800, utilization: 100 },
          ],
        })}
        nowSecs={NOW}
      />,
    );

    expect(screen.queryByText("现在")).not.toBeInTheDocument();
    expect(document.querySelector(".trend-limit-line")).toBeNull();
    expect(screen.getByText("100%")).toBeInTheDocument();
    expect(screen.queryByText("50%")).not.toBeInTheDocument();
    expect(document.querySelector(".trend-now-line")).not.toBeNull();
  });

  it("uses a compact pending state instead of an empty chart for a single sample", () => {
    render(
      <UsageTrendChart
        estimate={trendEstimate({
          state: "unknown",
          projectedUtilization: null,
          lastsForSecs: null,
          slopePctPerHour: null,
          trendWindowHours: null,
          observedPoints: [{ observedAtSecs: NOW, utilization: 26 }],
          projectedPoints: [],
        })}
        nowSecs={NOW}
      />,
    );

    expect(screen.getByRole("status")).toHaveTextContent("趋势图正在建立");
    expect(
      screen.getByText(/短期预测至少需要 5 个样本并稳定覆盖 20 分钟/),
    ).toBeInTheDocument();
    expect(screen.queryByRole("img")).not.toBeInTheDocument();
  });
});
