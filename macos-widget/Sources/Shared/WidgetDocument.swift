import Foundation

public let widgetSchemaVersion = 1
public let widgetSupportDirectory = "io.ccswitch.agent-quota-control"
public let widgetKind = "AccountQuotaWidget"

public struct WidgetDocument: Codable, Sendable {
    public let schemaVersion: Int
    public let generatedAt: Int64
    public let accounts: [WidgetAccount]
    public let cardsByAccount: [String: CardSnapshot]
}

public struct WidgetAccount: Codable, Hashable, Sendable {
    public let id: String
    public let service: String
    public let displayName: String
    public let providerIdentityHint: String?

    public init(id: String, service: String, displayName: String, providerIdentityHint: String?) {
        self.id = id
        self.service = service
        self.displayName = displayName
        self.providerIdentityHint = providerIdentityHint
    }
}

public enum CardStatus: String, Codable, Sendable {
    case fresh
    case stale
    case updateFailed = "update_failed"
    case loginExpired = "login_expired"
    case noData = "no_data"
}

public enum SufficiencyState: String, Codable, Sendable {
    case enough
    case tight
    case notEnough = "not_enough"
    case unknown
}

public struct QuotaTier: Codable, Hashable, Sendable {
    public let name: String
    public let utilization: Double
    public let resetsAt: String?
    public let used: Double?
    public let limit: Double?
    public let remaining: Double?
}

public struct QuotaEstimate: Codable, Hashable, Sendable {
    public let state: SufficiencyState
    public let projectedUtilization: Double?
    public let resetInSecs: Int64?
    public let lastsForSecs: Int64?
    public let exhaustedAtSecs: Int64?
    public let exhaustedBeforeResetSecs: Int64?
}

public struct ProxySnapshot: Codable, Hashable, Sendable {
    public let status: String
    public let proxyUrl: String?
    public let message: String
}

public struct CardSnapshot: Codable, Hashable, Sendable {
    public let accountId: String
    public let service: String
    public let serviceDisplayName: String
    public let accountDisplayName: String
    public let status: CardStatus
    public let tiers: [QuotaTier]
    public let weeklyEstimate: QuotaEstimate?
    public let proxy: ProxySnapshot
    public let queriedAt: Int64?
    public let lastSuccessfulAt: Int64?
    public let errorMessage: String?

    public var weeklyTier: QuotaTier? {
        tiers.first { $0.name == "weekly_limit" || $0.name == "seven_day" }
    }
}

public enum AccountSelection: Equatable, Sendable {
    case account(WidgetAccount)
    case removed(String)
}

public enum AccountResolver {
    public static func resolve(ids: [String], from accounts: [WidgetAccount]) -> [WidgetAccount] {
        let indexed = Dictionary(uniqueKeysWithValues: accounts.map { ($0.id, $0) })
        return ids.compactMap { indexed[$0] }
    }

    public static func defaultAccount(from accounts: [WidgetAccount]) -> WidgetAccount? {
        accounts.first
    }

    public static func selection(id: String, from accounts: [WidgetAccount]) -> AccountSelection {
        guard let account = accounts.first(where: { $0.id == id }) else {
            return .removed(id)
        }
        return .account(account)
    }
}
