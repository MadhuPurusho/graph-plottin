// GraphPlottingCoreBridge.swift
// Native Swift bridge delegating directly to the 100% Rust UniFFI engine.

import Foundation

// MARK: - FitType Extensions

extension FitType: CaseIterable, Identifiable, CustomStringConvertible {
    public var id: String { String(describing: self) }

    public static var allCases: [FitType] {
        [
            .auto, .linear, .quadratic, .cubic, .degree4, .degree5,
            .exponential, .logarithmic, .power, .sinusoidal,
            .cosinusoidal, .tangential, .hyperbolic, .squareRoot,
            .absoluteValue, .gaussian
        ]
    }

    public var description: String {
        switch self {
        case .auto: return "Auto (Smart Fit)"
        case .linear: return "Linear (y = mx + b)"
        case .quadratic: return "Quadratic (y = ax² + bx + c)"
        case .cubic: return "Cubic (y = ax³ + bx² + cx + d)"
        case .degree4: return "Quartic (Degree 4)"
        case .degree5: return "Quintic (Degree 5)"
        case .exponential: return "Exponential (y = ae^{bx})"
        case .logarithmic: return "Logarithmic (y = a + b ln x)"
        case .power: return "Power (y = ax^b)"
        case .sinusoidal: return "Sinusoidal (y = a sin(bx + c) + d)"
        case .cosinusoidal: return "Cosinusoidal (y = a cos(bx + c) + d)"
        case .tangential: return "Tangential (y = a tan(bx + c) + d)"
        case .hyperbolic: return "Hyperbolic (y = a/(x - h) + k)"
        case .squareRoot: return "Square Root (y = a√(x - h) + k)"
        case .absoluteValue: return "Absolute Value (y = a|x - h| + k)"
        case .gaussian: return "Gaussian (y = a e^{-(x-μ)²/(2σ²)} + c)"
        }
    }
}

// MARK: - Core Bridge Namespace

enum GraphPlottingCore {

    // MARK: Expression Parsing & Differentiation

    static func parseExpression(_ input: String) throws -> CompiledExpression {
        try GraphPlottingApp.parseExpression(input: input)
    }

    static func formatExpression(_ input: String) -> String {
        (try? GraphPlottingApp.formatExpression(input: input)) ?? input
    }

    static func derivative(_ input: String) -> String {
        (try? GraphPlottingApp.derivative(input: input)) ?? "d/dx(\(input))"
    }

    static func secondDerivative(_ input: String) -> String {
        (try? GraphPlottingApp.secondDerivative(input: input)) ?? "d²/dx²(\(input))"
    }

    // MARK: Tessellation

    static func tessellate(
        expression: CompiledExpression,
        xMin: Double,
        xMax: Double,
        yMin: Double = -100.0,
        yMax: Double = 100.0,
        viewportWidth: Int = 1000,
        params: [Double] = []
    ) -> [PolylineStrip] {
        GraphPlottingApp.tessellate(
            expr: expression,
            xMin: xMin,
            xMax: xMax,
            yMin: yMin,
            yMax: yMax,
            viewportWidth: UInt32(max(viewportWidth, 200)),
            params: params
        )
    }

    // MARK: Curve Fitting

    static func fitCurve(
        points: [CoreDataPoint],
        fitType: FitType,
        useRobust: Bool = false
    ) throws -> CoreFitResult {
        try GraphPlottingApp.fitCurve(points: points, fitType: fitType, useRobust: useRobust)
    }

    static func autoFit(
        points: [CoreDataPoint],
        useRobust: Bool = false
    ) throws -> [CoreFitResult] {
        let bundle = try GraphPlottingApp.autoFit(points: points, useRobust: useRobust)
        return bundle.candidates
    }

    // MARK: Calculus

    static func findRoot(
        expression: CompiledExpression,
        near guess: Double,
        xMin: Double,
        xMax: Double,
        params: [Double] = []
    ) throws -> CoreRootResult {
        try GraphPlottingApp.findRoot(expr: expression, guess: guess, xMin: xMin, xMax: xMax, params: params)
    }

    static func findExtrema(
        expression: CompiledExpression,
        xMin: Double,
        xMax: Double,
        params: [Double] = []
    ) -> [CoreExtremumResult] {
        GraphPlottingApp.findExtrema(expr: expression, xMin: xMin, xMax: xMax, params: params)
    }

    static func findIntersections(
        expr1: CompiledExpression,
        expr2: CompiledExpression,
        xMin: Double,
        xMax: Double,
        params: [Double] = []
    ) -> [Double] {
        GraphPlottingApp.findIntersections(expr1: expr1, expr2: expr2, xMin: xMin, xMax: xMax, params: params)
    }

    static func integrate(
        expression: CompiledExpression,
        from a: Double,
        to b: Double,
        params: [Double] = []
    ) throws -> CoreIntegralResult {
        try GraphPlottingApp.integrate(expr: expression, a: a, b: b, params: params)
    }

    // MARK: I/O

    static func parseCSV(_ content: String) -> [CoreDataPoint] {
        (try? GraphPlottingApp.parseCsv(content: content)) ?? []
    }

    static func exportCSV(points: [CoreDataPoint]) -> String {
        GraphPlottingApp.exportCsv(points: points)
    }

    // MARK: Export

    static func exportSVG(
        strips: [PolylineStrip] = [],
        points: [CoreDataPoint] = [],
        confUpper: [CoreDataPoint] = [],
        confLower: [CoreDataPoint] = [],
        equationUnicode: String = "",
        rSquared: Double? = nil,
        xMin: Double = -10.0,
        xMax: Double = 10.0,
        yMin: Double = -10.0,
        yMax: Double = 10.0,
        width: Int = 800,
        height: Int = 600
    ) -> String {
        GraphPlottingApp.exportSvg(
            strips: strips,
            points: points,
            confUpper: confUpper,
            confLower: confLower,
            equationUnicode: equationUnicode,
            rSquared: rSquared,
            xMin: xMin,
            xMax: xMax,
            yMin: yMin,
            yMax: yMax,
            width: UInt32(width),
            height: UInt32(height)
        )
    }

    static func exportLaTeX(
        equationLatex: String,
        points: [CoreDataPoint],
        rSquared: Double? = nil,
        xMin: Double = -10.0,
        xMax: Double = 10.0,
        yMin: Double = -10.0,
        yMax: Double = 10.0
    ) -> String {
        GraphPlottingApp.exportLatex(
            equationLatex: equationLatex,
            points: points,
            rSquared: rSquared,
            xMin: xMin,
            xMax: xMax,
            yMin: yMin,
            yMax: yMax
        )
    }
}
