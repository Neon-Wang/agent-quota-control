import { AlertTriangle, CheckCircle2, Clock, Network } from "lucide-react";
import { useFormatter, useTranslations } from "../i18n";
import type { Translator } from "../i18n/translate";
import { proxyDetailLabel } from "../proxyDisplay";
import type { CardSnapshot, SufficiencyState } from "../types";
import { UsageTrendChart } from "./UsageTrendChart";

interface QuotaCardProps {
  card: CardSnapshot;
  iconSrc: string;
}

export function QuotaCard({ card, iconSrc }: QuotaCardProps) {
  const t = useTranslations("dashboard");
  const format = useFormatter();
  const estimateState = card.weeklyEstimate?.state ?? "unknown";
  const displayState = stateLabel(estimateState, t);
  const hasData = card.tiers.length > 0;
  const isHealthy = card.status === "fresh" || card.status === "stale";
  const needsLogin = card.status === "login_expired";
  const statusTone = needsLogin ? "problem" : estimateState;
  const statusText = needsLogin ? t("needs_login") : displayState;
  const weeklyTier = card.tiers.find((tier) =>
    ["weekly_limit", "seven_day"].includes(tier.name),
  );
  const fiveHourTier = card.tiers.find((tier) => tier.name === "five_hour");
  const remainingTiers = card.tiers.filter(
    (tier) =>
      tier.name !== "weekly_limit" &&
      tier.name !== "seven_day" &&
      tier.name !== "five_hour",
  );
  const orderedTiers = [weeklyTier, fiveHourTier, ...remainingTiers].filter(
    (tier): tier is CardSnapshot["tiers"][number] => tier !== undefined,
  );
  const showAccountName =
    normalizeIdentity(card.accountDisplayName) !==
    normalizeIdentity(card.serviceDisplayName);

  return (
    <section
      className="quota-card"
      aria-label={t("quota_aria", { name: card.accountDisplayName })}
    >
      <div className="quota-header">
        <div className="service-heading">
          <img src={iconSrc} alt="" aria-hidden />
          <div
            className={
              showAccountName
                ? "service-copy"
                : "service-copy service-copy-single"
            }
          >
            {showAccountName ? (
              <>
                <p className="eyebrow">{card.serviceDisplayName}</p>
                <h3>{card.accountDisplayName}</h3>
              </>
            ) : (
              <h3>{card.serviceDisplayName}</h3>
            )}
          </div>
        </div>
        <div className={`status-badge ${statusTone}`}>
          {isHealthy ? (
            <CheckCircle2 size={13} strokeWidth={1.75} aria-hidden />
          ) : (
            <AlertTriangle size={13} strokeWidth={1.75} aria-hidden />
          )}
          {statusText}
        </div>
      </div>

      {card.errorMessage && (
        <div className="card-alert" role="status">
          <span>
            {needsLogin ? t("login_expired") : card.errorMessage}
          </span>
        </div>
      )}
      {!hasData && !card.errorMessage && (
        <p className="muted quota-empty">{t("waiting_first_refresh")}</p>
      )}
      {hasData && (
        <div className="tier-stack">
          {orderedTiers.map((tier) => (
            <div className="tier-row" key={tier.name}>
              <div className="tier-meta">
                <span className="tier-label">
                  <span>{tierLabel(tier.name, t)}</span>
                  {tier.resetsAt && (
                    <small>
                      （{formatResetLabel(tier.name, tier.resetsAt, t)}）
                    </small>
                  )}
                </span>
                <strong>{Math.round(tier.utilization)}%</strong>
              </div>
              <div className="meter" aria-label={`${tier.name} utilization`}>
                <div
                  className={meterClass(tier.utilization)}
                  style={{ width: `${Math.min(tier.utilization, 100)}%` }}
                />
              </div>
            </div>
          ))}
          {weeklyTier && !fiveHourTier ? (
            <div className="tier-unavailable" aria-label={t("no_5h_limit")}>
              <span>{t("tier_5h")}</span>
              <strong>{t("no_5h_limit")}</strong>
            </div>
          ) : null}
        </div>
      )}

      {hasData && card.weeklyEstimate && (
        <div className="estimate-box">
          <div>
            <span>{t("projected_usage")}</span>
            <strong>
              {card.weeklyEstimate.projectedUtilization == null
                ? t("accumulating")
                : `${Math.round(card.weeklyEstimate.projectedUtilization)}%`}
            </strong>
          </div>
          {card.weeklyEstimate.exhaustedBeforeResetSecs != null && (
            <p>
              {t("exhausted_early", {
                duration: formatDuration(
                  card.weeklyEstimate.exhaustedBeforeResetSecs,
                  t,
                ),
              })}
            </p>
          )}
          {card.weeklyEstimate.exhaustedBeforeResetSecs == null &&
            card.weeklyEstimate.state !== "unknown" && (
              <p>
                {estimateHint(
                  card.weeklyEstimate.state,
                  t,
                  card.weeklyEstimate.lastsForSecs,
                )}
              </p>
            )}
        </div>
      )}
      {hasData && card.weeklyEstimate && (
        <UsageTrendChart estimate={card.weeklyEstimate} />
      )}

      <div className="card-meta">
        <div className="proxy-line">
          <Network size={12} strokeWidth={1.75} aria-hidden />
          <span>{proxyDetailLabel(card.proxy, t)}</span>
        </div>
        {card.queriedAt && (
          <div className="proxy-line">
            <Clock size={12} strokeWidth={1.75} aria-hidden />
            <span>
              {t("updated_at", {
                time: format.dateTime(card.queriedAt, { timeStyle: "medium" }),
              })}
            </span>
          </div>
        )}
      </div>
    </section>
  );
}

function meterClass(utilization: number): string {
  if (utilization >= 90) return "meter-fill danger";
  if (utilization >= 70) return "meter-fill warn";
  return "meter-fill ok-fill";
}

function tierLabel(name: string, t: Translator<"dashboard">): string {
  if (name === "five_hour") return t("tier_5h");
  if (name === "weekly_limit" || name === "seven_day") return t("tier_7d");
  return name;
}

function stateLabel(state: SufficiencyState, t: Translator<"dashboard">): string {
  if (state === "enough") return t("state_enough");
  if (state === "tight") return t("state_tight");
  if (state === "not_enough") return t("state_not_enough");
  return t("state_waiting");
}

function normalizeIdentity(value: string): string {
  return value.trim().toLocaleLowerCase();
}

function estimateHint(
  state: SufficiencyState,
  t: Translator<"dashboard">,
  lastsForSecs?: number | null,
): string {
  if (state === "not_enough" && lastsForSecs != null) {
    return t("hint_exhaust_in", { duration: formatDuration(lastsForSecs, t) });
  }
  if (state === "tight") return t("hint_tight");
  if (state === "enough") return t("hint_enough");
  return t("hint_waiting");
}

function formatDuration(seconds: number, t: Translator<"dashboard">): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  if (days > 0 && hours > 0) {
    return t("duration_days_hours", { days, hours });
  }
  if (days > 0) return t("duration_days", { days });
  return t("duration_hours", { hours });
}

function formatResetTime(value: string, t: Translator<"dashboard">): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return t("unknown_time");
  const month = `${date.getMonth() + 1}`.padStart(2, "0");
  const day = `${date.getDate()}`.padStart(2, "0");
  const hour = `${date.getHours()}`.padStart(2, "0");
  const minute = `${date.getMinutes()}`.padStart(2, "0");
  return t("reset_datetime", { month, day, hour, minute });
}

function formatResetLabel(
  tierName: string,
  value: string,
  t: Translator<"dashboard">,
): string {
  if (tierName === "five_hour") {
    return t("reset_in", { duration: formatResetCountdown(value, t) });
  }
  return t("reset_at", { time: formatResetTime(value, t) });
}

function formatResetCountdown(
  value: string,
  t: Translator<"dashboard">,
): string {
  const resetAt = new Date(value).getTime();
  if (Number.isNaN(resetAt)) return t("unknown_time");
  const seconds = Math.max(0, Math.ceil((resetAt - Date.now()) / 1000));
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.ceil((seconds % 3600) / 60);
  if (hours > 0 && minutes > 0) {
    return t("duration_hours_minutes", { hours, minutes });
  }
  if (hours > 0) return t("duration_hours", { hours });
  return t("duration_minutes", { minutes });
}
