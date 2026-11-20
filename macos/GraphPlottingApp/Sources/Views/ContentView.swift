// ContentView.swift

import SwiftUI

struct ContentView: View {
    @Environment(AppState.self) private var appState

    var body: some View {
        @Bindable var state = appState

        NavigationSplitView(
            columnVisibility: Binding(
                get: { state.showSidebar ? .all : .detailOnly },
                set: { state.showSidebar = ($0 != .detailOnly) }
            )
        ) {
            SidebarView()
        } detail: {
            ZStack {
                MetalGraphView()
                    .frame(minWidth: 500, minHeight: 400)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                    .ignoresSafeArea()

                overlays
            }
            .inspector(isPresented: $state.showInspector) {
                inspectorContent
                    .inspectorColumnWidth(min: 260, ideal: 300, max: 400)
            }
        }
        .navigationSplitViewStyle(.balanced)
        .frame(minWidth: 1050, idealWidth: 1280, minHeight: 650, idealHeight: 800)
        .sheet(isPresented: $state.showExportSheet) {
            ExportView()
                .environment(appState)
        }
        .onDrop(of: [.fileURL, .commaSeparatedText], isTargeted: nil) { providers in
            handleDrop(providers)
        }
        .toolbar {
            toolbarContent
        }
        .background(WindowAccessor())
    }

    // MARK: - Overlays

    @ViewBuilder
    private var overlays: some View {
        GlassEffectContainer {
            VStack {
                HStack {
                    Spacer()
                    ViewportPill()
                        .environment(appState)
                }
                .padding(.top, 8)
                .padding(.trailing, 12)

                Spacer()

                HStack(alignment: .bottom) {
                    EquationOverlay()
                        .environment(appState)

                    Spacer()

                    if appState.showProbe {
                        ProbeHUD()
                            .environment(appState)
                    }
                }
                .padding(.horizontal, 12)
                .padding(.bottom, 12)
            }
        }
    }

    // MARK: - Inspector

    @ViewBuilder
    private var inspectorContent: some View {
        @Bindable var state = appState
        TabView(selection: $state.inspectorTab) {
            Tab("Diagnostics", systemImage: "chart.bar.doc.horizontal", value: AppState.InspectorTab.diagnostics) {
                DiagnosticsInspector()
                    .environment(appState)
            }
            Tab("Calculus", systemImage: "function", value: AppState.InspectorTab.calculus) {
                CalculusInspector()
                    .environment(appState)
            }
        }
    }

    // MARK: - Toolbar

    @ToolbarContentBuilder
    private var toolbarContent: some ToolbarContent {
        ToolbarItemGroup(placement: .primaryAction) {
            Picker("Mode", selection: Binding(
                get: { appState.mode },
                set: { appState.mode = $0 }
            )) {
                Text("Plot Points").tag(AppMode.plotPoints)
                Text("Functions").tag(AppMode.functions)
            }
            .pickerStyle(.segmented)
            .frame(width: 200)
        }

        ToolbarItemGroup(placement: .secondaryAction) {
            Button {
                appState.showInspector.toggle()
            } label: {
                Label("Inspector", systemImage: "sidebar.right")
            }

            Button {
                appState.fitToData()
            } label: {
                Label("Fit to Data", systemImage: "arrow.up.left.and.arrow.down.right")
            }
            .disabled(appState.points.isEmpty)
        }
    }

    // MARK: - Drop Handling

    private func handleDrop(_ providers: [NSItemProvider]) -> Bool {
        for provider in providers {
            if provider.hasItemConformingToTypeIdentifier("public.file-url") {
                provider.loadItem(forTypeIdentifier: "public.file-url", options: nil) { item, _ in
                    guard let data = item as? Data,
                          let url = URL(dataRepresentation: data, relativeTo: nil, isAbsolute: true) else { return }
                    if let content = try? String(contentsOf: url, encoding: .utf8) {
                        Task { @MainActor in
                            appState.importCSV(content)
                        }
                    }
                }
                return true
            }
        }
        return false
    }
}

private struct WindowAccessor: NSViewRepresentable {
    func makeNSView(context: Context) -> NSView {
        let view = NSView()
        DispatchQueue.main.async {
            if let window = view.window {
                window.setContentSize(NSSize(width: 1280, height: 800))
                window.center()
                window.makeKeyAndOrderFront(nil)
            }
        }
        return view
    }
    func updateNSView(_ nsView: NSView, context: Context) {}
}
