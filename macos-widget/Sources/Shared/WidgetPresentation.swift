import Foundation

public enum WidgetFamilyKind: Sendable {
    case small
    case medium
    case large
}

public enum WidgetField: Hashable, Sendable {
    case weekly
    case projection
    case connection
    case updatedAt
    case reset
    case estimateHint
    case allTiers
    case exactUpdate
    case error
}

public enum WidgetSeverity: Hashable, Sendable {
    case neutral
    case normal
    case warning
    case critical
}

public enum WidgetCardMode: Hashable, Sendable {
    case metrics
    case degradedMetrics
    case actionRequired
}

public struct WidgetStatusNotice: Equatable, Sendable {
    public let text: String
    public let severity: WidgetSeverity

    public init(text: String, severity: WidgetSeverity) {
        self.text = text
        self.severity = severity
    }
}

public enum WidgetPresentation {
    public static func stateLabel(_ state: SufficiencyState) -> String {
        switch state {
        case .enough: "够"
        case .tight: "偏紧"
        case .notEnough: "不够"
        case .unknown: "未知"
        }
    }

    public static func fields(for family: WidgetFamilyKind) -> [WidgetField] {
        switch family {
        case .small:
            [.weekly, .projection, .updatedAt]
        case .medium:
            [.weekly, .projection, .updatedAt, .connection, .reset, .estimateHint, .error]
        case .large:
            [.weekly, .projection, .connection, .updatedAt, .reset, .estimateHint, .allTiers, .exactUpdate, .error]
        }
    }

    public static func utilizationSeverity(_ utilization: Double) -> WidgetSeverity {
        if utilization >= 90 { return .critical }
        if utilization >= 70 { return .warning }
        return .normal
    }

    public static func projectionSeverity(
        _ projectedUtilization: Double?,
        state: SufficiencyState
    ) -> WidgetSeverity {
        guard let projectedUtilization else { return .neutral }
        if projectedUtilization > 100 { return .critical }
        switch state {
        case .enough: return .normal
        case .tight: return .warning
        case .notEnough: return .critical
        case .unknown: return .neutral
        }
    }

    public static func cardMode(for status: CardStatus) -> WidgetCardMode {
        switch status {
        case .fresh: .metrics
        case .stale, .updateFailed: .degradedMetrics
        case .loginExpired, .noData: .actionRequired
        }
    }

    public static func statusNotice(for status: CardStatus) -> WidgetStatusNotice? {
        switch status {
        case .fresh: nil
        case .stale: WidgetStatusNotice(text: "数据较旧", severity: .warning)
        case .updateFailed: WidgetStatusNotice(text: "更新失败", severity: .warning)
        case .loginExpired: WidgetStatusNotice(text: "登录失效", severity: .critical)
        case .noData: WidgetStatusNotice(text: "暂无数据", severity: .neutral)
        }
    }

    public static func tierLabel(_ name: String) -> String {
        switch name {
        case "five_hour": "5 小时"
        case "weekly_limit", "seven_day": "7 天"
        default: name
        }
    }

    public static func connectionLabel(_ proxy: ProxySnapshot) -> String {
        switch proxy.status {
        case "proxy": "代理已连接"
        case "direct": "当前直连"
        default: "连接状态未知"
        }
    }

    public static func connectionDetailLabel(_ proxy: ProxySnapshot) -> String {
        if proxy.status == "proxy", let proxyURL = proxy.proxyUrl {
            return "代理已连接：\(proxyURL)"
        }
        if proxy.status == "direct" {
            return "未检测到可用代理，当前走直连"
        }
        return proxy.message.isEmpty ? "代理未连通，请检查地址和本地端口" : proxy.message
    }

    public static func estimateHint(_ estimate: QuotaEstimate?) -> String {
        guard let estimate else { return "等待更多用量数据后估算。" }
        if let seconds = estimate.exhaustedBeforeResetSecs {
            return "已提前 \(duration(seconds))耗尽。"
        }
        switch estimate.state {
        case .enough: return "本周内预计够用。"
        case .tight: return "本周内预计不会耗尽，但余量偏紧。"
        case .notEnough:
            guard let seconds = estimate.lastsForSecs else { return "本周预计不够用。" }
            return "预计将在 \(duration(seconds)) 后耗尽。"
        case .unknown: return "等待更多用量数据后估算。"
        }
    }

    public static func duration(_ seconds: Int64) -> String {
        let days = seconds / 86_400
        let hours = (seconds % 86_400) / 3_600
        if days > 0 && hours > 0 { return "\(days) 天 \(hours) 小时" }
        if days > 0 { return "\(days) 天" }
        return "\(hours) 小时"
    }

    public static func date(milliseconds: Int64?) -> Date? {
        milliseconds.map { Date(timeIntervalSince1970: Double($0) / 1_000) }
    }
}
