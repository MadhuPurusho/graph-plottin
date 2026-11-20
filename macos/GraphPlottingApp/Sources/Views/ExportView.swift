// ExportView.swift
// Publication-grade export sheet for SVG, LaTeX/PGFPlots, and CSV.

import SwiftUI
import AppKit

struct ExportView: View {
    @Environment(AppState.self) private var appState
    @Environment(\.dismiss) private var dismiss

    enum ExportFormat: String, CaseIterable, Identifiable {
        case svg = "Vector SVG"
        case latex = "LaTeX / TikZ"
        case csv = "Data Points (CSV)"
        var id: String { rawValue }
    }

    @State private var selectedFormat: ExportFormat = .svg
    @State private var exportWidth: String = "800"
    @State private var exportHeight: String = "600"
    @State private var generatedContent: String = ""
    @State private var copiedFeedback: Bool = false

    var body: some View {
        VStack(spacing: 16) {
            // Header
            HStack {
                Text("Export Publication Graph")
                    .font(.headline)
                Spacer()
                Button {
                    dismiss()
                } label: {
                    Image(systemName: "xmark.circle.fill")
                        .foregroundStyle(.secondary)
                }
                .buttonStyle(.plain)
            }

            // Format Picker
            Picker("Format", selection: $selectedFormat) {
                ForEach(ExportFormat.allCases) { fmt in
                    Text(fmt.rawValue).tag(fmt)
                }
            }
            .pickerStyle(.segmented)
            .onChange(of: selectedFormat) { _, _ in
                updateGeneratedContent()
            }

            // Format-specific options
            if selectedFormat == .svg {
                HStack(spacing: 12) {
                    Text("Dimensions:").font(.caption).foregroundStyle(.secondary)
                    TextField("Width", text: $exportWidth)
                        .textFieldStyle(.roundedBorder)
                        .frame(width: 70)
                    Text("×").font(.caption).foregroundStyle(.secondary)
                    TextField("Height", text: $exportHeight)
                        .textFieldStyle(.roundedBorder)
                        .frame(width: 70)
                    Spacer()
                    Button("Refresh") {
                        updateGeneratedContent()
                    }
                    .buttonStyle(.glass)
                }
            }

            // Preview Box
            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    Text("Preview")
                        .font(.caption.bold())
                        .foregroundStyle(.secondary)
                    Spacer()
                    if copiedFeedback {
                        Text("Copied to Clipboard!")
                            .font(.caption2.bold())
                            .foregroundStyle(.green)
                    }
                }

                ScrollView([.horizontal, .vertical]) {
                    Text(generatedContent)
                        .font(.system(.caption2, design: .monospaced))
                        .textSelection(.enabled)
                        .padding(8)
                        .frame(maxWidth: .infinity, alignment: .topLeading)
                }
                .frame(minWidth: 460, minHeight: 240, maxHeight: 320)
                .background(Color(NSColor.textBackgroundColor), in: RoundedRectangle(cornerRadius: 6))
                .overlay(RoundedRectangle(cornerRadius: 6).stroke(Color.secondary.opacity(0.2), lineWidth: 1))
            }

            // Action Buttons
            HStack {
                Button("Copy to Clipboard") {
                    copyToClipboard()
                }
                .buttonStyle(.glass)

                Spacer()

                Button("Save File…") {
                    saveFile()
                }
                .buttonStyle(.borderedProminent)
            }
        }
        .padding(20)
        .frame(minWidth: 500, minHeight: 440)
        .onAppear {
            updateGeneratedContent()
        }
    }

    private func updateGeneratedContent() {
        let vp = appState.viewport
        let corePoints = appState.points.map(\.corePoint)
        let activeFit = appState.fitResults.first(where: { $0.id == appState.activeFitID })

        switch selectedFormat {
        case .svg:
            let w = Int(exportWidth) ?? 800
            let h = Int(exportHeight) ?? 600
            generatedContent = GraphPlottingCore.exportSVG(
                strips: [],
                points: corePoints,
                confUpper: activeFit?.confidenceBandUpper.map(\.corePoint) ?? [],
                confLower: activeFit?.confidenceBandLower.map(\.corePoint) ?? [],
                equationUnicode: activeFit?.equation ?? "",
                rSquared: activeFit?.rSquared,
                xMin: vp.xMin,
                xMax: vp.xMax,
                yMin: vp.yMin,
                yMax: vp.yMax,
                width: w,
                height: h
            )

        case .latex:
            generatedContent = GraphPlottingCore.exportLaTeX(
                equationLatex: activeFit?.latexEquation ?? "",
                points: corePoints,
                rSquared: activeFit?.rSquared,
                xMin: vp.xMin,
                xMax: vp.xMax,
                yMin: vp.yMin,
                yMax: vp.yMax
            )

        case .csv:
            generatedContent = GraphPlottingCore.exportCSV(points: corePoints)
        }
    }

    private func copyToClipboard() {
        let pasteboard = NSPasteboard.general
        pasteboard.clearContents()
        pasteboard.setString(generatedContent, forType: .string)
        withAnimation {
            copiedFeedback = true
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.8) {
            withAnimation {
                copiedFeedback = false
            }
        }
    }

    private func saveFile() {
        let panel = NSSavePanel()
        panel.canCreateDirectories = true
        switch selectedFormat {
        case .svg:
            panel.allowedContentTypes = [.svg]
            panel.nameFieldStringValue = "plot.svg"
        case .latex:
            panel.allowedContentTypes = [.plainText]
            panel.nameFieldStringValue = "plot.tex"
        case .csv:
            panel.allowedContentTypes = [.commaSeparatedText]
            panel.nameFieldStringValue = "data.csv"
        }

        if panel.runModal() == .OK, let url = panel.url {
            try? generatedContent.write(to: url, atomically: true, encoding: .utf8)
        }
    }
}
