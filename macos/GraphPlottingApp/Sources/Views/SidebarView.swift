// SidebarView.swift

import SwiftUI

struct SidebarView: View {
    @Environment(AppState.self) private var appState

    var body: some View {
        Group {
            switch appState.mode {
            case .plotPoints:
                PlotPointsSidebar()
                    .environment(appState)
            case .functions:
                FunctionSidebar()
                    .environment(appState)
            }
        }
        .navigationSplitViewColumnWidth(min: 280, ideal: 320, max: 450)
    }
}
