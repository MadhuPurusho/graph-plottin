// AppIntents.swift

import AppIntents
import Foundation

struct PlotExpressionIntent: AppIntent {
    static let title: LocalizedStringResource = "Plot Expression"
    static let description: IntentDescription = IntentDescription("Plot a mathematical expression on the graph.")

    @Parameter(title: "Expression")
    var expression: String

    static var parameterSummary: some ParameterSummary {
        Summary("Plot \(\.$expression)")
    }

    @MainActor
    func perform() async throws -> some IntentResult {
        // Will interact with the app state when integrated
        return .result()
    }
}

struct FitPointsIntent: AppIntent {
    static let title: LocalizedStringResource = "Fit Points"
    static let description: IntentDescription = IntentDescription("Run curve fitting on the current data points.")

    @Parameter(title: "Fit Type", default: "auto")
    var fitType: String

    static var parameterSummary: some ParameterSummary {
        Summary("Fit points with \(\.$fitType)")
    }

    @MainActor
    func perform() async throws -> some IntentResult {
        return .result()
    }
}
