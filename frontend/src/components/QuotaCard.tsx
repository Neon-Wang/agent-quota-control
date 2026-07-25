import { AlertTriangle, CheckCircle2, Clock, Network } from "lucide-react";
import type {
  CardSnapshot,
  SufficiencyState,
} from "../types";
import { proxyDetailLabel } from "../proxyDisplay";
import { UsageTrendChart } from "./UsageTrendChart";

interface QuotaCardProps {
  card: CardSnapshot;
  iconSrc: string;
}

const tierLabels: Record<string, string> = {
  five_hour: "5 小时",
  weekly_limit: "7 天",
  seven_day: "7 天",
};

export function QuotaCard({ card, iconSrc }: QuotaCardProps) {
  const displayState = stateLabel(card.weeklyEstimate?.state ?? "unknown");
  const hasData = card.tiers.length > 0;
  const isHealthy = card.status === "fresh" || card.status === "stale";
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
      aria-label={`${card.accountDisplayName} 配额`}
    >
      <div className="quota-header">
        <div className="service-heading">
          <img src={iconSrc} alt="" aria-hidden />
          <div>
            <p className="eyebrow">{card.serviceDisplayName}</p>
            <h3
              className={`quota-state-heading quota-state-${card.weeklyEstimate?.state ?? "unknown"}`}
            >
              {displayState}
            </h3>
            {showAccountName ? (
              <p className="account-card-name">{card.accountDisplayName}</p>
            ) : null}
          </div>
        </div>
        {isHealthy ? (
          <CheckCircle2 size={18} aria-hidden className="ok" />
        ) : (
          <AlertTriangle size={18} aria-hidden className="warn" />
        )}
      </div>

      {card.errorMessage && <p className="error-copy card-status-copy">{card.errorMessage}</p>}
      {!hasData && !card.errorMessage && <p className="muted quota-empty">等待后台首次刷新用量。</p>}
      {hasData && (
        <div className="tier-stack">
          {orderedTiers.map((tier) => (
            <div className="tier-row" key={tier.name}>
              <div className="tier-meta">
                <span className="tier-label">
                  <span>{tierLabels[tier.name] ?? tier.name}</span>
                  {tier.resetsAt && (
                    <small>（{formatResetLabel(tier.name, tier.resetsAt)}）</small>
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
            <div className="tier-unavailable" aria-label="当前无 5 小时限制">
              <span>5 小时</span>
              <strong>当前无 5 小时限制</strong>
            </div>
          ) : null}
        </div>
      )}

      {hasData && card.weeklyEstimate && (
        <div className="estimate-box">
          <div>
            <span>预计用量</span>
            <strong>
              {card.weeklyEstimate.projectedUtilization == null
                ? "积累中"
                : `${Math.round(card.weeklyEstimate.projectedUtilization)}%`}
            </strong>
          </div>
          {card.weeklyEstimate.exhaustedBeforeResetSecs != null && (
            <p>
              已提前 {formatDuration(card.weeklyEstimate.exhaustedBeforeResetSecs)} 耗尽。
            </p>
          )}
          {card.weeklyEstimate.exhaustedBeforeResetSecs == null &&
            card.weeklyEstimate.state !== "unknown" && (
            <p>{estimateHint(card.weeklyEstimate.state, card.weeklyEstimate.lastsForSecs)}</p>
          )}
        </div>
      )}
      {hasData && card.weeklyEstimate && (
        <UsageTrendChart estimate={card.weeklyEstimate} />
      )}

      <div className="proxy-line">
        <Network size={13} aria-hidden />
        <span>{proxyDetailLabel(card.proxy)}</span>
      </div>
      {card.queriedAt && (
        <div className="proxy-line">
          <Clock size={13} aria-hidden />
          <span>更新于 {new Date(card.queriedAt).toLocaleTimeString()}</span>
        </div>
      )}
    </section>
  );
}

function meterClass(utilization: number): string {
  if (utilization >= 90) return "meter-fill danger";
  if (utilization >= 70) return "meter-fill warn";
  return "meter-fill ok-fill";
}

function stateLabel(state: SufficiencyState): string {
  if (state === "enough") return "够";
  if (state === "tight") return "偏紧";
  if (state === "not_enough") return "不够";
  return "等待数据";
}

function normalizeIdentity(value: string): string {
  return value.trim().toLocaleLowerCase();
}

function estimateHint(state: SufficiencyState, lastsForSecs?: number | null): string {
  if (state === "not_enough" && lastsForSecs != null) {
    return `预计将在 ${formatDuration(lastsForSecs)} 后耗尽。`;
  }
  if (state === "tight") {
    return "本周内预计不会耗尽，但余量偏紧。";
  }
  if (state === "enough") {
    return "本周内预计够用。";
  }
  return "等待更多用量数据后估算。";
}

function formatDuration(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  if (days > 0 && hours > 0) return `${days} 天 ${hours} 小时`;
  if (days > 0) return `${days} 天`;
  return `${hours} 小时`;
}

function formatResetTime(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "未知时间";
  const month = `${date.getMonth() + 1}`.padStart(2, "0");
  const day = `${date.getDate()}`.padStart(2, "0");
  const hour = `${date.getHours()}`.padStart(2, "0");
  const minute = `${date.getMinutes()}`.padStart(2, "0");
  return `${month}月${day}日 ${hour}:${minute}`;
}

function formatResetLabel(tierName: string, value: string): string {
  if (tierName === "five_hour") {
    return `${formatResetCountdown(value)}后重置`;
  }
  return `${formatResetTime(value)} 重置`;
}

function formatResetCountdown(value: string): string {
  const resetAt = new Date(value).getTime();
  if (Number.isNaN(resetAt)) return "未知时间";
  const seconds = Math.max(0, Math.ceil((resetAt - Date.now()) / 1000));
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.ceil((seconds % 3600) / 60);
  if (hours > 0 && minutes > 0) return `${hours} 小时 ${minutes} 分钟`;
  if (hours > 0) return `${hours} 小时`;
  return `${minutes} 分钟`;
}
