use crate::types::{
    QuotaEstimate, QuotaSaturationEvent, QuotaTier, ServiceQuota, SufficiencyState,
};
use std::time::{SystemTime, UNIX_EPOCH};

const FIVE_HOUR_WINDOW_SECS: i64 = 18_000;
const SEVEN_DAY_WINDOW_SECS: i64 = 604_800;
const MIN_ELAPSED_SECS_FOR_PROJECTION: i64 = 900;
const TIGHT_PROJECTED_PCT: f64 = 85.0;
const EXHAUSTED_PROJECTED_PCT: f64 = 100.0;

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
    }
}

pub fn record_weekly_saturation_events(
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
            event.service == quota.service && event.tier == tier.name && event.reset_at == *reset_at
        });
        if !exists {
            events.push(QuotaSaturationEvent {
                service: quota.service.clone(),
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
    use crate::types::{QuotaTier, ServiceQuota, SufficiencyState};

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
}
