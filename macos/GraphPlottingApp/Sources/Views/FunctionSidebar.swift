// FunctionSidebar.swift

import SwiftUI

struct FunctionSidebar: View {
    @Environment(AppState.self) private var appState

    var body: some View {
        VStack(spacing: 0) {
            // Header
            HStack {
                Text("Expressions")
                    .font(.headline)
                Spacer()
                Button {
                    appState.addExpression()
                } label: {
                    Image(systemName: "plus")
                }
                .buttonStyle(.glass)
            }
            .padding(12)

            Divider()

            // Expression list
            ScrollView {
                LazyVStack(spacing: 12) {
                    ForEach(Array(appState.expressions.enumerated()), id: \.element.id) { index, expression in
                        ExpressionRow(index: index, expression: expression)
                            .environment(appState)
                    }
                }
                .padding(12)
            }

            Spacer()
        }
    }
}

// MARK: - Expression Row

struct ExpressionRow: View {
    @Environment(AppState.self) private var appState
    let index: Int
    let expression: ExpressionEntry

    var body: some View {
        @Bindable var state = appState

        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 8) {
                // Color indicator
                Circle()
                    .fill(expression.color)
                    .frame(width: 10, height: 10)

                // Expression text field
                TextField(
                    "f(x) = ...",
                    text: Binding(
                        get: { appState.expressions[safe: index]?.source ?? "" },
                        set: { newValue in
                            if appState.expressions.indices.contains(index) {
                                appState.expressions[index].source = newValue
                                appState.needsRedraw = true
                            }
                        }
                    )
                )
                .textFieldStyle(.roundedBorder)
                .font(.system(.body, design: .monospaced))

                // Visibility toggle
                Button {
                    if appState.expressions.indices.contains(index) {
                        appState.expressions[index].isVisible.toggle()
                        appState.needsRedraw = true
                    }
                } label: {
                    Image(systemName: expression.isVisible ? "eye" : "eye.slash")
                        .foregroundStyle(expression.isVisible ? .primary : .tertiary)
                }
                .buttonStyle(.borderless)

                // Delete
                Button {
                    appState.removeExpression(id: expression.id)
                } label: {
                    Image(systemName: "xmark.circle")
                        .foregroundStyle(.secondary)
                }
                .buttonStyle(.borderless)
                .disabled(appState.expressions.count <= 1)
            }

            // Derivative display
            if !expression.source.isEmpty {
                VStack(alignment: .leading, spacing: 2) {
                    let deriv = GraphPlottingCore.derivative(expression.source)
                    Text("f'(x) = \(deriv)")
                        .font(.system(.caption, design: .monospaced))
                        .foregroundStyle(.secondary)
                }
            }

            // Parameter sliders
            if !expression.parameters.isEmpty {
                VStack(spacing: 4) {
                    ForEach(Array(expression.parameters.enumerated()), id: \.element.id) { pIdx, param in
                        HStack {
                            Text(param.name)
                                .font(.system(.caption, design: .monospaced))
                                .frame(width: 20, alignment: .trailing)

                            Slider(
                                value: Binding(
                                    get: { appState.expressions[safe: index]?.parameters[safe: pIdx]?.value ?? param.value },
                                    set: { newValue in
                                        if appState.expressions.indices.contains(index),
                                           appState.expressions[index].parameters.indices.contains(pIdx) {
                                            appState.expressions[index].parameters[pIdx].value = newValue
                                            appState.needsRedraw = true
                                        }
                                    }
                                ),
                                in: param.min...param.max,
                                step: param.step
                            )

                            Text(String(format: "%.2f", param.value))
                                .font(.system(.caption, design: .monospaced))
                                .frame(width: 45, alignment: .trailing)
                        }
                    }
                }
            }

            // Line width control
            HStack {
                Text("Width")
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
                Slider(
                    value: Binding(
                        get: { appState.expressions[safe: index]?.lineWidth ?? 2.0 },
                        set: { newValue in
                            if appState.expressions.indices.contains(index) {
                                appState.expressions[index].lineWidth = newValue
                                appState.needsRedraw = true
                            }
                        }
                    ),
                    in: 0.5...6.0,
                    step: 0.5
                )
                .controlSize(.mini)

                ColorPicker(
                    "",
                    selection: Binding(
                        get: { appState.expressions[safe: index]?.color ?? .blue },
                        set: { newValue in
                            if appState.expressions.indices.contains(index) {
                                appState.expressions[index].color = newValue
                                appState.needsRedraw = true
                            }
                        }
                    )
                )
                .labelsHidden()
                .frame(width: 24)
            }
        }
        .padding(10)
        .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 8))
    }
}

// MARK: - Safe Array Subscript

extension Array {
    subscript(safe index: Index) -> Element? {
        indices.contains(index) ? self[index] : nil
    }
}
