import { ChartSpline } from "lucide-react";
import { useTranslations } from "../i18n";
import type { Translator } from "../i18n/translate";
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
  const t = useTranslations("dashboard");
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
        <span className="trend-pending-mark" aria-hidden="true">
          <ChartSpline size={16} strokeWidth={1.75} />
        </span>
        <div>
          <strong>{t("trend_pending_title")}</strong>
          <p>{t("trend_pending_body")}</p>
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
  const accessibleLabel = chartAccessibleLabel(latestObserved, projectedEnd, t);

  return (
    <div className="usage-trend">
      <div className="trend-legend" aria-hidden="true">
        <span>
          <i className="legend-line observed" />
          {t("legend_observed")}
        </span>
        <span>
          <i className="legend-line projected" />
          {t("legend_projected")}
        </span>
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
            {t("now")}
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
            {t("axis_reset", { date: formatAxisDate(windowEnd) })}
          </text>
        </svg>
      </figure>
      <p className="trend-summary">{trendSummary(estimate, t)}</p>
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

function trendSummary(
  estimate: QuotaEstimate,
  t: Translator<"dashboard">,
): string {
  if (
    estimate.exhaustedBeforeResetSecs != null &&
    estimate.exhaustedBeforeResetSecs >= 0
  ) {
    return t("trend_exhausted_early", {
      duration: formatDuration(estimate.exhaustedBeforeResetSecs, t),
    });
  }
  if (estimate.slopePctPerHour == null || estimate.trendWindowHours == null) {
    return t("trend_accumulating");
  }

  const observedSpanSecs = estimate.observedSpanSecs;
  const isStableShortTrend =
    observedSpanSecs != null &&
    observedSpanSecs >= 20 * 60 &&
    observedSpanSecs < 30 * 60;
  const windowLabel = isStableShortTrend
    ? t("trend_window_short", { span: formatShortSpan(observedSpanSecs, t) })
    : t("trend_window_hours", { hours: estimate.trendWindowHours });
  const pace =
    estimate.slopePctPerHour === 0
      ? t("trend_flat")
      : t("trend_slope", { rate: formatDecimal(estimate.slopePctPerHour) });
  if (
    estimate.state === "not_enough" &&
    estimate.lastsForSecs != null &&
    estimate.resetInSecs != null
  ) {
    const earlyBy = Math.max(0, estimate.resetInSecs - estimate.lastsForSecs);
    return t("trend_summary_exhaust", {
      window: windowLabel,
      pace,
      duration: formatDuration(estimate.lastsForSecs, t),
      early: formatDuration(earlyBy, t),
    });
  }
  if (estimate.projectedUtilization != null) {
    return t("trend_summary_projected", {
      window: windowLabel,
      pace,
      percent: Math.round(estimate.projectedUtilization),
    });
  }
  return t("trend_accumulating");
}

function chartAccessibleLabel(
  latestObserved: UsageChartPoint | undefined,
  projectedEnd: UsageChartPoint | undefined,
  t: Translator<"dashboard">,
): string {
  const actual = latestObserved
    ? t("chart_actual", { percent: Math.round(latestObserved.utilization) })
    : t("chart_actual_empty");
  const projected = projectedEnd
    ? t("chart_projected", { percent: Math.round(projectedEnd.utilization) })
    : t("chart_projected_empty");
  return t("chart_aria", { actual, projected });
}

function formatShortSpan(seconds: number, t: Translator<"dashboard">): string {
  return t("span_minutes", {
    minutes: Math.max(1, Math.round(seconds / 60)),
  });
}

function formatDecimal(value: number): string {
  return value.toFixed(1).replace(/\.0$/, "");
}

function formatDuration(seconds: number, t: Translator<"dashboard">): string {
  const safeSeconds = Math.max(0, seconds);
  const days = Math.floor(safeSeconds / 86_400);
  const hours = Math.floor((safeSeconds % 86_400) / 3_600);
  const minutes = Math.floor((safeSeconds % 3_600) / 60);
  if (days > 0 && hours > 0) {
    return t("duration_days_hours", { days, hours });
  }
  if (days > 0) return t("duration_days", { days });
  if (hours > 0 && minutes > 0) {
    return t("duration_hours_minutes", { hours, minutes });
  }
  if (hours > 0) return t("duration_hours", { hours });
  return t("duration_minutes", { minutes });
}

function formatAxisDate(timestamp: number): string {
  const date = new Date(timestamp * 1_000);
  return `${date.getMonth() + 1}/${date.getDate()}`;
}
