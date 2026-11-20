// EquationOverlay.swift
// Draggable Liquid Glass card displaying active fitted equation, exact R², and copy actions.

import SwiftUI
import AppKit

struct EquationOverlay: View {
    @Environment(AppState.self) private var appState
    @State private var dragOffset: CGSize = .zero
    @State private var accumulatedOffset: CGSize = .zero
    @State private var copiedFeedback: String?

    var body: some View {
        if appState.mode == .plotPoints,
           let fit = appState.fitResults.first(where: { $0.id == appState.activeFitID }) {
            VStack(alignment: .leading, spacing: 6) {
                HStack {
                    Text(fit.fitType.description)
                        .font(.caption.bold())
                        .foregroundStyle(.secondary)

                    Spacer()

                    if let feedback = copiedFeedback {
                        Text(feedback)
                            .font(.caption2.bold())
                            .foregroundStyle(.green)
                            .transition(.opacity)
                    }
                }

                // Fitted Unicode Equation
                Text(fit.equation)
                    .font(.system(.body, design: .monospaced))
                    .textSelection(.enabled)

                // Statistical goodness of fit metrics
                HStack(spacing: 12) {
                    HStack(spacing: 4) {
                        Text("R²:")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                        Text(fit.formattedR2)
                            .font(.system(.caption, design: .monospaced).bold())
                    }

                    HStack(spacing: 4) {
                        Text("RMSE:")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                        Text(fit.formattedRMSE)
                            .font(.system(.caption, design: .monospaced))
                    }

                    HStack(spacing: 4) {
                        Text("AICc:")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                        Text(fit.formattedAICc)
                            .font(.system(.caption, design: .monospaced))
                    }
                }

                Divider()

                HStack(spacing: 8) {
                    Button {
                        copyToClipboard(fit.equation, label: "Copied Equation")
                    } label: {
                        Label("Copy", systemImage: "doc.on.doc")
                            .font(.caption)
                    }
                    .buttonStyle(.plain)

                    Button {
                        copyToClipboard(fit.latexEquation, label: "Copied LaTeX")
                    } label: {
                        Label("LaTeX", systemImage: "function")
                            .font(.caption)
                    }
                    .buttonStyle(.plain)

                    Spacer()

                    if fit.rSquared >= 0.999 {
                        HStack(spacing: 3) {
                            Image(systemName: "checkmark.seal.fill")
                                .foregroundStyle(.green)
                            Text("Optimal Fit")
                                .font(.caption2)
                                .foregroundStyle(.secondary)
                        }
                    }
                }
            }
            .padding(12)
            .frame(minWidth: 260, maxWidth: 360)
            .glassEffect(.regular, in: RoundedRectangle(cornerRadius: 14, style: .continuous))
            .offset(x: accumulatedOffset.width + dragOffset.width, y: accumulatedOffset.height + dragOffset.height)
            .gesture(
                DragGesture()
                    .onChanged { value in
                        dragOffset = value.translation
                    }
                    .onEnded { value in
                        accumulatedOffset.width += value.translation.width
                        accumulatedOffset.height += value.translation.height
                        dragOffset = .zero
                    }
            )
        }
    }

    private func copyToClipboard(_ text: String, label: String) {
        let pasteboard = NSPasteboard.general
        pasteboard.clearContents()
        pasteboard.setString(text, forType: .string)
        withAnimation {
            copiedFeedback = label
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.8) {
            withAnimation {
                copiedFeedback = nil
            }
        }
    }
}
