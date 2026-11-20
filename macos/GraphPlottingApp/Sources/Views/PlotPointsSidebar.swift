// PlotPointsSidebar.swift

import SwiftUI

struct PlotPointsSidebar: View {
    @Environment(AppState.self) private var appState
    @State private var newX: String = ""
    @State private var newY: String = ""
    @State private var csvImportText: String = ""
    @State private var showCSVImport: Bool = false

    var body: some View {
        @Bindable var state = appState

        VStack(spacing: 0) {
            // Header
            headerSection

            Divider()

            // Data Points Table
            pointsTable

            Divider()

            // Fit Controls
            fitControlsSection

            // Fit Leaderboard
            if !appState.fitResults.isEmpty {
                Divider()
                fitLeaderboard
            }
        }
    }

    // MARK: - Header

    private var headerSection: some View {
        VStack(spacing: 8) {
            HStack {
                Text("Data Points")
                    .font(.headline)
                Spacer()
                Text("\(appState.points.count) pts")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            HStack(spacing: 6) {
                TextField("x", text: $newX)
                    .textFieldStyle(.roundedBorder)
                    .font(.system(.body, design: .monospaced))
                    .frame(maxWidth: .infinity)

                TextField("y", text: $newY)
                    .textFieldStyle(.roundedBorder)
                    .font(.system(.body, design: .monospaced))
                    .frame(maxWidth: .infinity)

                Button {
                    addPoint()
                } label: {
                    Image(systemName: "plus.circle.fill")
                }
                .buttonStyle(.borderless)
                .disabled(Double(newX) == nil || Double(newY) == nil)
                .keyboardShortcut(.return, modifiers: [])
            }

            HStack {
                Button("Import CSV…") {
                    importCSVFile()
                }
                .buttonStyle(.glass)

                Button("Paste CSV") {
                    showCSVImport.toggle()
                }
                .buttonStyle(.glass)

                Spacer()

                Button {
                    deleteSelected()
                } label: {
                    Image(systemName: "trash")
                }
                .buttonStyle(.glass)
                .disabled(appState.selectedPointIDs.isEmpty)
            }
        }
        .padding(12)
        .popover(isPresented: $showCSVImport) {
            VStack(spacing: 8) {
                Text("Paste CSV Data")
                    .font(.headline)
                TextEditor(text: $csvImportText)
                    .font(.system(.body, design: .monospaced))
                    .frame(minWidth: 300, minHeight: 200)
                HStack {
                    Button("Cancel") { showCSVImport = false }
                    Spacer()
                    Button("Import") {
                        appState.importCSV(csvImportText)
                        csvImportText = ""
                        showCSVImport = false
                    }
                    .buttonStyle(.borderedProminent)
                }
            }
            .padding()
        }
    }

    // MARK: - Points Table

    private var pointsTable: some View {
        @Bindable var state = appState
        return Table(of: DataPoint.self, selection: $state.selectedPointIDs) {
            TableColumn("x") { point in
                Text(String(format: "%.4f", point.x))
                    .font(.system(.body, design: .monospaced))
            }
            .width(min: 60, ideal: 80)

            TableColumn("y") { point in
                Text(String(format: "%.4f", point.y))
                    .font(.system(.body, design: .monospaced))
            }
            .width(min: 60, ideal: 80)

            TableColumn("") { point in
                if let label = point.label {
                    Text(label)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
            .width(min: 30, ideal: 50)
        } rows: {
            ForEach(appState.points) { point in
                TableRow(point)
            }
        }
        .alternatingRowBackgrounds(.enabled)
    }

    // MARK: - Fit Controls

    private var fitControlsSection: some View {
        @Bindable var state = appState
        return VStack(spacing: 8) {
            HStack {
                Text("Curve Fit")
                    .font(.headline)
                Spacer()
            }

            Picker("Fit Type", selection: $state.selectedFitType) {
                ForEach(FitType.allCases) { type in
                    Text(type.description).tag(type)
                }
            }
            .labelsHidden()

            HStack {
                Toggle("Robust (Huber)", isOn: $state.useRobustFitting)
                    .toggleStyle(.checkbox)
                    .font(.caption)
                Spacer()
            }

            Button("Run Fit") {
                appState.fitData()
            }
            .buttonStyle(.borderedProminent)
            .disabled(appState.points.count < 2 || appState.isComputing)
            .frame(maxWidth: .infinity)

            if appState.isComputing {
                ProgressView()
                    .controlSize(.small)
            }
        }
        .padding(12)
    }

    // MARK: - Fit Leaderboard

    private var fitLeaderboard: some View {
        @Bindable var state = appState
        return VStack(alignment: .leading, spacing: 4) {
            Text("Fit Results")
                .font(.headline)
                .padding(.horizontal, 12)
                .padding(.top, 8)

            List(appState.fitResults, selection: $state.activeFitID) { fit in
                VStack(alignment: .leading, spacing: 2) {
                    HStack {
                        Text(fit.fitType.description)
                            .font(.subheadline.weight(.semibold))
                        Spacer()
                        Text("R² = \(fit.formattedR2)")
                            .font(.system(.caption, design: .monospaced))
                    }
                    Text(fit.equation)
                        .font(.system(.caption, design: .monospaced))
                        .foregroundStyle(.secondary)
                        .lineLimit(2)
                    HStack(spacing: 12) {
                        Text("RMSE: \(fit.formattedRMSE)")
                        Text("AICc: \(fit.formattedAICc)")
                    }
                    .font(.system(.caption2, design: .monospaced))
                    .foregroundStyle(.tertiary)
                }
                .padding(.vertical, 2)
            }
            .listStyle(.sidebar)

            HStack {
                Toggle("Confidence", isOn: $state.showConfidenceBands)
                Toggle("Prediction", isOn: $state.showPredictionBands)
            }
            .toggleStyle(.checkbox)
            .font(.caption)
            .padding(.horizontal, 12)
            .padding(.bottom, 8)
        }
    }

    // MARK: - Actions

    private func addPoint() {
        guard let x = Double(newX), let y = Double(newY) else { return }
        appState.addPoint(DataPoint(x: x, y: y))
        newX = ""
        newY = ""
    }

    private func deleteSelected() {
        appState.removePoints(ids: appState.selectedPointIDs)
    }

    private func importCSVFile() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.commaSeparatedText, .tabSeparatedText, .plainText]
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        if panel.runModal() == .OK, let url = panel.url {
            if let content = try? String(contentsOf: url, encoding: .utf8) {
                appState.importCSV(content)
            }
        }
    }
}
