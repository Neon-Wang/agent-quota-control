import Foundation
import Testing
@testable import AgentQuotaWidgetShared

@Test func decodesRustSnapshotFixture() throws {
    let url = try #require(Bundle.module.url(forResource: "widget-snapshot", withExtension: "json", subdirectory: "Fixtures"))
    let document = try WidgetStore(fileURL: url).load()

    #expect(document.schemaVersion == 1)
    #expect(document.accounts.map(\.id) == ["kimi-work", "codex-personal"])
    #expect(document.cardsByAccount["kimi-work"]?.weeklyTier?.utilization == 48)
    #expect(document.cardsByAccount["kimi-work"]?.weeklyEstimate?.state == .enough)
}

@Test func rejectsUnsupportedFutureSchema() throws {
    let data = Data(#"{"schemaVersion":2,"generatedAt":0,"accounts":[],"cardsByAccount":{}}"#.utf8)
    let url = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
    try data.write(to: url)
    defer { try? FileManager.default.removeItem(at: url) }

    #expect(throws: WidgetStoreError.unsupportedSchema(2)) {
        try WidgetStore(fileURL: url).load()
    }
}

@Test func resolvesSnapshotBelowTheRealUserHome() {
    let url = WidgetStore.snapshotURL(
        homeDirectory: URL(fileURLWithPath: "/Users/tester", isDirectory: true)
    )

    #expect(url.path == "/Users/tester/Library/Application Support/io.ccswitch.agent-quota-control/widget-snapshot.json")
}

@Test func resolvesAccountsInRequestedOrderAndKeepsMissingSelection() throws {
    let accounts = [
        WidgetAccount(id: "first", service: "kimi", displayName: "First", providerIdentityHint: nil),
        WidgetAccount(id: "second", service: "codex", displayName: "Second", providerIdentityHint: nil),
    ]

    #expect(AccountResolver.resolve(ids: ["second", "first"], from: accounts).map(\.id) == ["second", "first"])
    #expect(AccountResolver.defaultAccount(from: accounts)?.id == "first")
    #expect(AccountResolver.selection(id: "removed", from: accounts) == .removed("removed"))
}
