import type { QuotaEstimate, UsageChartPoint } from "../types";

interface UsageTrendChartProps {
  estimate: QuotaEstimate;
  nowSecs?: number;
}

const VIEWBOX_WIDTH = 360;
const VIEWBOX_HEIGHT = 166;
const PLOT_LEFT = 38;
const PLOT_RIGHT = 12;
const PLOT_TOP = 24;
const PLOT_BOTTOM = 28;
const PLOT_WIDTH = VIEWBOX_WIDTH - PLOT_LEFT - PLOT_RIGHT;
const PLOT_HEIGHT = VIEWBOX_HEIGHT - PLOT_TOP - PLOT_BOTTOM;
const NOW_LABEL_SIDE_INSET = 18;

export function UsageTrendChart({
  estimate,
  nowSecs = Math.floor(Date.now() / 1_000),
}: UsageTrendChartProps) {
  const observedPoints = estimate.observedPoints ?? [];
  const projectedPoints = estimate.projectedPoints ?? [];
  const windowStart = estimate.windowStartSecs;
  const windowEnd = estimate.windowEndSecs;
  const hasDomain =
    windowStart != null &&
    windowEnd != null &&
    Number.isFinite(windowStart) &&
    Number.isFinite(windowEnd) &&
    windowEnd > windowStart;

  if (!hasDomain) return null;

  if (observedPoints.length < 2) {
    return (
      <div className="trend-pending" role="status" aria-live="polite">
        <span className="trend-pending-mark" aria-hidden="true" />
        <div>
          <strong>趋势图正在建立</strong>
          <p>短期预测至少需要 5 个样本并稳定覆盖 20 分钟；否则继续积累数据。</p>
        </div>
      </div>
    );
  }

  const observedCoordinates = toPolyline(observedPoints, windowStart, windowEnd);
  const projectedCoordinates = toPolyline(projectedPoints, windowStart, windowEnd);
  const nowX = xCoordinate(nowSecs, windowStart, windowEnd);
  const nowLabelX = Math.min(
    Math.max(nowX, PLOT_LEFT + NOW_LABEL_SIDE_INSET),
    VIEWBOX_WIDTH - PLOT_RIGHT - NOW_LABEL_SIDE_INSET,
  );
  const latestObserved = observedPoints[observedPoints.length - 1];
  const projectedEnd = projectedPoints[projectedPoints.length - 1];
  const accessibleLabel = chartAccessibleLabel(latestObserved, projectedEnd);

  return (
    <div className="usage-trend">
      <div className="trend-legend" aria-hidden="true">
        <span><i className="legend-line observed" />实际用量</span>
        <span><i className="legend-line projected" />近期趋势预测</span>
      </div>
      <figure
        className="usage-trend-figure"
        role="img"
        aria-label={accessibleLabel}
      >
        <svg
          className="usage-trend-chart"
          viewBox={`0 0 ${VIEWBOX_WIDTH} ${VIEWBOX_HEIGHT}`}
          aria-hidden="true"
        >
          {[0, 50, 100].map((utilization) => {
            const y = yCoordinate(utilization);
            return (
              <g key={utilization}>
                <line
                  className={utilization === 100 ? "trend-limit-line" : "trend-grid-line"}
                  x1={PLOT_LEFT}
                  x2={VIEWBOX_WIDTH - PLOT_RIGHT}
                  y1={y}
                  y2={y}
                />
                <text className="trend-axis-label" x={PLOT_LEFT - 6} y={y + 4}>
                  {utilization}%
                </text>
              </g>
            );
          })}

          <line
            className="trend-now-line"
            x1={nowX}
            x2={nowX}
            y1={PLOT_TOP}
            y2={PLOT_TOP + PLOT_HEIGHT}
          />
          <text className="trend-now-label" x={nowLabelX} y={PLOT_TOP - 8}>
            现在
          </text>

          {observedCoordinates && (
            <polyline
              data-testid="observed-usage-line"
              className="trend-observed-line"
              points={observedCoordinates}
            />
          )}
          {latestObserved && (
            <circle
              className="trend-observed-point"
              cx={xCoordinate(latestObserved.observedAtSecs, windowStart, windowEnd)}
              cy={yCoordinate(latestObserved.utilization)}
              r="3.5"
            />
          )}
          {projectedCoordinates && (
            <polyline
              data-testid="projected-usage-line"
              className="trend-projected-line"
              points={projectedCoordinates}
            />
          )}
          {projectedEnd?.utilization === 100 &&
            projectedEnd.observedAtSecs < windowEnd && (
              <circle
                className="trend-exhaustion-point"
                cx={xCoordinate(projectedEnd.observedAtSecs, windowStart, windowEnd)}
                cy={yCoordinate(100)}
                r="4"
              />
            )}

          <text className="trend-date-label start" x={PLOT_LEFT} y={VIEWBOX_HEIGHT - 6}>
            {formatAxisDate(windowStart)}
          </text>
          <text
            className="trend-date-label end"
            x={VIEWBOX_WIDTH - PLOT_RIGHT}
            y={VIEWBOX_HEIGHT - 6}
          >
            {formatAxisDate(windowEnd)} 重置
          </text>
        </svg>
      </figure>
      <p className="trend-summary">{trendSummary(estimate)}</p>
    </div>
  );
}

function toPolyline(
  points: UsageChartPoint[],
  windowStart: number,
  windowEnd: number,
): string | null {
  if (points.length < 2) return null;
  return points
    .map(
      (point) =>
        `${xCoordinate(point.observedAtSecs, windowStart, windowEnd).toFixed(1)},${yCoordinate(point.utilization).toFixed(1)}`,
    )
    .join(" ");
}

function xCoordinate(timestamp: number, windowStart: number, windowEnd: number): number {
  const ratio = (timestamp - windowStart) / (windowEnd - windowStart);
  return PLOT_LEFT + Math.min(Math.max(ratio, 0), 1) * PLOT_WIDTH;
}

function yCoordinate(utilization: number): number {
  const normalized = Math.min(Math.max(utilization, 0), 100) / 100;
  return PLOT_TOP + (1 - normalized) * PLOT_HEIGHT;
}

function trendSummary(estimate: QuotaEstimate): string {
  if (
    estimate.exhaustedBeforeResetSecs != null &&
    estimate.exhaustedBeforeResetSecs >= 0
  ) {
    return `本周期已提前 ${formatDuration(estimate.exhaustedBeforeResetSecs)}耗尽。`;
  }
  if (estimate.slopePctPerHour == null || estimate.trendWindowHours == null) {
    return "正在积累趋势数据";
  }

  const observedSpanSecs = estimate.observedSpanSecs;
  const isStableShortTrend =
    observedSpanSecs != null &&
    observedSpanSecs >= 20 * 60 &&
    observedSpanSecs < 30 * 60;
  const windowLabel = isStableShortTrend
    ? `最近 ${formatShortSpan(observedSpanSecs)}稳定趋势`
    : `近 ${estimate.trendWindowHours} 小时趋势`;
  const pace =
    estimate.slopePctPerHour === 0
      ? "用量基本持平"
      : `每小时增加 ${formatDecimal(estimate.slopePctPerHour)}%`;
  if (
    estimate.state === "not_enough" &&
    estimate.lastsForSecs != null &&
    estimate.resetInSecs != null
  ) {
    const earlyBy = Math.max(0, estimate.resetInSecs - estimate.lastsForSecs);
    return `${windowLabel}：${pace}。按当前趋势预计 ${formatDuration(estimate.lastsForSecs)}后耗尽，比重置早 ${formatDuration(earlyBy)}。`;
  }
  if (estimate.projectedUtilization != null) {
    return `${windowLabel}：${pace}。按当前趋势，重置时预计用量 ${Math.round(estimate.projectedUtilization)}%。`;
  }
  return "正在积累趋势数据";
}

function chartAccessibleLabel(
  latestObserved?: UsageChartPoint,
  projectedEnd?: UsageChartPoint,
): string {
  const actual = latestObserved
    ? `实际用量当前为 ${Math.round(latestObserved.utilization)}%`
    : "实际用量暂无数据";
  const projected = projectedEnd
    ? `近期趋势预测到 ${Math.round(projectedEnd.utilization)}%`
    : "近期趋势预测正在积累数据";
  return `7 天用量趋势图：${actual}；${projected}。`;
}

function formatShortSpan(seconds: number): string {
  return `${Math.max(1, Math.round(seconds / 60))} 分钟`;
}

function formatDecimal(value: number): string {
  return value.toFixed(1).replace(/\.0$/, "");
}

function formatDuration(seconds: number): string {
  const safeSeconds = Math.max(0, seconds);
  const days = Math.floor(safeSeconds / 86_400);
  const hours = Math.floor((safeSeconds % 86_400) / 3_600);
  const minutes = Math.floor((safeSeconds % 3_600) / 60);
  if (days > 0 && hours > 0) return `${days} 天 ${hours} 小时`;
  if (days > 0) return `${days} 天`;
  if (hours > 0 && minutes > 0) return `${hours} 小时 ${minutes} 分钟`;
  if (hours > 0) return `${hours} 小时`;
  return `${minutes} 分钟`;
}

function formatAxisDate(timestamp: number): string {
  const date = new Date(timestamp * 1_000);
  return `${date.getMonth() + 1}/${date.getDate()}`;
}
