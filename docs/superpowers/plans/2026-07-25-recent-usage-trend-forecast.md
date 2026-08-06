# Recent Usage Trend Forecast Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the weekly full-window average with a persisted, recent-weighted trend forecast and show observed versus projected utilization in an accessible line chart.

**Architecture:** A focused Rust history store persists successful samples outside user configuration. The estimator owns weighted regression and projection, while the existing card snapshot carries the authoritative forecast and chart points to a dependency-free React SVG component.

**Tech Stack:** Rust, Serde, Chrono, Tauri 2, React 18, TypeScript, native SVG, Cargo tests, Vitest.

---

No commit step is included because this checkout already contains user-owned
uncommitted work and the current authorization does not include committing.

### Task 1: Define Trend Types and Weighted Forecast

**Files:**
- Modify: `src-tauri/src/types.rs`
- Modify: `src-tauri/src/estimator.rs`

- [x] **Step 1: Write failing estimator tests**

Add tests that construct `UsageSample` values and assert:

```rust
let estimate = estimate_recent_weekly_trend(&tier, NOW, &samples, None);
assert_eq!(estimate.trend_window_hours, Some(24));
assert!(estimate.slope_pct_per_hour.unwrap() > 1.0);
assert_eq!(estimate.state, SufficiencyState::NotEnough);
```

Cover a day-five spike, insufficient span, 48-hour fallback, a flat series, and
an exhaustion time before reset.

- [x] **Step 2: Verify the tests fail for the missing API**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml estimator::tests::recent
```

Expected: compilation failure because `UsageSample`,
`estimate_recent_weekly_trend`, and trend response fields do not exist.

- [x] **Step 3: Add the authoritative serializable types**

Add:

```rust
pub struct UsageSample {
    pub service: String,
    pub tier: String,
    pub reset_at: String,
    pub observed_at_secs: i64,
    pub utilization: f64,
}

pub struct UsageChartPoint {
    pub observed_at_secs: i64,
    pub utilization: f64,
}
```

Extend `QuotaEstimate` with serde-defaulted optional fields:
`slope_pct_per_hour`, `trend_window_hours`, `observed_span_secs`,
`window_start_secs`, `window_end_secs`, `observed_points`, and
`projected_points`.

- [x] **Step 4: Implement weighted least-squares projection**

Implement `estimate_recent_weekly_trend` so it:

```rust
let selected = select_recent_samples(samples, now_secs, 24)
    .or_else(|| select_recent_samples(samples, now_secs, 48));
let slope = weighted_slope_pct_per_hour(selected, newest_at, 4.0).max(0.0);
let projected = utilization + slope * reset_in_secs as f64 / 3600.0;
```

It must start the projection at the current tier value, preserve saturation
event handling, return unknown without adequate samples, and cap only the SVG
projection endpoint at 100%.

- [x] **Step 5: Run the focused tests**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml estimator::tests
```

Expected: all estimator tests pass.

### Task 2: Persist and Prune Successful Usage Samples

**Files:**
- Create: `src-tauri/src/usage_history.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/commands.rs`

- [x] **Step 1: Write failing history-store tests**

Tests use a temporary directory and verify:

```rust
record_quota_samples(&path, "account-a", &quota, NOW)?;
let loaded = load_history_from(&path)?;
assert_eq!(loaded.samples.len(), quota.tiers.len());
```

Also cover deduplication, cycle separation, pruning to eight days and 2,500
samples, malformed JSON returning an empty document, and account deletion.

- [x] **Step 2: Verify the history tests fail**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml usage_history::tests
```

Expected: compilation failure because the module and store functions are absent.

- [x] **Step 3: Implement the bounded atomic store**

Create a `UsageHistoryDocument { version, samples }` with functions:

```rust
pub fn load_history() -> UsageHistoryDocument;
pub fn record_quota_samples(service: &str, quota: &ServiceQuota, now: i64) -> Result<(), String>;
pub fn samples_for(service: &str, tier: &QuotaTier) -> Vec<UsageSample>;
pub fn remove_service(service: &str) -> Result<(), String>;
```

Write JSON to `usage-history.json.tmp`, call `sync_all`, then rename it to
`usage-history.json`. A malformed document logs a warning and returns empty
history.

- [x] **Step 4: Integrate recording and deletion**

In `refresh_usage_inner`, append samples only when `quota.success`. In
`remove_account`, remove history for the estimator key after configuration and
credential cleanup. Declare `mod usage_history` in `lib.rs`.

- [x] **Step 5: Run focused persistence and command tests**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml usage_history
cargo test --manifest-path src-tauri/Cargo.toml account_runtime_tests
```

Expected: all selected tests pass.

### Task 3: Feed Recent History into Account Card Estimates

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/widget_snapshot.rs`

- [x] **Step 1: Write a failing dashboard/card contract test**

Build a current weekly tier and history document, then assert the card estimate
serializes:

```rust
assert_eq!(estimate.trend_window_hours, Some(24));
assert!(!estimate.observed_points.is_empty());
assert_eq!(estimate.projected_points.len(), 2);
```

- [x] **Step 2: Verify the contract test fails**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml widget_snapshot::tests::fresh_quota
```

Expected: failure because dashboard estimate creation still uses the
full-window estimator.

- [x] **Step 3: Route samples through `estimates_for`**

Load history once per `dashboard_state` call. For each tier, select matching
service/tier/reset samples and call `estimate_recent_weekly_trend` for weekly
tiers. Keep `estimate_tier_with_saturation` for five-hour tiers.

- [x] **Step 4: Restore all Rust fixtures**

Update every explicit `QuotaEstimate` test fixture with default trend fields via
`..QuotaEstimate::unknown()` or a test constructor so existing tray and widget
contracts continue to compile.

- [x] **Step 5: Run all Rust tests**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: all Rust tests pass.

### Task 4: Render the Accessible Weekly Trend Chart

**Files:**
- Create: `frontend/src/components/UsageTrendChart.tsx`
- Create: `frontend/src/components/UsageTrendChart.test.tsx`
- Modify: `frontend/src/types.ts`
- Modify: `frontend/src/components/QuotaCard.tsx`
- Modify: `frontend/src/styles.css`
- Modify: `frontend/src/__tests__/App.test.tsx`

- [x] **Step 1: Write failing component tests**

Render an estimate with observed and projected points and assert:

```tsx
expect(screen.getByRole("img", { name: /实际用量.*近期趋势预测/ })).toBeInTheDocument();
expect(screen.getByText("实际用量")).toBeInTheDocument();
expect(screen.getByText("近期趋势预测")).toBeInTheDocument();
expect(screen.getByText(/每小时增加 1.4%/)).toBeInTheDocument();
```

Add a second test where projection fields are absent and assert
`正在积累趋势数据`.

- [x] **Step 2: Verify the component tests fail**

Run:

```bash
pnpm --dir frontend test -- UsageTrendChart.test.tsx
```

Expected: failure because the component does not exist.

- [x] **Step 3: Add frontend trend types and SVG rendering**

Mirror the Rust optional fields in `QuotaEstimate`. Implement a 320×150
responsive SVG with fixed padding, a `viewBox`, 0/50/100 grid labels, observed
polyline, now marker, dashed projected line, and exhaustion marker.

Map coordinates with:

```ts
const x = left + ((timestamp - windowStart) / (windowEnd - windowStart)) * width;
const y = top + (1 - Math.min(Math.max(utilization, 0), 100) / 100) * height;
```

Expose the full visual conclusion through `role="img"` and `aria-label`.

- [x] **Step 4: Integrate chart and explanatory copy**

Render `UsageTrendChart` for the weekly tier below the meters. Replace the old
generic full-window wording with recent-window wording when trend fields exist,
while keeping post-saturation and accumulating-data messages.

- [x] **Step 5: Style with existing tokens and responsive behavior**

Add `.usage-trend`, `.usage-trend-chart`, `.trend-legend`, and line classes.
Use solid/dashed strokes plus labels, preserve visible focus behavior, and avoid
new gradients, oversized rounding, or color-only status.

- [x] **Step 6: Run frontend tests and typecheck**

Run:

```bash
pnpm --dir frontend test
pnpm --dir frontend typecheck
```

Expected: all tests pass and TypeScript exits successfully.

### Task 5: Documentation and Completion Verification

**Files:**
- Modify: `README.md`

- [x] **Step 1: Document recent-trend behavior**

Replace the full-window projection description with the persisted 24-hour
weighted trend, 48-hour fallback, chart semantics, and insufficient-history
behavior.

- [x] **Step 2: Run formatting and complete automated verification**

Run:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml
pnpm --dir frontend test
pnpm --dir frontend typecheck
pnpm --dir frontend build
```

Expected: every command exits zero.

- [x] **Step 3: Run a browser-visible renderer check**

Start the renderer with a deterministic dashboard fixture, open it in the
in-app browser, and verify the actual line, dashed projected line, axes, legend,
plain-language slope, responsive card layout, and accumulating-data state.

- [x] **Step 4: Inspect the final scoped diff**

Run:

```bash
git diff --check
git status --short
git diff -- src-tauri/src/usage_history.rs src-tauri/src/estimator.rs \
  src-tauri/src/types.rs src-tauri/src/commands.rs src-tauri/src/lib.rs \
  frontend/src/components/UsageTrendChart.tsx \
  frontend/src/components/UsageTrendChart.test.tsx \
  frontend/src/components/QuotaCard.tsx frontend/src/types.ts \
  frontend/src/styles.css frontend/src/__tests__/App.test.tsx README.md \
  docs/superpowers/specs/2026-07-25-recent-usage-trend-forecast-design.md \
  docs/superpowers/plans/2026-07-25-recent-usage-trend-forecast.md
```

Expected: no whitespace errors; the scoped diff contains only the forecast
feature plus pre-existing overlapping edits that were preserved.
