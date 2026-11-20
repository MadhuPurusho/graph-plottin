// DiagnosticsInspector.swift
// Statistical Diagnostics tab displaying goodness-of-fit, parameter standard errors, and outlier analysis.

import SwiftUI

struct DiagnosticsInspector: View {
    @Environment(AppState.self) private var appState

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                if let fit = appState.fitResults.first(where: { $0.id == appState.activeFitID }) {
                    // Header Card
                    VStack(alignment: .leading, spacing: 6) {
                        Text(fit.fitType.description)
                            .font(.headline)

                        Text(fit.equation)
                            .font(.system(.body, design: .monospaced))
                            .textSelection(.enabled)
                    }
                    .padding(10)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(Color(NSColor.controlBackgroundColor).opacity(0.6), in: RoundedRectangle(cornerRadius: 8))

                    // Goodness of Fit Table
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Goodness of Fit")
                            .font(.subheadline.bold())

                        Grid(alignment: .leading, horizontalSpacing: 12, verticalSpacing: 6) {
                            GridRow {
                                Text("R²:").foregroundStyle(.secondary)
                                Text(fit.formattedR2).font(.system(.caption, design: .monospaced).bold())

                                Text("Adj R²:").foregroundStyle(.secondary)
                                Text(String(format: "%.6f", fit.adjustedRSquared)).font(.system(.caption, design: .monospaced))
                            }
                            GridRow {
                                Text("RMSE:").foregroundStyle(.secondary)
                                Text(fit.formattedRMSE).font(.system(.caption, design: .monospaced))

                                Text("MAE:").foregroundStyle(.secondary)
                                Text(String(format: "%.4f", fit.mae)).font(.system(.caption, design: .monospaced))
                            }
                            GridRow {
                                Text("AICc:").foregroundStyle(.secondary)
                                Text(fit.formattedAICc).font(.system(.caption, design: .monospaced))

                                Text("BIC:").foregroundStyle(.secondary)
                                Text(String(format: "%.2f", fit.bic)).font(.system(.caption, design: .monospaced))
                            }
                            GridRow {
                                Text("LOOCV:").foregroundStyle(.secondary)
                                Text(String(format: "%.4f", fit.loocvRmse)).font(.system(.caption, design: .monospaced))
                            }
                        }
                        .font(.caption)
                    }

                    Divider()

                    // Parameter Estimates with Standard Errors
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Parameter Estimates (± SE)")
                            .font(.subheadline.bold())

                        Table(parameterRows(for: fit)) {
                            TableColumn("Parameter") { row in
                                Text(row.name)
                                    .font(.system(.caption, design: .monospaced).bold())
                            }
                            .width(min: 60, ideal: 70, max: 80)

                            TableColumn("Estimate") { row in
                                Text(String(format: "%+.6f", row.value))
                                    .font(.system(.caption, design: .monospaced))
                            }

                            TableColumn("Std Error") { row in
                                Text(String(format: "±%.6f", row.standardError))
                                    .font(.system(.caption, design: .monospaced))
                                    .foregroundStyle(.secondary)
                            }
                        }
                        .frame(minHeight: CGFloat(min(fit.parameters.count * 28 + 30, 160)))
                    }

                    Divider()

                    // Cook's Distance Outlier Detection
                    VStack(alignment: .leading, spacing: 6) {
                        let outlierIndices = fit.cooksDistances.enumerated().filter { $0.element > (4.0 / Double(max(fit.cooksDistances.count, 1))) }
                        HStack {
                            Text("Outlier Diagnostics")
                                .font(.subheadline.bold())
                            Spacer()
                            if outlierIndices.isEmpty {
                                Text("No high-leverage outliers")
                                    .font(.caption2)
                                    .foregroundStyle(.secondary)
                            } else {
                                Text("\(outlierIndices.count) flagged")
                                    .font(.caption2.bold())
                                    .foregroundStyle(.orange)
                            }
                        }

                        if !outlierIndices.isEmpty {
                            VStack(alignment: .leading, spacing: 4) {
                                ForEach(outlierIndices, id: \.offset) { idx, dist in
                                    HStack {
                                        Text("Point #\(idx + 1)")
                                            .font(.caption.bold())
                                        Spacer()
                                        Text(String(format: "Cook's D = %.3f", dist))
                                            .font(.system(.caption, design: .monospaced))
                                            .foregroundStyle(.orange)
                                    }
                                }
                            }
                            .padding(8)
                            .background(Color.orange.opacity(0.1), in: RoundedRectangle(cornerRadius: 6))
                        }
                    }

                    Divider()

                    // Visual Bands & Robust Toggles
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Plot Overlays")
                            .font(.subheadline.bold())

                        Toggle("95% Confidence Band", isOn: Binding(
                            get: { appState.showConfidenceBands },
                            set: { appState.showConfidenceBands = $0; appState.needsRedraw = true }
                        ))
                        .toggleStyle(.checkbox)

                        Toggle("95% Prediction Band", isOn: Binding(
                            get: { appState.showPredictionBands },
                            set: { appState.showPredictionBands = $0; appState.needsRedraw = true }
                        ))
                        .toggleStyle(.checkbox)

                        Toggle("Huber Robust Fitting (IRLS)", isOn: Binding(
                            get: { appState.useRobustFitting },
                            set: { appState.useRobustFitting = $0; appState.fitData() }
                        ))
                        .toggleStyle(.checkbox)
                    }
                    .font(.caption)

                } else {
                    ContentUnavailableView(
                        "No Active Fit",
                        systemImage: "chart.xyaxis.line",
                        description: Text("Add at least 2 data points and fit a curve to view statistical diagnostics.")
                    )
                }
            }
            .padding(12)
        }
    }

    private func parameterRows(for fit: FitResultEntry) -> [ParameterRow] {
        var rows: [ParameterRow] = []
        for i in 0..<fit.parameters.count {
            let name = i < fit.parameterNames.count ? fit.parameterNames[i] : "p\(i)"
            let val = fit.parameters[i]
            let se = i < fit.standardErrors.count ? fit.standardErrors[i] : 0.0
            rows.append(ParameterRow(id: i, name: name, value: val, standardError: se))
        }
        return rows
    }
}

private struct ParameterRow: Identifiable {
    let id: Int
    let name: String
    let value: Double
    let standardError: Double
}
