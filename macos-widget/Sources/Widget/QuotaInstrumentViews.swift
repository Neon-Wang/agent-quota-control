import SwiftUI
import WidgetKit

struct QuotaInstrumentView: View {
    let account: WidgetAccount
    let card: CardSnapshot
    let family: WidgetFamily

    var body: some View {
        VStack(alignment: .leading, spacing: family == .systemSmall ? 10 : 12) {
            InstrumentHeader(account: account, card: card)

            if WidgetPresentation.cardMode(for: card.status) == .actionRequired {
                CompactNoDataView(card: card)
            } else {
                switch family {
                case .systemSmall:
                    SmallQuotaInstrument(card: card)
                case .systemMedium:
                    MediumQuotaInstrument(card: card)
                default:
                    LargeQuotaInstrument(card: card)
                }
            }
        }
        .accessibilityElement(children: .combine)
        .accessibilityLabel(accessibilitySummary)
    }

    private var accessibilitySummary: String {
        var parts = [card.accountDisplayName]
        if let weekly = card.weeklyTier {
            parts.append("本周已用 \(percentage(weekly.utilization))")
        }
        if let estimate = card.weeklyEstimate {
            if let projected = estimate.projectedUtilization {
                parts.append("预计 \(percentage(projected))")
            }
            parts.append(WidgetPresentation.stateLabel(estimate.state))
        }
        return parts.joined(separator: "，")
    }
}

private struct InstrumentHeader: View {
    let account: WidgetAccount
    let card: CardSnapshot

    var body: some View {
        HStack(spacing: 9) {
            ZStack {
                RoundedRectangle(cornerRadius: 8, style: .continuous)
                    .fill(Color.primary.opacity(0.09))
                Image(systemName: account.service == "kimi" ? "sparkles" : "terminal")
                    .font(.system(size: 13, weight: .semibold))
                    .foregroundStyle(.primary)
            }
            .frame(width: 29, height: 29)

            VStack(alignment: .leading, spacing: 1) {
                Text(card.accountDisplayName)
                    .font(.subheadline.weight(.semibold))
                    .lineLimit(1)
                if card.accountDisplayName != card.serviceDisplayName {
                    Text(card.serviceDisplayName)
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
            }

            Spacer(minLength: 6)

            if let notice = WidgetPresentation.statusNotice(for: card.status) {
                StatusPill(text: notice.text, severity: notice.severity)
            }
        }
        .frame(maxWidth: .infinity)
    }
}

private struct SmallQuotaInstrument: View {
    let card: CardSnapshot

    var body: some View {
        if let weekly = card.weeklyTier {
            VStack(alignment: .leading, spacing: 8) {
                HStack(alignment: .firstTextBaseline) {
                    VStack(alignment: .leading, spacing: 1) {
                        Text("本周已用")
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                        Text(percentage(weekly.utilization))
                            .font(.system(size: 34, weight: .bold, design: .rounded))
                            .monospacedDigit()
                            .minimumScaleFactor(0.75)
                    }
                    Spacer(minLength: 4)
                }

                QuotaMeter(utilization: weekly.utilization)

                HStack(alignment: .center, spacing: 5) {
                    if let estimate = card.weeklyEstimate {
                        Text(projectionLabel(estimate))
                            .font(.caption2.weight(.medium))
                            .foregroundStyle(
                                Color.widgetSemantic(
                                    WidgetPresentation.projectionSeverity(
                                        estimate.projectedUtilization,
                                        state: estimate.state
                                    )
                                )
                            )
                            .lineLimit(1)
                    }
                    Spacer(minLength: 4)
                    FreshnessLabel(card: card, exact: false, showsIcon: false)
                }
            }
        } else {
            CompactNoDataView(card: card)
        }
    }
}

private struct MediumQuotaInstrument: View {
    let card: CardSnapshot

    var body: some View {
        if let weekly = card.weeklyTier {
            HStack(alignment: .top, spacing: 14) {
                VStack(alignment: .leading, spacing: 7) {
                    Text("本周已用")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    Text(percentage(weekly.utilization))
                        .font(.system(size: 36, weight: .bold, design: .rounded))
                        .monospacedDigit()
                        .minimumScaleFactor(0.75)
                    QuotaMeter(utilization: weekly.utilization)
                    ResetLabel(tier: weekly)
                }
                .frame(maxWidth: .infinity, alignment: .leading)

                Rectangle()
                    .fill(Color.primary.opacity(0.10))
                    .frame(width: 1)

                VStack(alignment: .leading, spacing: 5) {
                    if let estimate = card.weeklyEstimate {
                        Text("预计本周")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                        HStack(alignment: .firstTextBaseline, spacing: 7) {
                            Text(estimate.projectedUtilization.map(percentage) ?? "—")
                                .font(.title2.weight(.bold))
                                .monospacedDigit()
                            StatusPill(
                                text: WidgetPresentation.stateLabel(estimate.state),
                                severity: WidgetPresentation.projectionSeverity(
                                    estimate.projectedUtilization,
                                    state: estimate.state
                                )
                            )
                        }
                        Text(WidgetPresentation.estimateHint(estimate))
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                            .lineLimit(2)
                    } else {
                        Text("等待更多数据")
                            .font(.subheadline.weight(.semibold))
                        Text(WidgetPresentation.estimateHint(nil))
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                    }

                    Spacer(minLength: 0)

                    InstrumentFooter(card: card, exact: false)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
            }
        } else {
            CompactNoDataView(card: card)
        }
    }
}

private struct LargeQuotaInstrument: View {
    let card: CardSnapshot

    var body: some View {
        if let weekly = card.weeklyTier {
            VStack(alignment: .leading, spacing: 12) {
                HStack(alignment: .top, spacing: 18) {
                    VStack(alignment: .leading, spacing: 7) {
                        Text("本周已用")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                        Text(percentage(weekly.utilization))
                            .font(.system(size: 40, weight: .bold, design: .rounded))
                            .monospacedDigit()
                        QuotaMeter(utilization: weekly.utilization)
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)

                    VStack(alignment: .leading, spacing: 5) {
                        Text("预计本周")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                        if let estimate = card.weeklyEstimate {
                            HStack(alignment: .firstTextBaseline, spacing: 8) {
                                Text(estimate.projectedUtilization.map(percentage) ?? "—")
                                    .font(.title.weight(.bold))
                                    .monospacedDigit()
                                StatusPill(
                                    text: WidgetPresentation.stateLabel(estimate.state),
                                    severity: WidgetPresentation.projectionSeverity(
                                        estimate.projectedUtilization,
                                        state: estimate.state
                                    )
                                )
                            }
                            Text(WidgetPresentation.estimateHint(estimate))
                                .font(.caption)
                                .foregroundStyle(.secondary)
                                .lineLimit(2)
                        }
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                }

                Divider()

                VStack(spacing: 10) {
                    ForEach(card.tiers, id: \.name) { tier in
                        TierInstrumentRow(tier: tier)
                    }
                }

                if let errorMessage = card.errorMessage {
                    Label(errorMessage, systemImage: "exclamationmark.triangle.fill")
                        .font(.caption2)
                        .foregroundStyle(Color.widgetSemantic(.warning))
                        .lineLimit(2)
                }

                Spacer(minLength: 0)
                InstrumentFooter(card: card, exact: true)
            }
        } else {
            CompactNoDataView(card: card)
        }
    }
}

private struct TierInstrumentRow: View {
    let tier: QuotaTier

    var body: some View {
        VStack(spacing: 5) {
            HStack(alignment: .firstTextBaseline) {
                Text(WidgetPresentation.tierLabel(tier.name))
                    .font(.subheadline.weight(.medium))
                Spacer()
                ResetLabel(tier: tier)
                Text(percentage(tier.utilization))
                    .font(.subheadline.weight(.semibold))
                    .monospacedDigit()
            }
            QuotaMeter(utilization: tier.utilization)
        }
    }
}

private struct ResetLabel: View {
    let tier: QuotaTier

    var body: some View {
        if let resetDate = resetDate(for: tier) {
            Text(resetDate, style: .relative)
                .font(.caption2)
                .foregroundStyle(.secondary)
                .lineLimit(1)
        }
    }
}

private struct InstrumentFooter: View {
    let card: CardSnapshot
    let exact: Bool

    var body: some View {
        HStack(spacing: 5) {
            Image(systemName: card.proxy.status == "proxy" ? "network" : "arrow.left.arrow.right")
                .accessibilityHidden(true)
            Text(
                exact
                    ? WidgetPresentation.connectionDetailLabel(card.proxy)
                    : WidgetPresentation.connectionLabel(card.proxy)
            )
            .lineLimit(exact ? 2 : 1)
            Spacer(minLength: 6)
            FreshnessLabel(card: card, exact: exact, showsIcon: false)
        }
        .font(.caption2)
        .foregroundStyle(.secondary)
    }
}

private struct FreshnessLabel: View {
    let card: CardSnapshot
    let exact: Bool
    let showsIcon: Bool

    var body: some View {
        if let updated = WidgetPresentation.date(milliseconds: card.queriedAt) {
            HStack(spacing: 3) {
                if showsIcon {
                    Image(systemName: "clock")
                }
                if exact {
                    Text(updated, format: .dateTime.month().day().hour().minute())
                } else {
                    Text(updated, style: .relative)
                }
            }
            .monospacedDigit()
            .lineLimit(1)
        }
    }
}

private struct CompactNoDataView: View {
    let card: CardSnapshot

    var body: some View {
        VStack(alignment: .leading, spacing: 7) {
            Spacer(minLength: 0)
            Text(card.status == .loginExpired ? "登录已失效" : "暂无用量数据")
                .font(.headline)
            Text(card.errorMessage ?? "打开 Agent Quota Control 检查账号并刷新。")
                .font(.caption)
                .foregroundStyle(.secondary)
                .lineLimit(3)
            Spacer(minLength: 0)
        }
    }
}

private func percentage(_ value: Double) -> String {
    "\(Int(value.rounded()))%"
}

private func projectionLabel(_ estimate: QuotaEstimate) -> String {
    let state = WidgetPresentation.stateLabel(estimate.state)
    guard let projected = estimate.projectedUtilization else { return state }
    return "预计 \(percentage(projected)) · \(state)"
}

private func resetDate(for tier: QuotaTier) -> Date? {
    guard let resetsAt = tier.resetsAt else { return nil }
    return ISO8601DateFormatter().date(from: resetsAt)
}
