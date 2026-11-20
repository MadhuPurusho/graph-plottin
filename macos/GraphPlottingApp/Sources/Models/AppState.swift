// AppState.swift

import Foundation
import SwiftUI

enum AppMode: String, Sendable {
    case plotPoints = "Plot Points"
    case functions = "Functions"
}

@MainActor
@Observable
final class AppState {
    // MARK: - Mode
    var mode: AppMode = .plotPoints

    // MARK: - Data Points
    var points: [DataPoint] = []
    var selectedPointIDs: Set<UUID> = []

    // MARK: - Fit Results
    var selectedFitType: FitType = .auto
    var fitResults: [FitResultEntry] = []
    var activeFitID: UUID?
    var useRobustFitting: Bool = false
    var showConfidenceBands: Bool = true
    var showPredictionBands: Bool = false

    // MARK: - Expressions
    var expressions: [ExpressionEntry] = [ExpressionEntry()]

    // MARK: - Viewport
    var viewport: ViewportState = ViewportState()

    // MARK: - UI State
    var showSidebar: Bool = true
    var showInspector: Bool = false
    var inspectorTab: InspectorTab = .diagnostics
    var showExportSheet: Bool = false
    var isComputing: Bool = false
    var errorMessage: String?

    // MARK: - Probe
    var probeX: Double?
    var probeY: Double?
    var showProbe: Bool = false

    // MARK: - Rendering
    var needsRedraw: Bool = true

    init() {
        let samplePoints: [(Double, Double)] = [
            (-4.0, 16.2), (-3.0, 9.1), (-2.0, 4.05), (-1.0, 1.1),
            (0.0, 0.05), (1.0, 1.02), (2.0, 3.98), (3.0, 8.95), (4.0, 15.9)
        ]
        self.points = samplePoints.map { DataPoint(x: $0.0, y: $0.1) }
        self.selectedFitType = .quadratic
        self.viewport = ViewportState(xMin: -6.0, xMax: 6.0, yMin: -2.0, yMax: 18.0)
        self.showInspector = true
        self.expressions = [
            ExpressionEntry(source: "sin(x) * cos(x/2)", color: .blue),
            ExpressionEntry(source: "x^2 - 4", isVisible: false, color: .orange)
        ]
        Task {
            self.fitData()
        }
    }

    enum InspectorTab: String, CaseIterable {
        case diagnostics = "Diagnostics"
        case calculus = "Calculus"
    }

    // MARK: - Actions

    func addPoint(_ point: DataPoint) {
        points.append(point)
        needsRedraw = true
    }

    func removePoints(ids: Set<UUID>) {
        points.removeAll { ids.contains($0.id) }
        selectedPointIDs.subtract(ids)
        needsRedraw = true
    }

    func addExpression() {
        let idx = expressions.count
        let expr = ExpressionEntry(
            color: CurvePalette.color(at: idx)
        )
        expressions.append(expr)
    }

    func removeExpression(id: UUID) {
        expressions.removeAll { $0.id == id }
        needsRedraw = true
    }

    func fitData() {
        guard points.count >= 2 else { return }
        isComputing = true
        let corePoints = points.map(\.corePoint)
        let fitType = selectedFitType
        let robust = useRobustFitting

        Task {
            let results: [CoreFitResult]? = await Task.detached {
                do {
                    if fitType == .auto {
                        return try GraphPlottingCore.autoFit(points: corePoints, useRobust: robust)
                    } else {
                        return [try GraphPlottingCore.fitCurve(
                            points: corePoints,
                            fitType: fitType,
                            useRobust: robust
                        )]
                    }
                } catch {
                    return nil
                }
            }.value

            if let results = results {
                self.fitResults = results.map { FitResultEntry(from: $0) }
                self.activeFitID = self.fitResults.first?.id
                self.isComputing = false
                self.needsRedraw = true
            } else {
                self.errorMessage = "Curve fitting failed to converge"
                self.isComputing = false
            }
        }
    }

    func fitToData() {
        viewport.fitToData(points: points)
        needsRedraw = true
    }

    func clearAll() {
        points.removeAll()
        fitResults.removeAll()
        activeFitID = nil
        selectedPointIDs.removeAll()
        expressions = [ExpressionEntry()]
        viewport.reset()
        needsRedraw = true
    }

    func importCSV(_ content: String) {
        let parsed = GraphPlottingCore.parseCSV(content)
        let newPoints = parsed.map { DataPoint(from: $0) }
        points.append(contentsOf: newPoints)
        needsRedraw = true
    }
}
