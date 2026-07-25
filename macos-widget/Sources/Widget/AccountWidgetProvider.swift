import Foundation
import WidgetKit

enum AccountQuotaContent {
    case card(WidgetAccount, CardSnapshot)
    case accountRemoved
    case unavailable(String)
}

struct AccountQuotaEntry: TimelineEntry {
    let date: Date
    let content: AccountQuotaContent
}

struct AccountWidgetProvider: AppIntentTimelineProvider {
    func placeholder(in context: Context) -> AccountQuotaEntry {
        AccountQuotaEntry(date: .now, content: .unavailable("正在读取用量"))
    }

    func snapshot(for configuration: AccountWidgetIntent, in context: Context) async -> AccountQuotaEntry {
        entry(for: configuration)
    }

    func timeline(for configuration: AccountWidgetIntent, in context: Context) async -> Timeline<AccountQuotaEntry> {
        let current = entry(for: configuration)
        return Timeline(entries: [current], policy: .after(Date().addingTimeInterval(15 * 60)))
    }

    private func entry(for configuration: AccountWidgetIntent) -> AccountQuotaEntry {
        do {
            let document = try WidgetStore().load()
            let selectedId = configuration.account?.id ?? document.accounts.first?.id
            guard let selectedId else {
                return AccountQuotaEntry(date: .now, content: .unavailable("还没有监控账号"))
            }
            guard let account = document.accounts.first(where: { $0.id == selectedId }) else {
                return AccountQuotaEntry(date: .now, content: .accountRemoved)
            }
            guard let card = document.cardsByAccount[selectedId] else {
                return AccountQuotaEntry(date: .now, content: .unavailable("暂无用量数据"))
            }
            return AccountQuotaEntry(date: .now, content: .card(account, card))
        } catch let error as WidgetStoreError {
            return AccountQuotaEntry(date: .now, content: .unavailable(error.description))
        } catch {
            return AccountQuotaEntry(date: .now, content: .unavailable("共享数据无法读取"))
        }
    }
}
