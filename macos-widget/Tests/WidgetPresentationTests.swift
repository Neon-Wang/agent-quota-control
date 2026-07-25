import Testing
@testable import AgentQuotaWidgetShared

@Test func presentationUsesCanonicalEstimateState() {
    #expect(WidgetPresentation.stateLabel(.enough) == "够")
    #expect(WidgetPresentation.stateLabel(.tight) == "偏紧")
    #expect(WidgetPresentation.stateLabel(.notEnough) == "不够")
    #expect(WidgetPresentation.fields(for: .small) == [.weekly, .projection, .updatedAt])
    #expect(
        WidgetPresentation.fields(for: .medium)
            == [.weekly, .projection, .updatedAt, .connection, .reset, .estimateHint, .error]
    )
    #expect(WidgetPresentation.fields(for: .large).contains(.allTiers))
    #expect(WidgetPresentation.fields(for: .large).contains(.exactUpdate))
}

@Test func presentationCopyMatchesTheOverviewCard() {
    let proxy = ProxySnapshot(
        status: "proxy",
        proxyUrl: "http://127.0.0.1:7897",
        message: "Proxy"
    )
    let direct = ProxySnapshot(status: "direct", proxyUrl: nil, message: "Direct")
    let estimate = QuotaEstimate(
        state: .notEnough,
        projectedUtilization: 188,
        resetInSecs: nil,
        lastsForSecs: 93_600,
        exhaustedAtSecs: nil,
        exhaustedBeforeResetSecs: nil
    )
    let exhausted = QuotaEstimate(
        state: .notEnough,
        projectedUtilization: 200,
        resetInSecs: nil,
        lastsForSecs: nil,
        exhaustedAtSecs: nil,
        exhaustedBeforeResetSecs: 7_200
    )

    #expect(WidgetPresentation.connectionDetailLabel(proxy) == "代理已连接：http://127.0.0.1:7897")
    #expect(WidgetPresentation.connectionDetailLabel(direct) == "未检测到可用代理，当前走直连")
    #expect(WidgetPresentation.estimateHint(estimate) == "预计将在 1 天 2 小时 后耗尽。")
    #expect(WidgetPresentation.estimateHint(exhausted) == "已提前 2 小时耗尽。")
}

@Test func utilizationSeverityUsesApprovedThresholds() {
    #expect(WidgetPresentation.utilizationSeverity(0) == .normal)
    #expect(WidgetPresentation.utilizationSeverity(69.99) == .normal)
    #expect(WidgetPresentation.utilizationSeverity(70) == .warning)
    #expect(WidgetPresentation.utilizationSeverity(89.99) == .warning)
    #expect(WidgetPresentation.utilizationSeverity(90) == .critical)
    #expect(WidgetPresentation.utilizationSeverity(160) == .critical)
}

@Test func projectionOverOneHundredIsAlwaysCritical() {
    #expect(WidgetPresentation.projectionSeverity(nil, state: .notEnough) == .neutral)
    #expect(WidgetPresentation.projectionSeverity(85, state: .enough) == .normal)
    #expect(WidgetPresentation.projectionSeverity(95, state: .tight) == .warning)
    #expect(WidgetPresentation.projectionSeverity(100, state: .notEnough) == .critical)
    #expect(WidgetPresentation.projectionSeverity(100.01, state: .enough) == .critical)
}

@Test func degradedRefreshKeepsMetricsButCredentialFailureDoesNot() {
    #expect(WidgetPresentation.cardMode(for: .fresh) == .metrics)
    #expect(WidgetPresentation.cardMode(for: .stale) == .degradedMetrics)
    #expect(WidgetPresentation.cardMode(for: .updateFailed) == .degradedMetrics)
    #expect(WidgetPresentation.cardMode(for: .loginExpired) == .actionRequired)
    #expect(WidgetPresentation.cardMode(for: .noData) == .actionRequired)
}

@Test func cardStatusUsesBoundedTextAndSeverity() {
    #expect(WidgetPresentation.statusNotice(for: .fresh) == nil)
    #expect(
        WidgetPresentation.statusNotice(for: .stale)
            == WidgetStatusNotice(text: "数据较旧", severity: .warning)
    )
    #expect(
        WidgetPresentation.statusNotice(for: .updateFailed)
            == WidgetStatusNotice(text: "更新失败", severity: .warning)
    )
    #expect(
        WidgetPresentation.statusNotice(for: .loginExpired)
            == WidgetStatusNotice(text: "登录失效", severity: .critical)
    )
    #expect(
        WidgetPresentation.statusNotice(for: .noData)
            == WidgetStatusNotice(text: "暂无数据", severity: .neutral)
    )
}
