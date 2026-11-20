// ProbeHUD.swift
// Cursor-tracked magnetic probe HUD displaying snapped coordinates.

import SwiftUI

struct ProbeHUD: View {
    @Environment(AppState.self) private var appState

    var body: some View {
        if let x = appState.probeX, let y = appState.probeY {
            HStack(spacing: 8) {
                Image(systemName: "scope")
                    .font(.caption)
                    .foregroundStyle(.blue)

                HStack(spacing: 4) {
                    Text("x:")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                    Text(String(format: "%.4f", x))
                        .font(.system(.caption, design: .monospaced).bold())
                }

                HStack(spacing: 4) {
                    Text("y:")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                    Text(String(format: "%.4f", y))
                        .font(.system(.caption, design: .monospaced).bold())
                }
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 5)
            .glassEffect(.regular, in: Capsule())
            .transition(.opacity.combined(with: .scale(scale: 0.95)))
        }
    }
}
