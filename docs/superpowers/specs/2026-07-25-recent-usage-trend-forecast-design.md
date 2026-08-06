# Recent Usage Trend Forecast Design

**Date:** 2026-07-25

## Goal

Replace the misleading full-window average forecast with a recent-trend forecast
and a chart that makes the relationship between time, observed utilization, and
projected utilization visible.

The primary case is a seven-day quota whose usage is uneven: old bursts must not
dominate the estimate after the user's recent consumption pattern changes.

## Scope

- Persist successful quota samples for each account and tier.
- Forecast the active seven-day tier from recent samples.
- Render observed and projected utilization on a time-series chart.
- Explain the recent pace, projected reset utilization, and exhaustion time in
  plain Chinese.
- Preserve the existing saturation-event behavior after a weekly quota reaches
  100%.
- Keep the five-hour tier's existing meter and estimate behavior.

## Data Model and Persistence

Usage history is stored separately from user configuration at
`<config-dir>/usage-history.json`.

Each sample contains:

- account/service estimator key;
- tier name;
- reset timestamp, which identifies the quota cycle;
- observation timestamp;
- cumulative utilization percentage.

Only successful provider responses are recorded. Duplicate observations for the
same account, tier, reset, and timestamp are replaced. Samples are sorted and
bounded:

- keep at most the active cycle plus eight days of recent observations;
- keep at most 2,500 samples per account/tier;
- remove an account's history when the account is removed.

The file is written through a temporary sibling and rename so an interrupted
write does not corrupt the last good history. A missing or malformed file is
treated as empty history and logged without breaking quota refresh.

## Forecast Algorithm

For a weekly tier, select samples that match the tier's current reset timestamp.
The estimator has two paths:

- Stable short-window path: inspect successful observations from the latest 30
  minutes and require at least five samples spanning at least 20 minutes. The
  newest sample must be no more than 10 minutes old, adjacent samples may be no
  more than 10 minutes apart, and total utilization growth must be at least two
  percentage points. The weighted fit must imply at least six percentage points
  per hour and achieve an R-squared value of at least 0.8. This path takes
  precedence only after the short-window signal is continuous and stable.
- Normal path: use the latest 24 hours when they contain at least three distinct
  observations spanning at least 30 minutes. If that is insufficient, expand to
  48 hours.

If neither path qualifies, return an unknown recent-trend forecast instead of
falling back to the full-cycle average.

Fit utilization against time with weighted least squares:

- normalize time to hours before the newest sample;
- apply exponentially increasing recency weights with a four-hour half-life;
- clamp the fitted slope to zero or greater;
- reject non-finite results.

The fitted slope is expressed as percentage points per hour. Projection starts
at the current observed utilization, not at the regression intercept:

`projected = current + slope * remaining_hours`

The estimate state uses the existing thresholds:

- below 85% at reset: enough;
- 85% through 100%: tight;
- over 100%: not enough.

When the projected value exceeds 100%, compute the duration from now to
exhaustion. When a recorded weekly saturation event already exists, keep the
existing frozen post-saturation estimate.

The estimate response also carries the selected trend window, slope, observation
span, and chart series. This keeps numerical behavior authoritative in Rust and
the frontend focused on presentation.

## Chart and UI

Each account quota card renders a weekly trend figure below the tier meters.

- Horizontal axis: current weekly cycle start through reset.
- Vertical axis: cumulative utilization, labeled at 0%, 50%, and 100%.
- Solid line: successful observed samples in the current cycle.
- Vertical marker: now.
- Dashed line: projection from the latest real point toward reset.
- If exhaustion occurs first, the projected line stops at 100% and marks the
  predicted exhaustion time.

The chart uses a small native SVG component and existing design tokens. It does
not add a chart dependency. Solid/dashed line styles, legend text, and a textual
summary ensure color is not the only carrier of meaning. The figure receives a
complete accessible label.

Example summary:

> 近 24 小时趋势：每小时增加 1.4%。按当前趋势预计 18 小时后耗尽，比重置早 9 小时。

Stable short-window forecasts are labeled with their actual confirmed span:

> 最近 20 分钟稳定趋势：每小时增加 12%。按当前趋势预计 4 小时后耗尽。

When history is insufficient, the observed series is still shown and the summary
says that short-term prediction needs at least five samples stably covering 20
minutes. On refresh failure, the last successful samples remain visible together
with the existing stale/error state.

## Data Flow

1. The scheduler or manual refresh receives a successful provider quota.
2. The history store records current samples and prunes old data.
3. Dashboard construction loads the matching samples.
4. The estimator computes the weekly recent-trend result and chart series.
5. The card snapshot serializes the result to the frontend.
6. The quota card renders the SVG and plain-language result.

## Error and Boundary Handling

- Missing or invalid reset timestamps yield no trend forecast.
- A reset timestamp change starts a new chart cycle.
- Decreasing utilization inside one reset cycle is not modeled as negative
  consumption; the slope is clamped to zero.
- Duplicate timestamps and zero-variance time series cannot produce division by
  zero.
- Stable short-window prediction rejects fewer than five samples, less than 20
  minutes of coverage, stale or discontinuous samples, less than two percentage
  points of total growth, slopes below six percentage points per hour, and
  weighted fits with R-squared below 0.8. This accounts for providers that expose
  integer percentages and prevents the common 1% over five minutes = 12% per
  hour false positive.
- Provider failures never append samples.
- History persistence errors are logged and do not discard current quota data.
- Percentages above 100 remain valid projected values, while chart coordinates
  are capped at 100 for readability.

## Testing and Verification

Rust tests cover:

- a recent spike outweighing old low activity;
- insufficient recent data returning unknown;
- a two-point 26% to 28% jump over five minutes being rejected;
- three samples over ten minutes being rejected;
- five linear samples over twenty minutes starting a stable short-window
  prediction;
- an isolated final-sample burst failing the fit-quality gate;
- overly close refreshes being rejected;
- 24-hour selection and 48-hour fallback;
- flat and decreasing samples producing a zero slope;
- projected reset utilization and exhaustion time;
- cycle separation, pruning, persistence round-trip, and malformed history.

Frontend tests cover:

- actual and projected SVG series;
- slope and exhaustion summary;
- insufficient-data state;
- accessible figure labeling;
- the existing no-data, error, stale, and success card states.

Completion verification runs Rust tests, frontend tests, TypeScript typecheck,
and the renderer production build. A rendered browser check confirms line
placement, labels, responsive layout, and the insufficient-data state.
