use crate::types::{
    QuotaEstimate, QuotaSaturationEvent, QuotaTier, ServiceQuota, SufficiencyState,
    UsageChartPoint, UsageSample,
};
use std::time::{SystemTime, UNIX_EPOCH};

const FIVE_HOUR_WINDOW_SECS: i64 = 18_000;
const SEVEN_DAY_WINDOW_SECS: i64 = 604_800;
const MIN_ELAPSED_SECS_FOR_PROJECTION: i64 = 900;
const TIGHT_PROJECTED_PCT: f64 = 85.0;
const EXHAUSTED_PROJECTED_PCT: f64 = 100.0;
const MIN_TREND_SAMPLES: usize = 3;
const MIN_TREND_SPAN_SECS: i64 = 30 * 60;
const TREND_HALF_LIFE_HOURS: f64 = 4.0;
const STABLE_SHORT_WINDOW_SECS: i64 = 30 * 60;
const STABLE_SHORT_MIN_SAMPLES: usize = 5;
const STABLE_SHORT_MIN_SPAN_SECS: i64 = 20 * 60;
const STABLE_SHORT_MAX_SAMPLE_GAP_SECS: i64 = 10 * 60;
const STABLE_SHORT_MAX_AGE_SECS: i64 = 10 * 60;
const STABLE_SHORT_MIN_DELTA_PCT: f64 = 2.0;
const STABLE_SHORT_MIN_SLOPE_PCT_PER_HOUR: f64 = 6.0;
const STABLE_SHORT_MIN_R_SQUARED: f64 = 0.8;

pub fn now_unix_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub fn estimate_tier(tier: &QuotaTier, now_secs: i64) -> QuotaEstimate {
    estimate_tier_with_saturation(tier, now_secs, None)
}

pub fn estimate_tier_with_saturation(
    tier: &QuotaTier,
    now_secs: i64,
    saturation: Option<&QuotaSaturationEvent>,
) -> QuotaEstimate {
    let reset_at = match tier
        .resets_at
        .as_deref()
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
    {
        Some(reset_at) => reset_at.timestamp(),
        None => return unknown_estimate(),
    };

    let reset_in_secs = reset_at - now_secs;
    if reset_in_secs <= 0 {
        return QuotaEstimate {
            reset_in_secs: Some(0),
            ..unknown_estimate()
        };
    }

    let window_secs = match window_seconds(&tier.name) {
        Some(window_secs) => window_secs,
        None => {
            return QuotaEstimate {
                reset_in_secs: Some(reset_in_secs),
                ..unknown_estimate()
            };
        }
    };

    let utilization = tier.utilization.clamp(0.0, 100.0);
    if let Some(event) = saturation {
        if is_weekly_tier(&tier.name) && utilization >= 100.0 {
            let window_start = reset_at - window_secs;
            let elapsed_to_saturation =
                (event.reached_at_secs - window_start).clamp(1, window_secs);
            let projected_utilization = 100.0 * window_secs as f64 / elapsed_to_saturation as f64;
            return QuotaEstimate {
                state: SufficiencyState::NotEnough,
                projected_utilization: Some(projected_utilization.round()),
                reset_in_secs: Some(reset_in_secs),
                lasts_for_secs: Some(elapsed_to_saturation),
                exhausted_at_secs: Some(event.reached_at_secs),
                exhausted_before_reset_secs: Some((reset_at - event.reached_at_secs).max(0)),
                ..empty_trend_fields()
            };
        }
    }

    if utilization == 0.0 {
        return QuotaEstimate {
            state: SufficiencyState::Enough,
            projected_utilization: Some(0.0),
            reset_in_secs: Some(reset_in_secs),
            lasts_for_secs: None,
            exhausted_at_secs: None,
            exhausted_before_reset_secs: None,
            ..empty_trend_fields()
        };
    }

    let elapsed_secs = (window_secs - reset_in_secs).clamp(0, window_secs);
    if elapsed_secs < MIN_ELAPSED_SECS_FOR_PROJECTION {
        return QuotaEstimate {
            reset_in_secs: Some(reset_in_secs),
            ..unknown_estimate()
        };
    }

    let burn_rate = utilization / elapsed_secs as f64;
    if burn_rate <= 0.0 || !burn_rate.is_finite() {
        return QuotaEstimate {
            reset_in_secs: Some(reset_in_secs),
            ..unknown_estimate()
        };
    }

    let projected_utilization = utilization + burn_rate * reset_in_secs as f64;
    let lasts_for_secs = ((100.0 - utilization).max(0.0) / burn_rate).round() as i64;
    let state = if projected_utilization < TIGHT_PROJECTED_PCT {
        SufficiencyState::Enough
    } else if projected_utilization <= EXHAUSTED_PROJECTED_PCT {
        SufficiencyState::Tight
    } else {
        SufficiencyState::NotEnough
    };

    QuotaEstimate {
        state,
        projected_utilization: Some(projected_utilization.round()),
        reset_in_secs: Some(reset_in_secs),
        lasts_for_secs: Some(lasts_for_secs),
        exhausted_at_secs: None,
        exhausted_before_reset_secs: None,
        ..empty_trend_fields()
    }
}

pub fn estimate_recent_weekly_trend(
    tier: &QuotaTier,
    now_secs: i64,
    samples: &[UsageSample],
    saturation: Option<&QuotaSaturationEvent>,
) -> QuotaEstimate {
    if !is_weekly_tier(&tier.name) {
        return estimate_tier_with_saturation(tier, now_secs, saturation);
    }

    let Some(reset_at_text) = tier.resets_at.as_deref() else {
        return unknown_estimate();
    };
    let Some(reset_at) = chrono::DateTime::parse_from_rfc3339(reset_at_text)
        .ok()
        .map(|value| value.timestamp())
    else {
        return unknown_estimate();
    };
    let reset_in_secs = reset_at - now_secs;
    let window_start_secs = reset_at - SEVEN_DAY_WINDOW_SECS;
    if reset_in_secs <= 0 {
        return QuotaEstimate {
            reset_in_secs: Some(0),
            window_start_secs: Some(window_start_secs),
            window_end_secs: Some(reset_at),
            ..unknown_estimate()
        };
    }

    let mut observed_points = samples
        .iter()
        .filter(|sample| {
            sample.tier == tier.name
                && sample.reset_at == reset_at_text
                && sample.observed_at_secs >= window_start_secs
                && sample.observed_at_secs <= now_secs
                && sample.utilization.is_finite()
        })
        .map(|sample| UsageChartPoint {
            observed_at_secs: sample.observed_at_secs,
            utilization: sample.utilization.clamp(0.0, 100.0),
        })
        .collect::<Vec<_>>();
    observed_points.sort_by_key(|point| point.observed_at_secs);
    observed_points.dedup_by_key(|point| point.observed_at_secs);

    if saturation.is_some() && tier.utilization >= 100.0 {
        let mut estimate = estimate_tier_with_saturation(tier, now_secs, saturation);
        estimate.window_start_secs = Some(window_start_secs);
        estimate.window_end_secs = Some(reset_at);
        estimate.observed_span_secs = observation_span(&observed_points);
        estimate.observed_points = observed_points;
        return estimate;
    }

    let selected = select_stable_short_trend_points(&observed_points, now_secs)
        .map(|points| (24, points))
        .or_else(|| select_recent_points(&observed_points, now_secs, 24).map(|points| (24, points)))
        .or_else(|| {
            select_recent_points(&observed_points, now_secs, 48).map(|points| (48, points))
        });
    let Some((trend_window_hours, trend_points)) = selected else {
        return QuotaEstimate {
            reset_in_secs: Some(reset_in_secs),
            observed_span_secs: observation_span(&observed_points),
            window_start_secs: Some(window_start_secs),
            window_end_secs: Some(reset_at),
            observed_points,
            ..unknown_estimate()
        };
    };
    let Some(raw_slope) = weighted_slope_pct_per_hour(&trend_points) else {
        return QuotaEstimate {
            reset_in_secs: Some(reset_in_secs),
            observed_span_secs: observation_span(&observed_points),
            window_start_secs: Some(window_start_secs),
            window_end_secs: Some(reset_at),
            observed_points,
            ..unknown_estimate()
        };
    };

    let slope_pct_per_hour = raw_slope.max(0.0);
    let utilization = tier.utilization.clamp(0.0, 100.0);
    let remaining_hours = reset_in_secs as f64 / 3_600.0;
    let projected_utilization = utilization + slope_pct_per_hour * remaining_hours;
    let lasts_for_secs = if slope_pct_per_hour > 0.0 {
        Some((((100.0 - utilization).max(0.0) / slope_pct_per_hour) * 3_600.0).round() as i64)
    } else {
        None
    };
    let state = projected_state(projected_utilization);
    let projected_end = if projected_utilization > 100.0 {
        UsageChartPoint {
            observed_at_secs: now_secs + lasts_for_secs.unwrap_or(0),
            utilization: 100.0,
        }
    } else {
        UsageChartPoint {
            observed_at_secs: reset_at,
            utilization: projected_utilization,
        }
    };

    QuotaEstimate {
        state,
        projected_utilization: Some(round_to(projected_utilization, 1)),
        reset_in_secs: Some(reset_in_secs),
        lasts_for_secs,
        exhausted_at_secs: None,
        exhausted_before_reset_secs: None,
        slope_pct_per_hour: Some(round_to(slope_pct_per_hour, 2)),
        trend_window_hours: Some(trend_window_hours),
        observed_span_secs: observation_span(&trend_points),
        window_start_secs: Some(window_start_secs),
        window_end_secs: Some(reset_at),
        observed_points,
        projected_points: vec![
            UsageChartPoint {
                observed_at_secs: now_secs,
                utilization,
            },
            projected_end,
        ],
    }
}

fn select_stable_short_trend_points(
    points: &[UsageChartPoint],
    now_secs: i64,
) -> Option<Vec<UsageChartPoint>> {
    let cutoff = now_secs - STABLE_SHORT_WINDOW_SECS;
    let selected = points
        .iter()
        .filter(|point| point.observed_at_secs >= cutoff)
        .cloned()
        .collect::<Vec<_>>();
    if selected.len() < STABLE_SHORT_MIN_SAMPLES {
        return None;
    }

    let span_secs = observation_span(&selected)?;
    if span_secs < STABLE_SHORT_MIN_SPAN_SECS {
        return None;
    }

    let newest = selected.last()?;
    let age_secs = now_secs.saturating_sub(newest.observed_at_secs);
    if !(0..=STABLE_SHORT_MAX_AGE_SECS).contains(&age_secs) {
        return None;
    }

    if selected.windows(2).any(|pair| {
        pair[1]
            .observed_at_secs
            .saturating_sub(pair[0].observed_at_secs)
            > STABLE_SHORT_MAX_SAMPLE_GAP_SECS
    }) {
        return None;
    }

    let total_delta = newest.utilization - selected.first()?.utilization;
    let fit = weighted_trend_fit(&selected)?;
    (total_delta >= STABLE_SHORT_MIN_DELTA_PCT
        && fit.slope_pct_per_hour >= STABLE_SHORT_MIN_SLOPE_PCT_PER_HOUR
        && fit.r_squared >= STABLE_SHORT_MIN_R_SQUARED)
        .then_some(selected)
}

fn select_recent_points(
    points: &[UsageChartPoint],
    now_secs: i64,
    hours: i64,
) -> Option<Vec<UsageChartPoint>> {
    let cutoff = now_secs - hours * 3_600;
    let selected = points
        .iter()
        .filter(|point| point.observed_at_secs >= cutoff)
        .cloned()
        .collect::<Vec<_>>();
    if selected.len() < MIN_TREND_SAMPLES
        || observation_span(&selected).unwrap_or(0) < MIN_TREND_SPAN_SECS
    {
        return None;
    }
    Some(selected)
}

fn weighted_slope_pct_per_hour(points: &[UsageChartPoint]) -> Option<f64> {
    weighted_trend_fit(points).map(|fit| fit.slope_pct_per_hour)
}

struct WeightedTrendFit {
    slope_pct_per_hour: f64,
    r_squared: f64,
}

fn weighted_trend_fit(points: &[UsageChartPoint]) -> Option<WeightedTrendFit> {
    let newest_at = points.last()?.observed_at_secs;
    let weighted = points
        .iter()
        .map(|point| {
            let x = (point.observed_at_secs - newest_at) as f64 / 3_600.0;
            let age_hours = -x;
            let weight = 0.5_f64.powf(age_hours / TREND_HALF_LIFE_HOURS);
            (x, point.utilization, weight)
        })
        .collect::<Vec<_>>();
    let weight_sum = weighted.iter().map(|(_, _, weight)| weight).sum::<f64>();
    if weight_sum <= 0.0 || !weight_sum.is_finite() {
        return None;
    }
    let mean_x = weighted
        .iter()
        .map(|(x, _, weight)| x * weight)
        .sum::<f64>()
        / weight_sum;
    let mean_y = weighted
        .iter()
        .map(|(_, y, weight)| y * weight)
        .sum::<f64>()
        / weight_sum;
    let numerator = weighted
        .iter()
        .map(|(x, y, weight)| weight * (x - mean_x) * (y - mean_y))
        .sum::<f64>();
    let denominator = weighted
        .iter()
        .map(|(x, _, weight)| weight * (x - mean_x).powi(2))
        .sum::<f64>();
    if denominator <= f64::EPSILON {
        return None;
    }
    let slope = numerator / denominator;
    let intercept = mean_y - slope * mean_x;
    let residual_sum = weighted
        .iter()
        .map(|(x, y, weight)| {
            let predicted = intercept + slope * x;
            weight * (y - predicted).powi(2)
        })
        .sum::<f64>();
    let total_sum = weighted
        .iter()
        .map(|(_, y, weight)| weight * (y - mean_y).powi(2))
        .sum::<f64>();
    let r_squared = if total_sum <= f64::EPSILON {
        1.0
    } else {
        (1.0 - residual_sum / total_sum).clamp(0.0, 1.0)
    };
    (slope.is_finite() && r_squared.is_finite()).then_some(WeightedTrendFit {
        slope_pct_per_hour: slope,
        r_squared,
    })
}

fn observation_span(points: &[UsageChartPoint]) -> Option<i64> {
    Some(
        points
            .last()?
            .observed_at_secs
            .saturating_sub(points.first()?.observed_at_secs),
    )
}

fn projected_state(projected_utilization: f64) -> SufficiencyState {
    if projected_utilization < TIGHT_PROJECTED_PCT {
        SufficiencyState::Enough
    } else if projected_utilization <= EXHAUSTED_PROJECTED_PCT {
        SufficiencyState::Tight
    } else {
        SufficiencyState::NotEnough
    }
}

fn round_to(value: f64, decimals: i32) -> f64 {
    let factor = 10_f64.powi(decimals);
    (value * factor).round() / factor
}

pub fn record_weekly_saturation_events(
    quota: &ServiceQuota,
    events: &mut Vec<QuotaSaturationEvent>,
    now_secs: i64,
) {
    record_weekly_saturation_events_for(&quota.service, quota, events, now_secs);
}

pub fn record_weekly_saturation_events_for(
    service_key: &str,
    quota: &ServiceQuota,
    events: &mut Vec<QuotaSaturationEvent>,
    now_secs: i64,
) {
    if !quota.success {
        return;
    }

    for tier in quota.tiers.iter().filter(|tier| is_weekly_tier(&tier.name)) {
        let Some(reset_at) = tier.resets_at.as_ref() else {
            continue;
        };
        if tier.utilization < 100.0 {
            continue;
        }
        let exists = events.iter().any(|event| {
            event.service == service_key && event.tier == tier.name && event.reset_at == *reset_at
        });
        if !exists {
            events.push(QuotaSaturationEvent {
                service: service_key.to_string(),
                tier: tier.name.clone(),
                reset_at: reset_at.clone(),
                reached_at_secs: now_secs,
                utilization_at: tier.utilization,
            });
        }
    }

    if events.len() > 20 {
        let drop_count = events.len() - 20;
        events.drain(0..drop_count);
    }
}

pub fn matching_saturation_event<'a>(
    tier: &QuotaTier,
    service: &str,
    events: &'a [QuotaSaturationEvent],
) -> Option<&'a QuotaSaturationEvent> {
    let reset_at = tier.resets_at.as_ref()?;
    events.iter().find(|event| {
        event.service == service && event.tier == tier.name && event.reset_at == *reset_at
    })
}

pub fn overall_state(tiers: &[QuotaTier], now_secs: i64) -> SufficiencyState {
    tiers
        .iter()
        .map(|tier| estimate_tier(tier, now_secs).state)
        .filter(|state| *state != SufficiencyState::Unknown)
        .max_by_key(|state| state_rank(*state))
        .unwrap_or(SufficiencyState::Unknown)
}

pub fn overall_state_for_services<'a>(
    quotas: impl IntoIterator<Item = &'a Option<crate::types::ServiceQuota>>,
    now_secs: i64,
) -> SufficiencyState {
    let tiers = quotas
        .into_iter()
        .filter_map(|quota| quota.as_ref())
        .filter(|quota| quota.success)
        .flat_map(|quota| quota.tiers.iter())
        .cloned()
        .collect::<Vec<_>>();

    overall_state(&tiers, now_secs)
}

fn unknown_estimate() -> QuotaEstimate {
    QuotaEstimate {
        state: SufficiencyState::Unknown,
        projected_utilization: None,
        reset_in_secs: None,
        lasts_for_secs: None,
        exhausted_at_secs: None,
        exhausted_before_reset_secs: None,
        ..empty_trend_fields()
    }
}

fn empty_trend_fields() -> QuotaEstimate {
    QuotaEstimate {
        state: SufficiencyState::Unknown,
        projected_utilization: None,
        reset_in_secs: None,
        lasts_for_secs: None,
        exhausted_at_secs: None,
        exhausted_before_reset_secs: None,
        slope_pct_per_hour: None,
        trend_window_hours: None,
        observed_span_secs: None,
        window_start_secs: None,
        window_end_secs: None,
        observed_points: Vec::new(),
        projected_points: Vec::new(),
    }
}

fn window_seconds(name: &str) -> Option<i64> {
    match name {
        "five_hour" => Some(FIVE_HOUR_WINDOW_SECS),
        "weekly_limit" | "seven_day" => Some(SEVEN_DAY_WINDOW_SECS),
        _ => None,
    }
}

fn is_weekly_tier(name: &str) -> bool {
    matches!(name, "weekly_limit" | "seven_day")
}

fn state_rank(state: SufficiencyState) -> u8 {
    match state {
        SufficiencyState::Enough => 1,
        SufficiencyState::Tight => 2,
        SufficiencyState::NotEnough => 3,
        SufficiencyState::Unknown => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{QuotaTier, ServiceQuota, SufficiencyState, UsageSample};

    fn iso_after(now: i64, seconds: i64) -> String {
        chrono::DateTime::from_timestamp(now + seconds, 0)
            .unwrap()
            .to_rfc3339()
    }

    fn tier(name: &str, utilization: f64, reset_in: Option<i64>, now: i64) -> QuotaTier {
        QuotaTier {
            name: name.to_string(),
            utilization,
            resets_at: reset_in.map(|seconds| iso_after(now, seconds)),
            used: None,
            limit: None,
            remaining: None,
        }
    }

    fn sample(
        service: &str,
        tier: &QuotaTier,
        observed_at_secs: i64,
        utilization: f64,
    ) -> UsageSample {
        UsageSample {
            service: service.to_string(),
            tier: tier.name.clone(),
            reset_at: tier.resets_at.clone().unwrap(),
            observed_at_secs,
            utilization,
        }
    }

    #[test]
    fn recent_spike_drives_weekly_forecast_instead_of_old_cycle_average() {
        let now = 1_700_000_000;
        let weekly = tier("seven_day", 90.0, Some(86_400), now);
        let samples = vec![
            sample("codex", &weekly, now - 23 * 3_600, 20.0),
            sample("codex", &weekly, now - 12 * 3_600, 20.0),
            sample("codex", &weekly, now - 4 * 3_600, 25.0),
            sample("codex", &weekly, now - 2 * 3_600, 60.0),
            sample("codex", &weekly, now, 90.0),
        ];

        let estimate = estimate_recent_weekly_trend(&weekly, now, &samples, None);

        assert_eq!(estimate.trend_window_hours, Some(24));
        assert!(estimate.slope_pct_per_hour.unwrap() > 5.0);
        assert_eq!(estimate.state, SufficiencyState::NotEnough);
        assert!(estimate.projected_utilization.unwrap() > 100.0);
    }

    #[test]
    fn recent_forecast_ignores_old_burst_outside_the_selected_window() {
        let now = 1_700_000_000;
        let weekly = tier("weekly_limit", 62.0, Some(86_400), now);
        let samples = vec![
            sample("kimi", &weekly, now - 5 * 86_400, 0.0),
            sample("kimi", &weekly, now - 4 * 86_400, 60.0),
            sample("kimi", &weekly, now - 20 * 3_600, 60.0),
            sample("kimi", &weekly, now - 10 * 3_600, 61.0),
            sample("kimi", &weekly, now, 62.0),
        ];

        let estimate = estimate_recent_weekly_trend(&weekly, now, &samples, None);

        assert_eq!(estimate.trend_window_hours, Some(24));
        assert!(estimate.slope_pct_per_hour.unwrap() < 0.2);
        assert_eq!(estimate.state, SufficiencyState::Enough);
    }

    #[test]
    fn recent_forecast_falls_back_to_forty_eight_hours() {
        let now = 1_700_000_000;
        let weekly = tier("seven_day", 35.0, Some(172_800), now);
        let samples = vec![
            sample("codex", &weekly, now - 36 * 3_600, 20.0),
            sample("codex", &weekly, now - 30 * 3_600, 23.0),
            sample("codex", &weekly, now, 35.0),
        ];

        let estimate = estimate_recent_weekly_trend(&weekly, now, &samples, None);

        assert_eq!(estimate.trend_window_hours, Some(48));
        assert!(estimate.slope_pct_per_hour.unwrap() > 0.0);
    }

    #[test]
    fn recent_forecast_waits_when_short_samples_do_not_show_a_meaningful_spike() {
        let now = 1_700_000_000;
        let weekly = tier("seven_day", 35.0, Some(172_800), now);
        let samples = vec![
            sample("codex", &weekly, now - 600, 34.0),
            sample("codex", &weekly, now - 300, 34.5),
            sample("codex", &weekly, now, 35.0),
        ];

        let estimate = estimate_recent_weekly_trend(&weekly, now, &samples, None);

        assert_eq!(estimate.state, SufficiencyState::Unknown);
        assert_eq!(estimate.slope_pct_per_hour, None);
        assert_eq!(estimate.observed_points.len(), 3);
    }

    #[test]
    fn stable_short_forecast_rejects_a_two_percent_jump_over_five_minutes() {
        let now = 1_700_000_000;
        let weekly = tier("seven_day", 28.0, Some(172_800), now);
        let samples = vec![
            sample("codex", &weekly, now - 300, 26.0),
            sample("codex", &weekly, now, 28.0),
        ];

        let estimate = estimate_recent_weekly_trend(&weekly, now, &samples, None);

        assert_eq!(estimate.slope_pct_per_hour, None);
        assert_eq!(estimate.state, SufficiencyState::Unknown);
    }

    #[test]
    fn stable_short_forecast_rejects_a_flat_follow_up_with_too_few_samples() {
        let now = 1_700_000_000;
        let weekly = tier("seven_day", 28.0, Some(172_800), now);
        let samples = vec![
            sample("codex", &weekly, now - 600, 26.0),
            sample("codex", &weekly, now - 300, 28.0),
            sample("codex", &weekly, now, 28.0),
        ];

        let estimate = estimate_recent_weekly_trend(&weekly, now, &samples, None);

        assert_eq!(estimate.slope_pct_per_hour, None);
        assert_eq!(estimate.state, SufficiencyState::Unknown);
    }

    #[test]
    fn stable_short_forecast_rejects_a_single_one_percent_quantized_step() {
        let now = 1_700_000_000;
        let weekly = tier("seven_day", 50.0, Some(172_800), now);
        let samples = vec![
            sample("codex", &weekly, now - 600, 49.0),
            sample("codex", &weekly, now - 300, 49.0),
            sample("codex", &weekly, now, 50.0),
        ];

        let estimate = estimate_recent_weekly_trend(&weekly, now, &samples, None);

        assert_eq!(estimate.slope_pct_per_hour, None);
        assert_eq!(estimate.state, SufficiencyState::Unknown);
    }

    #[test]
    fn stable_short_forecast_rejects_only_three_samples_over_ten_minutes() {
        let now = 1_700_000_000;
        let weekly = tier("seven_day", 50.0, Some(172_800), now);
        let samples = vec![
            sample("codex", &weekly, now - 600, 48.0),
            sample("codex", &weekly, now - 300, 49.0),
            sample("codex", &weekly, now, 50.0),
        ];

        let estimate = estimate_recent_weekly_trend(&weekly, now, &samples, None);

        assert_eq!(estimate.slope_pct_per_hour, None);
        assert_eq!(estimate.state, SufficiencyState::Unknown);
    }

    #[test]
    fn stable_short_forecast_uses_five_linear_samples_over_twenty_minutes() {
        let now = 1_700_000_000;
        let weekly = tier("seven_day", 50.0, Some(172_800), now);
        let samples = vec![
            sample("codex", &weekly, now - 1_200, 46.0),
            sample("codex", &weekly, now - 900, 47.0),
            sample("codex", &weekly, now - 600, 48.0),
            sample("codex", &weekly, now - 300, 49.0),
            sample("codex", &weekly, now, 50.0),
        ];

        let estimate = estimate_recent_weekly_trend(&weekly, now, &samples, None);

        assert_eq!(estimate.slope_pct_per_hour, Some(12.0));
        assert_eq!(estimate.observed_span_secs, Some(1_200));
        assert_ne!(estimate.state, SufficiencyState::Unknown);
    }

    #[test]
    fn stable_short_forecast_rejects_a_last_sample_only_burst() {
        let now = 1_700_000_000;
        let weekly = tier("seven_day", 50.0, Some(172_800), now);
        let samples = vec![
            sample("codex", &weekly, now - 1_200, 46.0),
            sample("codex", &weekly, now - 900, 46.0),
            sample("codex", &weekly, now - 600, 46.0),
            sample("codex", &weekly, now - 300, 46.0),
            sample("codex", &weekly, now, 50.0),
        ];

        let estimate = estimate_recent_weekly_trend(&weekly, now, &samples, None);

        assert_eq!(estimate.slope_pct_per_hour, None);
        assert_eq!(estimate.state, SufficiencyState::Unknown);
    }

    #[test]
    fn stable_short_forecast_rejects_refreshes_that_are_too_close_together() {
        let now = 1_700_000_000;
        let weekly = tier("seven_day", 28.0, Some(172_800), now);
        let samples = vec![
            sample("codex", &weekly, now - 60, 26.0),
            sample("codex", &weekly, now, 28.0),
        ];

        let estimate = estimate_recent_weekly_trend(&weekly, now, &samples, None);

        assert_eq!(estimate.state, SufficiencyState::Unknown);
        assert_eq!(estimate.slope_pct_per_hour, None);
    }

    #[test]
    fn stable_short_forecast_rejects_a_two_minute_rounding_jump() {
        let now = 1_700_000_000;
        let weekly = tier("seven_day", 28.0, Some(172_800), now);
        let samples = vec![
            sample("codex", &weekly, now - 120, 27.0),
            sample("codex", &weekly, now, 28.0),
        ];

        let estimate = estimate_recent_weekly_trend(&weekly, now, &samples, None);

        assert_eq!(estimate.state, SufficiencyState::Unknown);
        assert_eq!(estimate.slope_pct_per_hour, None);
    }

    #[test]
    fn recent_flat_usage_has_zero_slope_and_no_exhaustion_time() {
        let now = 1_700_000_000;
        let weekly = tier("seven_day", 40.0, Some(172_800), now);
        let samples = vec![
            sample("codex", &weekly, now - 2 * 3_600, 40.0),
            sample("codex", &weekly, now - 3_600, 40.0),
            sample("codex", &weekly, now, 40.0),
        ];

        let estimate = estimate_recent_weekly_trend(&weekly, now, &samples, None);

        assert_eq!(estimate.slope_pct_per_hour, Some(0.0));
        assert_eq!(estimate.projected_utilization, Some(40.0));
        assert_eq!(estimate.lasts_for_secs, None);
        assert_eq!(estimate.state, SufficiencyState::Enough);
    }

    #[test]
    fn five_hour_projection_under_eighty_five_percent_is_enough() {
        let now = 1_700_000_000;
        let estimate = estimate_tier(&tier("five_hour", 20.0, Some(10_000), now), now);

        assert_eq!(estimate.state, SufficiencyState::Enough);
        assert!(estimate.projected_utilization.unwrap() < 85.0);
    }

    #[test]
    fn five_hour_projection_between_eighty_five_and_one_hundred_percent_is_tight() {
        let now = 1_700_000_000;
        let estimate = estimate_tier(&tier("five_hour", 40.0, Some(10_000), now), now);

        assert_eq!(estimate.state, SufficiencyState::Tight);
        let projected = estimate.projected_utilization.unwrap();
        assert!((85.0..=100.0).contains(&projected));
    }

    #[test]
    fn five_hour_projection_over_one_hundred_percent_is_not_enough() {
        let now = 1_700_000_000;
        let estimate = estimate_tier(&tier("five_hour", 60.0, Some(10_000), now), now);

        assert_eq!(estimate.state, SufficiencyState::NotEnough);
        assert!(estimate.projected_utilization.unwrap() > 100.0);
    }

    #[test]
    fn seven_day_window_uses_weekly_seconds() {
        let now = 1_700_000_000;
        let estimate = estimate_tier(&tier("seven_day", 50.0, Some(302_400), now), now);

        assert_eq!(estimate.state, SufficiencyState::Tight);
        assert_eq!(estimate.reset_in_secs, Some(302_400));
        assert_eq!(estimate.projected_utilization, Some(100.0));
    }

    #[test]
    fn missing_reset_time_returns_unknown() {
        let now = 1_700_000_000;
        let estimate = estimate_tier(&tier("five_hour", 30.0, None, now), now);

        assert_eq!(estimate.state, SufficiencyState::Unknown);
        assert_eq!(estimate.projected_utilization, None);
    }

    #[test]
    fn very_early_window_with_nonzero_usage_returns_unknown() {
        let now = 1_700_000_000;
        let estimate = estimate_tier(&tier("five_hour", 1.0, Some(17_500), now), now);

        assert_eq!(estimate.state, SufficiencyState::Unknown);
    }

    #[test]
    fn zero_usage_returns_enough_even_early_in_window() {
        let now = 1_700_000_000;
        let estimate = estimate_tier(&tier("five_hour", 0.0, Some(17_900), now), now);

        assert_eq!(estimate.state, SufficiencyState::Enough);
        assert_eq!(estimate.projected_utilization, Some(0.0));
    }

    #[test]
    fn overall_state_chooses_worst_known_tier() {
        let now = 1_700_000_000;
        let tiers = vec![
            tier("five_hour", 20.0, Some(10_000), now),
            tier("seven_day", 80.0, Some(302_400), now),
        ];

        assert_eq!(overall_state(&tiers, now), SufficiencyState::NotEnough);
    }

    #[test]
    fn weekly_saturation_event_freezes_projection() {
        let now = 1_700_000_000;
        let reset_in = 302_400;
        let tier = tier("seven_day", 100.0, Some(reset_in), now);
        let reset_at = tier.resets_at.clone().unwrap();
        let reset_ts = chrono::DateTime::parse_from_rfc3339(&reset_at)
            .unwrap()
            .timestamp();
        let event = QuotaSaturationEvent {
            service: "codex".to_string(),
            tier: "seven_day".to_string(),
            reset_at,
            reached_at_secs: reset_ts - 100_000,
            utilization_at: 100.0,
        };

        let estimate = estimate_tier_with_saturation(&tier, now, Some(&event));

        assert_eq!(estimate.state, SufficiencyState::NotEnough);
        assert_eq!(estimate.exhausted_at_secs, Some(event.reached_at_secs));
        assert_eq!(estimate.exhausted_before_reset_secs, Some(100_000));
        assert_eq!(estimate.projected_utilization, Some(120.0));
    }

    #[test]
    fn records_weekly_saturation_once_per_reset() {
        let now = 1_700_000_000;
        let quota = ServiceQuota {
            service: "kimi".to_string(),
            display_name: "Kimi Code".to_string(),
            success: true,
            tiers: vec![
                tier("five_hour", 100.0, Some(10_000), now),
                tier("weekly_limit", 100.0, Some(302_400), now),
            ],
            error: None,
            queried_at: None,
            credential_valid: true,
        };
        let mut events = Vec::new();

        record_weekly_saturation_events(&quota, &mut events, now);
        record_weekly_saturation_events(&quota, &mut events, now + 60);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].tier, "weekly_limit");
    }

    #[test]
    fn records_saturation_independently_for_accounts_on_the_same_service() {
        let now = 1_700_000_000;
        let quota = ServiceQuota {
            service: "codex".to_string(),
            display_name: "Codex".to_string(),
            success: true,
            tiers: vec![tier("seven_day", 100.0, Some(302_400), now)],
            error: None,
            queried_at: None,
            credential_valid: true,
        };
        let mut events = Vec::new();

        record_weekly_saturation_events_for("account-one", &quota, &mut events, now);
        record_weekly_saturation_events_for("account-two", &quota, &mut events, now + 60);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].service, "account-one");
        assert_eq!(events[1].service, "account-two");
    }
}
