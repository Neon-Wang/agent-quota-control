// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "AgentQuotaWidget",
    platforms: [.macOS(.v14)],
    products: [
        .library(name: "AgentQuotaWidgetShared", targets: ["AgentQuotaWidgetShared"]),
        .library(name: "AgentQuotaWidgetConfiguration", targets: ["AgentQuotaWidgetConfiguration"]),
    ],
    targets: [
        .target(
            name: "AgentQuotaWidgetShared",
            path: "Sources/Shared"
        ),
        .target(
            name: "AgentQuotaWidgetConfiguration",
            dependencies: ["AgentQuotaWidgetShared"],
            path: "Sources/Configuration"
        ),
        .testTarget(
            name: "AgentQuotaWidgetSharedTests",
            dependencies: ["AgentQuotaWidgetShared", "AgentQuotaWidgetConfiguration"],
            path: "Tests",
            resources: [.copy("Fixtures")]
        ),
    ]
)
