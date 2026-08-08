import { ChartSpline } from "lucide-react";
import { useId } from "react";
import { useTranslations } from "../i18n";
import type { Translator } from "../i18n/translate";
import type { QuotaEstimate, UsageChartPoint } from "../types";
import {
  USAGE_GRADIENT_STOPS,
  usageTone,
  type UsageTone,
} from "../usageColor";

interface UsageTrendChartProps {
  estimate: QuotaEstimate;
  nowSecs?: number;
}

const VIEWBOX_WIDTH = 360;
const VIEWBOX_HEIGHT = 158;
const PLOT_LEFT = 28;
const PLOT_RIGHT = 12;
const PLOT_TOP = 16;
const PLOT_BOTTOM = 26;
const PLOT_WIDTH = VIEWBOX_WIDTH - PLOT_LEFT - PLOT_RIGHT;
const PLOT_HEIGHT = VIEWBOX_HEIGHT - PLOT_TOP - PLOT_BOTTOM;
/** Dense samples look like a polyline; draw through fewer anchors so cubics read as smooth. */
const CURVE_ANCHORS = 15;

const TONE_CSS: Record<UsageTone, string> = {
  ok: "var(--ok-fill)",
  warn: "var(--warn-fill)",
  danger: "var(--danger)",
};

export function UsageTrendChart({
  estimate,
  nowSecs = Math.floor(Date.now() / 1_000),
}: UsageTrendChartProps) {
  const t = useTranslations("dashboard");
  const gradientId = useId().replace(/:/g, "");
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

  const observedCoords = toPlotPoints(observedPoints, windowStart, windowEnd);
  const projectedCoords = toPlotPoints(projectedPoints, windowStart, windowEnd);
  const observedAnchors = toCurveAnchors(observedCoords, CURVE_ANCHORS);
  const projectedAnchors = toCurveAnchors(
    projectedCoords,
    Math.min(CURVE_ANCHORS, Math.max(4, projectedCoords.length)),
  );
  const observedPath = toMonotonePath(observedAnchors);
  const projectedPath = toMonotonePath(projectedAnchors);
  const observedAreaPath = toAreaPath(observedAnchors);
  const nowX = xCoordinate(nowSecs, windowStart, windowEnd);
  const latestObserved = observedPoints[observedPoints.length - 1];
  const projectedEnd = projectedPoints[projectedPoints.length - 1];
  const accessibleLabel = chartAccessibleLabel(latestObserved, projectedEnd, t);
  const showExhaustion =
    projectedEnd?.utilization === 100 &&
    projectedEnd.observedAtSecs < windowEnd;
  const strokeGradientId = `trend-stroke-${gradientId}`;
  const areaGradientId = `trend-area-${gradientId}`;
  const yAt0 = yCoordinate(0);
  const yAt100 = yCoordinate(100);
  const currentTone = usageTone(latestObserved?.utilization ?? 0);
  const currentCss = TONE_CSS[currentTone];

  return (
    <div className={`usage-trend usage-trend-${currentTone}`}>
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
          <defs>
            <linearGradient
              id={strokeGradientId}
              gradientUnits="userSpaceOnUse"
              x1={PLOT_LEFT}
              y1={yAt0}
              x2={PLOT_LEFT}
              y2={yAt100}
            >
              {USAGE_GRADIENT_STOPS.map((stop) => (
                <stop
                  key={`${stop.offset}-${stop.tone}`}
                  offset={`${stop.offset}%`}
                  stopColor={TONE_CSS[stop.tone]}
                />
              ))}
            </linearGradient>
            {/* Fade to fully clear at the baseline — object box bottom = chart floor. */}
            <linearGradient
              id={areaGradientId}
              gradientUnits="objectBoundingBox"
              x1="0"
              y1="0"
              x2="0"
              y2="1"
            >
              <stop offset="0%" stopColor={currentCss} stopOpacity="0.32" />
              <stop offset="35%" stopColor={currentCss} stopOpacity="0.14" />
              <stop offset="72%" stopColor={currentCss} stopOpacity="0.04" />
              <stop offset="100%" stopColor={currentCss} stopOpacity="0" />
            </linearGradient>
          </defs>

          {[0, 100].map((utilization) => {
            const y = yCoordinate(utilization);
            return (
              <g key={utilization}>
                <line
                  className="trend-grid-line"
                  x1={PLOT_LEFT}
                  x2={VIEWBOX_WIDTH - PLOT_RIGHT}
                  y1={y}
                  y2={y}
                />
                <text className="trend-axis-label" x={PLOT_LEFT - 6} y={y + 3.5}>
                  {utilization}%
                </text>
              </g>
            );
          })}

          <line
            className="trend-now-line"
            x1={nowX}
            x2={nowX}
            y1={PLOT_TOP + 6}
            y2={PLOT_TOP + PLOT_HEIGHT - 4}
          />

          {observedAreaPath && (
            <path
              className="trend-observed-area"
              d={observedAreaPath}
              fill={`url(#${areaGradientId})`}
            />
          )}
          {observedPath && (
            <path
              data-testid="observed-usage-line"
              className="trend-observed-line"
              d={observedPath}
              stroke={`url(#${strokeGradientId})`}
            />
          )}
          {projectedPath && (
            <path
              data-testid="projected-usage-line"
              className="trend-projected-line"
              d={projectedPath}
              stroke={`url(#${strokeGradientId})`}
            />
          )}

          {latestObserved && (
            <g
              className="trend-current-mark"
              transform={`translate(${xCoordinate(
                latestObserved.observedAtSecs,
                windowStart,
                windowEnd,
              )} ${yCoordinate(latestObserved.utilization)})`}
            >
              <circle className="trend-observed-halo" r="8" fill={currentCss} />
              <circle className="trend-observed-point" r="3.4" fill={currentCss} />
            </g>
          )}
          {showExhaustion && projectedEnd && (
            <circle
              className="trend-exhaustion-point"
              cx={xCoordinate(
                projectedEnd.observedAtSecs,
                windowStart,
                windowEnd,
              )}
              cy={yCoordinate(100)}
              r="3.1"
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
      <TrendReport estimate={estimate} nowSecs={nowSecs} t={t} />
    </div>
  );
}

interface TrendReportModel {
  primary: string;
  detail: string | null;
}

function TrendReport({
  estimate,
  nowSecs,
  t,
}: {
  estimate: QuotaEstimate;
  nowSecs: number;
  t: Translator<"dashboard">;
}) {
  const report = buildTrendReport(estimate, nowSecs, t);
  return (
    <div className="trend-report" role="status">
      <p className="trend-report-primary">{report.primary}</p>
      {report.detail ? <p className="trend-report-detail">{report.detail}</p> : null}
    </div>
  );
}

interface PlotPoint {
  x: number;
  y: number;
}

function toPlotPoints(
  points: UsageChartPoint[],
  windowStart: number,
  windowEnd: number,
): PlotPoint[] {
  return points.map((point) => ({
    x: xCoordinate(point.observedAtSecs, windowStart, windowEnd),
    y: yCoordinate(point.utilization),
  }));
}

/** Evenly spaced anchors (+ curvature picks) so cubic segments are long enough to read as smooth. */
function toCurveAnchors(points: PlotPoint[], target: number): PlotPoint[] {
  if (points.length <= target) return points;

  const keep = new Set<number>();
  for (let i = 0; i < target; i += 1) {
    keep.add(Math.round((i / (target - 1)) * (points.length - 1)));
  }

  const ranked = points
    .map((point, index) => {
      if (index === 0 || index === points.length - 1) {
        return { index, score: Number.POSITIVE_INFINITY };
      }
      const prev = points[index - 1];
      const next = points[index + 1];
      const span = Math.max(1e-6, next.x - prev.x);
      const expectedY = prev.y + ((point.x - prev.x) / span) * (next.y - prev.y);
      return { index, score: Math.abs(point.y - expectedY) };
    })
    .sort((a, b) => b.score - a.score);

  for (const item of ranked) {
    if (keep.size >= target + 3) break;
    keep.add(item.index);
  }

  return [...keep]
    .sort((a, b) => a - b)
    .map((index) => points[index]);
}

/**
 * Monotone cubic Hermite (Fritsch–Carlson) so the stroke never dips below
 * a prior sample — matches non-decreasing utilization.
 */
function toMonotonePath(points: PlotPoint[]): string | null {
  if (points.length < 2) return null;
  if (points.length === 2) {
    return `M ${fmt(points[0].x)} ${fmt(points[0].y)} L ${fmt(points[1].x)} ${fmt(points[1].y)}`;
  }

  const xs = points.map((point) => point.x);
  const ys = points.map((point) => point.y);
  const n = points.length;
  const delta: number[] = [];
  for (let i = 0; i < n - 1; i += 1) {
    const dx = xs[i + 1] - xs[i];
    delta.push(dx === 0 ? 0 : (ys[i + 1] - ys[i]) / dx);
  }

  const slopes = new Array<number>(n);
  slopes[0] = delta[0];
  slopes[n - 1] = delta[n - 2];
  for (let i = 1; i < n - 1; i += 1) {
    if (delta[i - 1] * delta[i] <= 0) {
      slopes[i] = 0;
    } else {
      slopes[i] = (delta[i - 1] + delta[i]) / 2;
    }
  }

  for (let i = 0; i < n - 1; i += 1) {
    if (Math.abs(delta[i]) < 1e-8) {
      slopes[i] = 0;
      slopes[i + 1] = 0;
      continue;
    }
    const a = slopes[i] / delta[i];
    const b = slopes[i + 1] / delta[i];
    const s = a * a + b * b;
    if (s > 9) {
      const t = 3 / Math.sqrt(s);
      slopes[i] = t * a * delta[i];
      slopes[i + 1] = t * b * delta[i];
    }
  }

  let d = `M ${fmt(xs[0])} ${fmt(ys[0])}`;
  for (let i = 0; i < n - 1; i += 1) {
    const h = xs[i + 1] - xs[i];
    d += ` C ${fmt(xs[i] + h / 3)} ${fmt(ys[i] + (slopes[i] * h) / 3)}, ${fmt(
      xs[i + 1] - h / 3,
    )} ${fmt(ys[i + 1] - (slopes[i + 1] * h) / 3)}, ${fmt(xs[i + 1])} ${fmt(ys[i + 1])}`;
  }
  return d;
}

function toAreaPath(points: PlotPoint[]): string | null {
  const stroke = toMonotonePath(points);
  if (!stroke || points.length < 2) return null;
  const baseline = PLOT_TOP + PLOT_HEIGHT;
  const first = points[0];
  const last = points[points.length - 1];
  return `${stroke} L ${fmt(last.x)} ${fmt(baseline)} L ${fmt(first.x)} ${fmt(baseline)} Z`;
}

function fmt(value: number): string {
  return value.toFixed(1);
}

function xCoordinate(timestamp: number, windowStart: number, windowEnd: number): number {
  const ratio = (timestamp - windowStart) / (windowEnd - windowStart);
  return PLOT_LEFT + Math.min(Math.max(ratio, 0), 1) * PLOT_WIDTH;
}

function yCoordinate(utilization: number): number {
  const normalized = Math.min(Math.max(utilization, 0), 100) / 100;
  return PLOT_TOP + (1 - normalized) * PLOT_HEIGHT;
}

function buildTrendReport(
  estimate: QuotaEstimate,
  nowSecs: number,
  t: Translator<"dashboard">,
): TrendReportModel {
  // Backend only sets exhaustedAtSecs when usage already hit 100% this cycle.
  const alreadyExhausted =
    estimate.exhaustedAtSecs != null && estimate.exhaustedAtSecs <= nowSecs;
  if (alreadyExhausted) {
    const early =
      estimate.exhaustedBeforeResetSecs != null
        ? formatDuration(estimate.exhaustedBeforeResetSecs, t)
        : null;
    return {
      primary: t("trend_primary_exhausted"),
      detail: early ? t("trend_detail_exhausted", { early }) : null,
    };
  }

  if (estimate.slopePctPerHour == null || estimate.trendWindowHours == null) {
    return {
      primary: t("trend_accumulating"),
      detail: null,
    };
  }

  const observedSpanSecs = estimate.observedSpanSecs;
  const isStableShortTrend =
    observedSpanSecs != null &&
    observedSpanSecs >= 20 * 60 &&
    observedSpanSecs < 30 * 60;
  const windowLabel = isStableShortTrend
    ? t("trend_window_short", { span: formatShortSpan(observedSpanSecs, t) })
    : t("trend_window_hours", { hours: estimate.trendWindowHours });
  const isFlat = estimate.slopePctPerHour === 0;
  const rate = formatDecimal(estimate.slopePctPerHour ?? 0);

  const paceDetail = isFlat
    ? t("trend_detail_pace_flat", { window: windowLabel })
    : t("trend_detail_pace_up", { window: windowLabel, rate });

  if (
    estimate.state === "not_enough" &&
    estimate.lastsForSecs != null &&
    estimate.resetInSecs != null
  ) {
    // One timeline only: when it runs out. Don't also say "early by X" —
    // that second countdown fights the primary and reads as a contradiction.
    return {
      primary: t("trend_primary_exhaust", {
        duration: formatDuration(estimate.lastsForSecs, t),
      }),
      detail: paceDetail,
    };
  }

  if (estimate.projectedUtilization != null) {
    const percent = Math.round(estimate.projectedUtilization);
    return {
      primary:
        estimate.state === "tight"
          ? t("trend_primary_at_reset_tight", { percent })
          : t("trend_primary_at_reset", { percent }),
      detail: paceDetail,
    };
  }

  return {
    primary: t("trend_accumulating"),
    detail: null,
  };
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
