// GraphPlottingApp.swift

import SwiftUI

@main
struct GraphPlottingMainApp: App {
    @State private var appState = AppState()

    init() {
        NSApplication.shared.setActivationPolicy(.regular)
        DispatchQueue.main.async {
            NSApplication.shared.activate(ignoringOtherApps: true)
        }
    }

    var body: some Scene {
        WindowGroup("Graph Plotting 2.0") {
            ContentView()
                .environment(appState)
        }
        .commands {
            CommandGroup(after: .sidebar) {
                Button("Toggle Sidebar") {
                    appState.showSidebar.toggle()
                }
                .keyboardShortcut("k", modifiers: .command)
            }

            CommandMenu("Graph") {
                Button("Fit to Data") {
                    appState.fitToData()
                }
                .keyboardShortcut("f", modifiers: [.command, .option])

                Button("Export…") {
                    appState.showExportSheet = true
                }
                .keyboardShortcut("e", modifiers: [.command, .shift])

                Divider()

                Button("Run Fit") {
                    appState.fitData()
                }
                .keyboardShortcut(.return, modifiers: .command)
                .disabled(appState.points.count < 2)

                Divider()

                Button("Clear All") {
                    appState.clearAll()
                }
            }

            CommandGroup(replacing: .help) {
                Button("Graph Plotting Help") {
                    // Placeholder
                }
            }
        }
    }
}
