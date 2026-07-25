import Foundation
import Darwin
import os

public enum WidgetStoreError: Error, Equatable, CustomStringConvertible {
    case sharedContainerUnavailable
    case snapshotUnavailable
    case unsupportedSchema(Int)
    case invalidSnapshot

    public var description: String {
        switch self {
        case .sharedContainerUnavailable:
            return "无法访问共享数据"
        case .snapshotUnavailable:
            return "请先打开 Agent 配额控制台刷新用量"
        case .unsupportedSchema:
            return "请更新 Agent 配额控制台"
        case .invalidSnapshot:
            return "共享数据无法读取"
        }
    }
}

public struct WidgetStore: Sendable {
    private static let logger = Logger(
        subsystem: "io.ccswitch.agent-quota-control.widget",
        category: "WidgetStore"
    )
    private let fileURL: URL?

    public init(fileURL: URL? = nil) {
        self.fileURL = fileURL
    }

    public func load() throws -> WidgetDocument {
        let url = try resolvedFileURL()
        guard FileManager.default.fileExists(atPath: url.path) else {
            Self.logger.error("Widget snapshot is unavailable")
            throw WidgetStoreError.snapshotUnavailable
        }
        guard let document = try? JSONDecoder().decode(WidgetDocument.self, from: Data(contentsOf: url)) else {
            Self.logger.error("Widget snapshot could not be decoded")
            throw WidgetStoreError.invalidSnapshot
        }
        guard document.schemaVersion == widgetSchemaVersion else {
            throw WidgetStoreError.unsupportedSchema(document.schemaVersion)
        }
        Self.logger.info("Loaded widget snapshot with \(document.accounts.count) accounts")
        return document
    }

    private func resolvedFileURL() throws -> URL {
        if let fileURL { return fileURL }
        if let override = ProcessInfo.processInfo.environment["AGENT_QUOTA_WIDGET_DIR"] {
            return URL(fileURLWithPath: override, isDirectory: true)
                .appendingPathComponent("widget-snapshot.json")
        }
        guard let passwordEntry = getpwuid(getuid()) else {
            throw WidgetStoreError.sharedContainerUnavailable
        }
        let homeDirectory = URL(
            fileURLWithPath: String(cString: passwordEntry.pointee.pw_dir),
            isDirectory: true
        )
        return Self.snapshotURL(homeDirectory: homeDirectory)
    }

    static func snapshotURL(homeDirectory: URL) -> URL {
        homeDirectory
            .appendingPathComponent("Library", isDirectory: true)
            .appendingPathComponent("Application Support", isDirectory: true)
            .appendingPathComponent(widgetSupportDirectory, isDirectory: true)
            .appendingPathComponent("widget-snapshot.json")
    }
}
