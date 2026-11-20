// CalculusInspector.swift
// Calculus tab for root finding (Brent's method), local extrema, numerical integration (G7K15), and curve intersections.

import SwiftUI

struct CalculusInspector: View {
    @Environment(AppState.self) private var appState

    @State private var selectedExprIndex: Int = 0
    @State private var rootGuess: String = "0.0"
    @State private var rootResult: String?
    @State private var isFindingRoot: Bool = false

    @State private var extremaRangeA: String = "-5.0"
    @State private var extremaRangeB: String = "5.0"
    @State private var extremaResults: [CoreExtremumResult] = []

    @State private var intA: String = "0.0"
    @State private var intB: String = "1.0"
    @State private var integralResult: CoreIntegralResult?

    @State private var isComputing: Bool = false
    @State private var errorMessage: String?

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 18) {
                // Active Expression Selector
                if appState.expressions.isEmpty || appState.expressions.allSatisfy({ $0.source.isEmpty }) {
                    ContentUnavailableView(
                        "No Expressions",
                        systemImage: "function",
                        description: Text("Enter a mathematical expression in Functions mode to inspect calculus operations.")
                    )
                } else {
                    expressionSelector

                    Divider()

                    // 1. Root Finding (Brent's Method)
                    rootFindingSection

                    Divider()

                    // 2. Local Extrema
                    extremaSection

                    Divider()

                    // 3. Adaptive Gauss-Kronrod Definite Integration
                    integrationSection
                }
            }
            .padding(12)
        }
    }

    // MARK: - Expression Selector

    private var expressionSelector: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Target Function")
                .font(.subheadline.bold())

            Picker("Expression", selection: $selectedExprIndex) {
                ForEach(Array(appState.expressions.enumerated()), id: \.offset) { idx, expr in
                    Text(expr.source.isEmpty ? "Expression \(idx + 1)" : expr.source)
                        .tag(idx)
                }
            }
            .pickerStyle(.menu)

            if appState.expressions.indices.contains(selectedExprIndex) {
                let expr = appState.expressions[selectedExprIndex]
                if !expr.source.isEmpty {
                    VStack(alignment: .leading, spacing: 2) {
                        Text("f'(x) = " + GraphPlottingCore.derivative(expr.source))
                            .font(.system(.caption2, design: .monospaced))
                            .foregroundStyle(.secondary)
                        Text("f''(x) = " + GraphPlottingCore.secondDerivative(expr.source))
                            .font(.system(.caption2, design: .monospaced))
                            .foregroundStyle(.secondary)
                    }
                }
            }
        }
    }

    // MARK: - Root Finding Section

    private var rootFindingSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Root Finding (Brent's Method)")
                .font(.subheadline.bold())

            HStack(spacing: 8) {
                TextField("Initial Guess x₀", text: $rootGuess)
                    .textFieldStyle(.roundedBorder)
                    .font(.system(.caption, design: .monospaced))

                Button("Find Root") {
                    calculateRoot()
                }
                .buttonStyle(.glass)
                .disabled(currentExpression == nil || Double(rootGuess) == nil)
            }

            if let result = rootResult {
                HStack {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundStyle(.green)
                    Text(result)
                        .font(.system(.caption, design: .monospaced).bold())
                }
                .padding(6)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(Color.green.opacity(0.1), in: RoundedRectangle(cornerRadius: 6))
            }
        }
    }

    // MARK: - Extrema Section

    private var extremaSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Critical Points & Extrema")
                .font(.subheadline.bold())

            HStack(spacing: 6) {
                Text("Range [a, b]:").font(.caption).foregroundStyle(.secondary)
                TextField("a", text: $extremaRangeA)
                    .textFieldStyle(.roundedBorder)
                    .font(.system(.caption, design: .monospaced))
                    .frame(width: 50)
                TextField("b", text: $extremaRangeB)
                    .textFieldStyle(.roundedBorder)
                    .font(.system(.caption, design: .monospaced))
                    .frame(width: 50)

                Spacer()

                Button("Find Extrema") {
                    calculateExtrema()
                }
                .buttonStyle(.glass)
                .disabled(currentExpression == nil)
            }

            if !extremaResults.isEmpty {
                VStack(alignment: .leading, spacing: 4) {
                    ForEach(Array(extremaResults.enumerated()), id: \.offset) { idx, ext in
                        HStack {
                            Image(systemName: ext.isMinimum ? "arrow.down.circle.fill" : "arrow.up.circle.fill")
                                .foregroundStyle(ext.isMinimum ? .blue : .purple)
                            Text(ext.isMinimum ? "Local Min:" : "Local Max:")
                                .font(.caption.bold())
                            Spacer()
                            Text(String(format: "(%.4f, %.4f)", ext.x, ext.y))
                                .font(.system(.caption, design: .monospaced))
                        }
                    }
                }
                .padding(8)
                .background(Color(NSColor.controlBackgroundColor).opacity(0.6), in: RoundedRectangle(cornerRadius: 6))
            }
        }
    }

    // MARK: - Definite Integration Section

    private var integrationSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Definite Integral ∫ₐᵇ f(x) dx")
                .font(.subheadline.bold())

            HStack(spacing: 6) {
                Text("Bounds:").font(.caption).foregroundStyle(.secondary)
                TextField("a", text: $intA)
                    .textFieldStyle(.roundedBorder)
                    .font(.system(.caption, design: .monospaced))
                    .frame(width: 55)
                TextField("b", text: $intB)
                    .textFieldStyle(.roundedBorder)
                    .font(.system(.caption, design: .monospaced))
                    .frame(width: 55)

                Spacer()

                Button("Integrate") {
                    calculateIntegral()
                }
                .buttonStyle(.glass)
                .disabled(currentExpression == nil || Double(intA) == nil || Double(intB) == nil)
            }

            if let intRes = integralResult {
                VStack(alignment: .leading, spacing: 3) {
                    HStack {
                        Text("Area:")
                            .font(.caption.bold())
                        Spacer()
                        Text(String(format: "%.8f", intRes.value))
                            .font(.system(.caption, design: .monospaced).bold())
                    }
                    HStack {
                        Text("Est. Error:")
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                        Spacer()
                        Text(String(format: "±%.2e", intRes.errorEstimate))
                            .font(.system(.caption2, design: .monospaced))
                            .foregroundStyle(.secondary)
                    }
                }
                .padding(8)
                .background(Color.blue.opacity(0.1), in: RoundedRectangle(cornerRadius: 6))
            }
        }
    }

    // MARK: - Calculations

    private var currentExpression: ExpressionEntry? {
        guard appState.expressions.indices.contains(selectedExprIndex) else { return nil }
        let expr = appState.expressions[selectedExprIndex]
        return expr.source.isEmpty ? nil : expr
    }

    private func calculateRoot() {
        guard let expr = currentExpression,
              let guess = Double(rootGuess),
              let compiled = try? GraphPlottingCore.parseExpression(expr.source) else { return }

        do {
            let res = try GraphPlottingCore.findRoot(
                expression: compiled,
                near: guess,
                xMin: appState.viewport.xMin,
                xMax: appState.viewport.xMax,
                params: expr.parameters.map(\.value)
            )
            rootResult = String(format: "x* = %.8f", res.x)
        } catch {
            rootResult = "No root found near \(guess)"
        }
    }

    private func calculateExtrema() {
        guard let expr = currentExpression,
              let a = Double(extremaRangeA),
              let b = Double(extremaRangeB),
              let compiled = try? GraphPlottingCore.parseExpression(expr.source) else { return }

        extremaResults = GraphPlottingCore.findExtrema(
            expression: compiled,
            xMin: min(a, b),
            xMax: max(a, b),
            params: expr.parameters.map(\.value)
        )
    }

    private func calculateIntegral() {
        guard let expr = currentExpression,
              let a = Double(intA),
              let b = Double(intB),
              let compiled = try? GraphPlottingCore.parseExpression(expr.source) else { return }

        integralResult = try? GraphPlottingCore.integrate(
            expression: compiled,
            from: a,
            to: b,
            params: expr.parameters.map(\.value)
        )
    }
}
