// GraphPlottingApp.swift

import SwiftUI

@main
struct GraphPlottingMainApp: App {
    @State private var appState = AppState()

    init() {
        NSApplication.shared.setActivationPolicy(.regular)
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
            NSApplication.shared.activate(ignoringOtherApps: true)
            if let window = NSApplication.shared.windows.first {
                window.setContentSize(NSSize(width: 1280, height: 800))
                window.center()
                window.makeKeyAndOrderFront(nil)
            }
        }
    }

    var body: some Scene {
        WindowGroup("Graph Plotting 2.0") {
            ContentView()
                .environment(appState)
                .frame(minWidth: 1000, minHeight: 650)
        }
        .defaultSize(width: 1280, height: 800)
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
