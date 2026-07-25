#if SWIFT_PACKAGE
import AgentQuotaWidgetShared
#endif
import AppIntents
import Foundation

struct AccountEntity: AppEntity, Identifiable {
    static let typeDisplayRepresentation = TypeDisplayRepresentation(name: "监控账号")
    static let defaultQuery = AccountEntityQuery()

    let id: String
    @Property(title: "服务") var service: String
    @Property(title: "账号名称") var name: String
    @Property(title: "身份提示") var identityHint: String?

    var displayRepresentation: DisplayRepresentation {
        DisplayRepresentation(
            title: LocalizedStringResource(stringLiteral: name),
            subtitle: LocalizedStringResource(stringLiteral: service == "kimi" ? "Kimi Code" : "Codex")
        )
    }

    init(account: WidgetAccount) {
        id = account.id
        service = account.service
        name = account.displayName
        identityHint = account.providerIdentityHint
    }
}

struct AccountEntityQuery: EntityQuery {
    private let store: WidgetStore

    init() {
        store = WidgetStore()
    }

    init(store: WidgetStore) {
        self.store = store
    }

    func entities(for identifiers: [String]) async throws -> [AccountEntity] {
        let accounts = try store.load().accounts
        return AccountResolver.resolve(ids: identifiers, from: accounts).map(AccountEntity.init)
    }

    func suggestedEntities() async throws -> [AccountEntity] {
        try store.load().accounts.map(AccountEntity.init)
    }

    func defaultResult() async -> AccountEntity? {
        guard let account = try? store.load().accounts.first else { return nil }
        return AccountEntity(account: account)
    }
}
