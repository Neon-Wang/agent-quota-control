import AppIntents

struct AccountWidgetIntent: WidgetConfigurationIntent {
    static let title: LocalizedStringResource = "配额账号"
    static let description = IntentDescription("选择此 Widget 显示的上游监控账号。")

    @Parameter(title: "账号")
    var account: AccountEntity?

    init() {}

    init(account: AccountEntity?) {
        self.account = account
    }
}
