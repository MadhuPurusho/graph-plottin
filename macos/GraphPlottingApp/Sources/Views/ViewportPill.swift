// ViewportPill.swift
// Liquid Glass floating pill displaying viewport coordinates and fast navigation actions.

import SwiftUI

struct ViewportPill: View {
    @Environment(AppState.self) private var appState

    var body: some View {
        HStack(spacing: 8) {
            Text(appState.viewport.formattedRange)
                .font(.system(.caption, design: .monospaced))
                .foregroundStyle(.secondary)
                .padding(.leading, 4)

            Divider()
                .frame(height: 14)

            Button {
                appState.viewport.zoom(by: 0.8)
                appState.needsRedraw = true
            } label: {
                Image(systemName: "plus.magnifyingglass")
            }
            .buttonStyle(.plain)
            .help("Zoom In")

            Button {
                appState.viewport.zoom(by: 1.25)
                appState.needsRedraw = true
            } label: {
                Image(systemName: "minus.magnifyingglass")
            }
            .buttonStyle(.plain)
            .help("Zoom Out")

            Button {
                appState.viewport.reset()
                appState.needsRedraw = true
            } label: {
                Image(systemName: "arrow.counterclockwise")
            }
            .buttonStyle(.plain)
            .help("Reset Viewport (-10 to 10)")

            if !appState.points.isEmpty {
                Button {
                    appState.fitToData()
                } label: {
                    Image(systemName: "arrow.up.left.and.arrow.down.right")
                }
                .buttonStyle(.plain)
                .help("Fit to Data")
            }
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .glassEffect(.regular, in: Capsule())
    }
}
