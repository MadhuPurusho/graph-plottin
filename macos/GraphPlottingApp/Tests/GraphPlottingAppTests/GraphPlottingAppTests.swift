// GraphPlottingAppTests.swift
// End-to-end Swift testing suite for Graph Plotting 2.0 Rust UniFFI engine integration.

import Testing
import Foundation
@testable import GraphPlottingApp
import GraphPlottingCoreFFI

@Suite("Graph Plotting Core Engine Tests")
struct GraphPlottingCoreTests {

    @Test("Expression Parsing and Bytecode Evaluation")
    func testExpressionParsingAndEvaluation() throws {
        let compiled = try GraphPlottingCore.parseExpression("3x^2 - 2x + 5")
        #expect(compiled.source() == "3x^2 - 2x + 5")

        let val0 = compiled.eval(x: 0.0, params: [])
        #expect(abs(val0 - 5.0) < 1e-9)

        let val1 = compiled.eval(x: 1.0, params: [])
        #expect(abs(val1 - 6.0) < 1e-9)

        let val2 = compiled.eval(x: 2.0, params: [])
        #expect(abs(val2 - 13.0) < 1e-9)
    }

    @Test("Symbolic First and Second Differentiation")
    func testDifferentiation() {
        let d1 = GraphPlottingCore.derivative("x^3")
        #expect(!d1.isEmpty)

        let d2 = GraphPlottingCore.secondDerivative("x^3")
        #expect(!d2.isEmpty)

        let sinDiff = GraphPlottingCore.derivative("sin(x)")
        #expect(!sinDiff.isEmpty)
    }

    @Test("Adaptive Tessellation with Singularity Guards")
    func testTessellation() throws {
        let expr = try GraphPlottingCore.parseExpression("tan(x)")
        let strips = GraphPlottingCore.tessellate(
            expression: expr,
            xMin: -3.0,
            xMax: 3.0,
            yMin: -10.0,
            yMax: 10.0,
            viewportWidth: 800,
            params: []
        )
        // tan(x) has a pole at pi/2 (~1.57), so it must break into multiple strips
        #expect(strips.count >= 2)
        for strip in strips {
            #expect(!strip.points.isEmpty)
            #expect(strip.points.count % 2 == 0)
        }
    }

    @Test("Linear Curve Fitting")
    func testLinearFit() throws {
        let pts = [
            CoreDataPoint(x: 0.0, y: 1.0),
            CoreDataPoint(x: 1.0, y: 3.0),
            CoreDataPoint(x: 2.0, y: 5.0),
            CoreDataPoint(x: 3.0, y: 7.0),
            CoreDataPoint(x: 4.0, y: 9.0)
        ]

        let result = try GraphPlottingCore.fitCurve(points: pts, fitType: .linear)
        #expect(result.fitType == .linear)
        #expect(abs(result.parameters[0] - 2.0) < 1e-5) // slope
        #expect(abs(result.parameters[1] - 1.0) < 1e-5) // intercept
        #expect(result.rSquared > 0.9999)
        #expect(result.rmse < 1e-5)
        #expect(!result.equation.isEmpty)
        #expect(!result.latexEquation.isEmpty)
    }

    @Test("Quadratic Curve Fitting")
    func testQuadraticFit() throws {
        let pts = [
            CoreDataPoint(x: -2.0, y: 4.0),
            CoreDataPoint(x: -1.0, y: 1.0),
            CoreDataPoint(x: 0.0, y: 0.0),
            CoreDataPoint(x: 1.0, y: 1.0),
            CoreDataPoint(x: 2.0, y: 4.0)
        ]

        let result = try GraphPlottingCore.fitCurve(points: pts, fitType: .quadratic)
        #expect(result.fitType == .quadratic)
        #expect(abs(result.parameters[0] - 1.0) < 1e-5) // a
        #expect(abs(result.parameters[1] - 0.0) < 1e-5) // b
        #expect(abs(result.parameters[2] - 0.0) < 1e-5) // c
        #expect(result.rSquared > 0.9999)
    }

    @Test("Smart Auto Fit Leaderboard Ranking")
    func testAutoFit() throws {
        let pts = [
            CoreDataPoint(x: 0.0, y: 1.0),
            CoreDataPoint(x: 1.0, y: 3.0),
            CoreDataPoint(x: 2.0, y: 5.0),
            CoreDataPoint(x: 3.0, y: 7.0),
            CoreDataPoint(x: 4.0, y: 9.0)
        ]

        let candidates = try GraphPlottingCore.autoFit(points: pts)
        #expect(!candidates.isEmpty)
        let winner = candidates[0]
        #expect(winner.fitType == .linear)
        #expect(winner.rSquared > 0.9999)
    }

    @Test("Calculus - Brent's Root Finder")
    func testRootFinding() throws {
        let expr = try GraphPlottingCore.parseExpression("x^2 - 4")
        let root = try GraphPlottingCore.findRoot(
            expression: expr,
            near: 1.8,
            xMin: 0.0,
            xMax: 5.0,
            params: []
        )
        #expect(root.converged)
        #expect(abs(root.x - 2.0) < 1e-6)
    }

    @Test("Calculus - Local Extrema")
    func testLocalExtrema() throws {
        let expr = try GraphPlottingCore.parseExpression("x^2 - 4")
        let extrema = GraphPlottingCore.findExtrema(
            expression: expr,
            xMin: -3.0,
            xMax: 3.0,
            params: []
        )
        #expect(!extrema.isEmpty)
        let minExt = extrema.first { $0.isMinimum }
        #expect(minExt != nil)
        if let m = minExt {
            #expect(abs(m.x - 0.0) < 1e-4)
            #expect(abs(m.y - (-4.0)) < 1e-4)
        }
    }

    @Test("Calculus - Adaptive Gauss-Kronrod Integration")
    func testNumericalIntegration() throws {
        let expr = try GraphPlottingCore.parseExpression("x^2")
        let integral = try GraphPlottingCore.integrate(
            expression: expr,
            from: 0.0,
            to: 1.0,
            params: []
        )
        // Exact integral is 1/3 = 0.33333333...
        #expect(abs(integral.value - 1.0 / 3.0) < 1e-7)
    }

    @Test("CSV Parsing and Export Roundtrip")
    func testCSVIORoundtrip() {
        let csvText = """
        x,y
        0.5,1.25
        1.5,3.75
        2.5,6.25
        """

        let parsed = GraphPlottingCore.parseCSV(csvText)
        #expect(parsed.count == 3)
        #expect(abs(parsed[0].x - 0.5) < 1e-6)
        #expect(abs(parsed[0].y - 1.25) < 1e-6)

        let exported = GraphPlottingCore.exportCSV(points: parsed)
        #expect(exported.contains("0.5"))
        #expect(exported.contains("1.25"))
    }

    @Test("Publication Vector SVG and LaTeX Exporters")
    func testSVGAndLaTeXExport() {
        let pts = [
            CoreDataPoint(x: 0.0, y: 1.0),
            CoreDataPoint(x: 1.0, y: 2.0)
        ]

        let svg = GraphPlottingCore.exportSVG(
            strips: [],
            points: pts,
            equationUnicode: "y = x + 1",
            rSquared: 1.0,
            xMin: -2.0,
            xMax: 4.0,
            yMin: -2.0,
            yMax: 6.0
        )
        #expect(svg.contains("<svg"))
        #expect(svg.contains("y = x + 1"))
        #expect(svg.contains("</svg>"))

        let latex = GraphPlottingCore.exportLaTeX(
            equationLatex: "y = x + 1",
            points: pts,
            rSquared: 1.0,
            xMin: -2.0,
            xMax: 4.0,
            yMin: -2.0,
            yMax: 6.0
        )
        #expect(latex.contains("\\begin{tikzpicture}"))
        #expect(latex.contains("\\begin{axis}"))
        #expect(latex.contains("\\end{tikzpicture}"))
    }
}
