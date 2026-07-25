import AgentQuotaWidgetShared
import Foundation
import Testing
@testable import AgentQuotaWidgetConfiguration

@Test func accountEntityQueryListsAndResolvesSnapshotAccounts() async throws {
    let url = try #require(Bundle.module.url(
        forResource: "widget-snapshot",
        withExtension: "json",
        subdirectory: "Fixtures"
    ))
    let query = AccountEntityQuery(store: WidgetStore(fileURL: url))

    let suggested = try await query.suggestedEntities()
    #expect(suggested.map(\.id) == ["kimi-work", "codex-personal"])
    #expect(suggested.map(\.name) == ["工作 Kimi", "个人 Codex"])

    let resolved = try await query.entities(for: ["codex-personal", "kimi-work"])
    #expect(resolved.map(\.id) == ["codex-personal", "kimi-work"])
    #expect(await query.defaultResult()?.id == "kimi-work")
}
