import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
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
  it("renders actual and projected lines with an accessible recent-pace summary", () => {
    render(<UsageTrendChart estimate={trendEstimate()} nowSecs={NOW} />);

    expect(
      screen.getByRole("img", { name: /实际用量.*近期趋势预测/ }),
    ).toBeInTheDocument();
    expect(screen.getByText("实际用量")).toBeInTheDocument();
    expect(screen.getByText("近期趋势预测")).toBeInTheDocument();
    expect(screen.getByText(/每小时增加 1.4%/)).toBeInTheDocument();
    expect(screen.getByText(/预计 18 小时后耗尽，比重置早 9 小时/)).toBeInTheDocument();
    expect(screen.getByTestId("observed-usage-line")).toHaveAttribute("points");
    expect(screen.getByTestId("projected-usage-line")).toHaveAttribute("points");
  });

  it("labels a short two-sample forecast as a rapid trend", () => {
    render(
      <UsageTrendChart
        estimate={trendEstimate({
          slopePctPerHour: 24,
          trendWindowHours: 24,
          observedSpanSecs: 300,
          observedPoints: [
            { observedAtSecs: NOW - 300, utilization: 26 },
            { observedAtSecs: NOW, utilization: 28 },
          ],
        })}
        nowSecs={NOW}
      />,
    );

    expect(screen.getByText(/最近 5 分钟快速趋势/)).toBeInTheDocument();
    expect(screen.getByText(/每小时增加 24%/)).toBeInTheDocument();
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
      screen.getByText(/出现明显用量跃升时可提前启动快速预测/),
    ).toBeInTheDocument();
    expect(screen.queryByRole("img")).not.toBeInTheDocument();
    expect(screen.queryByText("实际用量")).not.toBeInTheDocument();
  });
});
