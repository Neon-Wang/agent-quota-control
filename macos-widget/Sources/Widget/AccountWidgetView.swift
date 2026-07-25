import SwiftUI
import WidgetKit

struct AccountWidgetView: View {
    @Environment(\.widgetFamily) private var family
    let entry: AccountQuotaEntry

    var body: some View {
        Group {
            switch entry.content {
            case let .card(account, card):
                QuotaInstrumentView(account: account, card: card, family: family)
            case .accountRemoved:
                WidgetMessageView(
                    symbol: "person.crop.circle.badge.xmark",
                    title: "账号已删除",
                    detail: "编辑 Widget，重新选择一个监控账号。"
                )
            case let .unavailable(message):
                WidgetMessageView(
                    symbol: "gauge.with.dots.needle.0percent",
                    title: "Agent 配额",
                    detail: message
                )
            }
        }
        .containerBackground(for: .widget) {
            WidgetSurface()
        }
    }
}

private struct WidgetMessageView: View {
    let symbol: String
    let title: String
    let detail: String

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            Image(systemName: symbol)
                .font(.title2.weight(.medium))
                .foregroundStyle(.secondary)
            Spacer(minLength: 0)
            Text(title)
                .font(.headline)
            Text(detail)
                .font(.caption)
                .foregroundStyle(.secondary)
                .lineLimit(3)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .leading)
        .accessibilityElement(children: .combine)
    }
}
