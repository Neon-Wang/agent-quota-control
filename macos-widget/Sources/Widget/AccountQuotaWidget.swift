import SwiftUI
import WidgetKit

struct AccountQuotaWidget: Widget {
    let kind = widgetKind

    var body: some WidgetConfiguration {
        AppIntentConfiguration(
            kind: kind,
            intent: AccountWidgetIntent.self,
            provider: AccountWidgetProvider()
        ) { entry in
            AccountWidgetView(entry: entry)
        }
        .configurationDisplayName("账号配额")
        .description("显示一个 Kimi Code 或 Codex 监控账号的用量。")
        .supportedFamilies([.systemSmall, .systemMedium, .systemLarge])
    }
}
