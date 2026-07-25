import SwiftUI

struct WidgetSurface: View {
    @Environment(\.colorScheme) private var colorScheme

    var body: some View {
        Group {
            if colorScheme == .dark {
                Color(red: 0.075, green: 0.082, blue: 0.094)
            } else {
                Color(red: 0.965, green: 0.972, blue: 0.982)
            }
        }
    }
}

extension Color {
    static func widgetSemantic(_ severity: WidgetSeverity) -> Color {
        switch severity {
        case .neutral: .secondary
        case .normal: Color(red: 0.15, green: 0.62, blue: 0.38)
        case .warning: Color(red: 0.92, green: 0.55, blue: 0.10)
        case .critical: Color(red: 0.88, green: 0.25, blue: 0.24)
        }
    }
}

struct QuotaMeter: View {
    let utilization: Double

    private var progress: Double {
        min(max(utilization / 100, 0), 1)
    }

    var body: some View {
        GeometryReader { proxy in
            ZStack(alignment: .leading) {
                Capsule()
                    .fill(Color.primary.opacity(0.10))
                Capsule()
                    .fill(Color.widgetSemantic(WidgetPresentation.utilizationSeverity(utilization)))
                    .frame(width: proxy.size.width * progress)
            }
        }
        .frame(height: 5)
        .accessibilityLabel("本周用量")
        .accessibilityValue("\(Int(utilization.rounded()))%")
    }
}

struct StatusPill: View {
    let text: String
    let severity: WidgetSeverity

    var body: some View {
        Text(text)
            .font(.caption2.weight(.semibold))
            .foregroundStyle(Color.widgetSemantic(severity))
            .padding(.horizontal, 7)
            .padding(.vertical, 3)
            .background(Color.widgetSemantic(severity).opacity(0.12))
            .clipShape(Capsule())
    }
}
