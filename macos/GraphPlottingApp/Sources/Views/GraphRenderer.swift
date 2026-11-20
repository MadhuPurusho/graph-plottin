// GraphRenderer.swift
// 120 FPS Metal rendering coordinator executing tessellated curve strokes, grid shaders, and instanced markers.

import MetalKit
import SwiftUI

final class GraphRenderer: NSObject, MTKViewDelegate, @unchecked Sendable {
    weak var appState: AppState?
    weak var mtkView: MTKView?

    private var device: MTLDevice?
    private var commandQueue: MTLCommandQueue?

    private var gridPipeline: MTLRenderPipelineState?
    private var curvePipeline: MTLRenderPipelineState?
    private var areaPipeline: MTLRenderPipelineState?
    private var pointPipeline: MTLRenderPipelineState?

    // Compiled expression cache: (id, source) -> CompiledExpression
    private var compiledCache: [UUID: (source: String, compiled: CompiledExpression)] = [:]

    struct Uniforms {
        var viewport_min: SIMD2<Float>
        var viewport_max: SIMD2<Float>
        var screen_size: SIMD2<Float>
        var _padding: SIMD2<Float> = .zero
        var axis_color: SIMD4<Float> = SIMD4<Float>(0.25, 0.28, 0.35, 0.85)
        var grid_color: SIMD4<Float> = SIMD4<Float>(0.65, 0.68, 0.75, 0.40)
        var subgrid_color: SIMD4<Float> = SIMD4<Float>(0.82, 0.84, 0.88, 0.22)
    }

    @MainActor
    func setup(device: MTLDevice, view: MTKView) {
        self.device = device
        self.mtkView = view
        self.commandQueue = device.makeCommandQueue()

        do {
            let library = try device.makeLibrary(source: MetalShaderSource.source, options: nil)

            // 1. Grid Pipeline
            let gridDesc = MTLRenderPipelineDescriptor()
            gridDesc.vertexFunction = library.makeFunction(name: "grid_vertex")
            gridDesc.fragmentFunction = library.makeFunction(name: "grid_fragment")
            gridDesc.colorAttachments[0].pixelFormat = view.colorPixelFormat
            self.gridPipeline = try device.makeRenderPipelineState(descriptor: gridDesc)

            // 2. Curve Pipeline
            let curveDesc = MTLRenderPipelineDescriptor()
            curveDesc.vertexFunction = library.makeFunction(name: "curve_vertex")
            curveDesc.fragmentFunction = library.makeFunction(name: "curve_fragment")
            curveDesc.colorAttachments[0].pixelFormat = view.colorPixelFormat
            curveDesc.colorAttachments[0].isBlendingEnabled = true
            curveDesc.colorAttachments[0].sourceRGBBlendFactor = .sourceAlpha
            curveDesc.colorAttachments[0].destinationRGBBlendFactor = .oneMinusSourceAlpha
            self.curvePipeline = try device.makeRenderPipelineState(descriptor: curveDesc)

            // 3. Area Pipeline (Confidence bands, integrals)
            let areaDesc = MTLRenderPipelineDescriptor()
            areaDesc.vertexFunction = library.makeFunction(name: "area_vertex")
            areaDesc.fragmentFunction = library.makeFunction(name: "area_fragment")
            areaDesc.colorAttachments[0].pixelFormat = view.colorPixelFormat
            areaDesc.colorAttachments[0].isBlendingEnabled = true
            areaDesc.colorAttachments[0].sourceRGBBlendFactor = .sourceAlpha
            areaDesc.colorAttachments[0].destinationRGBBlendFactor = .oneMinusSourceAlpha
            self.areaPipeline = try device.makeRenderPipelineState(descriptor: areaDesc)

            // 4. Point Pipeline (Instanced Circles)
            let pointDesc = MTLRenderPipelineDescriptor()
            pointDesc.vertexFunction = library.makeFunction(name: "point_vertex")
            pointDesc.fragmentFunction = library.makeFunction(name: "point_fragment")
            pointDesc.colorAttachments[0].pixelFormat = view.colorPixelFormat
            pointDesc.colorAttachments[0].isBlendingEnabled = true
            pointDesc.colorAttachments[0].sourceRGBBlendFactor = .sourceAlpha
            pointDesc.colorAttachments[0].destinationRGBBlendFactor = .oneMinusSourceAlpha
            self.pointPipeline = try device.makeRenderPipelineState(descriptor: pointDesc)

        } catch {
            print("Failed to initialize Metal pipelines:", error)
        }
    }

    func mtkView(_ view: MTKView, drawableSizeWillChange size: CGSize) {
        view.setNeedsDisplay(view.bounds)
    }

    func draw(in view: MTKView) {
        guard let appState = self.appState,
              let device = self.device,
              let commandQueue = self.commandQueue,
              let commandBuffer = commandQueue.makeCommandBuffer(),
              let renderPassDesc = view.currentRenderPassDescriptor,
              let drawable = view.currentDrawable else {
            return
        }

        let vp = appState.viewport
        let drawableW = Float(view.drawableSize.width)
        let drawableH = Float(view.drawableSize.height)
        guard drawableW > 0 && drawableH > 0 else { return }

        var uniforms = Uniforms(
            viewport_min: SIMD2<Float>(Float(vp.xMin), Float(vp.yMin)),
            viewport_max: SIMD2<Float>(Float(vp.xMax), Float(vp.yMax)),
            screen_size: SIMD2<Float>(drawableW, drawableH)
        )

        guard let encoder = commandBuffer.makeRenderCommandEncoder(descriptor: renderPassDesc) else {
            return
        }

        // ─────────────────────────────────────────────────────────────────────
        // 1. Draw Grid
        // ─────────────────────────────────────────────────────────────────────
        if let gridPipeline = self.gridPipeline {
            encoder.setRenderPipelineState(gridPipeline)
            encoder.setVertexBytes(&uniforms, length: MemoryLayout<Uniforms>.stride, index: 0)
            encoder.setFragmentBytes(&uniforms, length: MemoryLayout<Uniforms>.stride, index: 0)
            encoder.drawPrimitives(type: .triangleStrip, vertexStart: 0, vertexCount: 4)
        }

        // ─────────────────────────────────────────────────────────────────────
        // 2. Draw Plot Points Mode (Data points, Confidence Bands, Fitted Curves)
        // ─────────────────────────────────────────────────────────────────────
        if appState.mode == .plotPoints {
            // A. Confidence / Prediction Bands
            if let activeFit = appState.fitResults.first(where: { $0.id == appState.activeFitID }),
               let areaPipeline = self.areaPipeline {
                if appState.showConfidenceBands && !activeFit.confidenceBandUpper.isEmpty && !activeFit.confidenceBandLower.isEmpty {
                    drawBand(
                        upper: activeFit.confidenceBandUpper,
                        lower: activeFit.confidenceBandLower,
                        color: SIMD4<Float>(0.2, 0.55, 0.95, 0.22),
                        encoder: encoder,
                        pipeline: areaPipeline,
                        uniforms: &uniforms,
                        device: device
                    )
                }

                if appState.showPredictionBands && !activeFit.predictionBandUpper.isEmpty && !activeFit.predictionBandLower.isEmpty {
                    drawBand(
                        upper: activeFit.predictionBandUpper,
                        lower: activeFit.predictionBandLower,
                        color: SIMD4<Float>(0.4, 0.4, 0.5, 0.12),
                        encoder: encoder,
                        pipeline: areaPipeline,
                        uniforms: &uniforms,
                        device: device
                    )
                }
            }

            // B. Fitted Curve
            if let activeFit = appState.fitResults.first(where: { $0.id == appState.activeFitID }),
               let curvePipeline = self.curvePipeline {
                drawFittedCurve(
                    fit: activeFit,
                    xMin: vp.xMin,
                    xMax: vp.xMax,
                    samples: 600,
                    encoder: encoder,
                    pipeline: curvePipeline,
                    uniforms: &uniforms,
                    device: device
                )
            }

            // C. Data Points (Instanced Circles)
            if !appState.points.isEmpty, let pointPipeline = self.pointPipeline {
                let coords: [SIMD2<Float>] = appState.points.map { SIMD2<Float>(Float($0.x), Float($0.y)) }
                if let buffer = device.makeBuffer(bytes: coords, length: MemoryLayout<SIMD2<Float>>.stride * coords.count, options: .storageModeShared) {
                    encoder.setRenderPipelineState(pointPipeline)
                    encoder.setVertexBuffer(buffer, offset: 0, index: 0)
                    encoder.setVertexBytes(&uniforms, length: MemoryLayout<Uniforms>.stride, index: 1)

                    var ptColor = SIMD4<Float>(0.15, 0.45, 0.92, 1.0)
                    var ptRadius: Float = 5.5
                    encoder.setVertexBytes(&ptColor, length: MemoryLayout<SIMD4<Float>>.stride, index: 2)
                    encoder.setVertexBytes(&ptRadius, length: MemoryLayout<Float>.stride, index: 3)

                    encoder.drawPrimitives(type: .triangleStrip, vertexStart: 0, vertexCount: 4, instanceCount: coords.count)
                }
            }
        }

        // ─────────────────────────────────────────────────────────────────────
        // 3. Draw Functions Mode (Expressions & Tessellated Strips)
        // ─────────────────────────────────────────────────────────────────────
        if appState.mode == .functions, let curvePipeline = self.curvePipeline {
            encoder.setRenderPipelineState(curvePipeline)
            encoder.setVertexBytes(&uniforms, length: MemoryLayout<Uniforms>.stride, index: 1)

            for expr in appState.expressions where expr.isVisible && !expr.source.trimmingCharacters(in: .whitespaces).isEmpty {
                // Get or compile expression
                let compiled: CompiledExpression
                if let cached = compiledCache[expr.id], cached.source == expr.source {
                    compiled = cached.compiled
                } else if let parsed = try? GraphPlottingCore.parseExpression(expr.source) {
                    compiled = parsed
                    compiledCache[expr.id] = (expr.source, parsed)
                } else {
                    continue
                }

                // Tessellate with interval arithmetic
                let strips = GraphPlottingCore.tessellate(
                    expression: compiled,
                    xMin: vp.xMin,
                    xMax: vp.xMax,
                    yMin: vp.yMin,
                    yMax: vp.yMax,
                    viewportWidth: Int(drawableW),
                    params: expr.parameters.map(\.value)
                )

                // Draw each strip
                var strokeColor = SIMD4<Float>(0.2, 0.6, 0.9, 1.0)
                if let components = NSColor(expr.color).usingColorSpace(.deviceRGB) {
                    strokeColor = SIMD4<Float>(Float(components.redComponent), Float(components.greenComponent), Float(components.blueComponent), 1.0)
                }
                encoder.setFragmentBytes(&strokeColor, length: MemoryLayout<SIMD4<Float>>.stride, index: 0)

                for strip in strips {
                    let ptCount = strip.points.count / 2
                    guard ptCount >= 2 else { continue }

                    let byteLen = MemoryLayout<Float>.stride * strip.points.count
                    strip.points.withUnsafeBytes { rawPtr in
                        encoder.setVertexBytes(rawPtr.baseAddress!, length: byteLen, index: 0)
                        encoder.drawPrimitives(type: .lineStrip, vertexStart: 0, vertexCount: ptCount)
                    }
                }
            }
        }

        encoder.endEncoding()
        commandBuffer.present(drawable)
        commandBuffer.commit()
    }

    // MARK: - Band Drawing Helper

    private func drawBand(
        upper: [DataPoint],
        lower: [DataPoint],
        color: SIMD4<Float>,
        encoder: MTLRenderCommandEncoder,
        pipeline: MTLRenderPipelineState,
        uniforms: inout Uniforms,
        device: MTLDevice
    ) {
        let count = min(upper.count, lower.count)
        guard count >= 2 else { return }

        // Build triangle strip: (upper[0], lower[0], upper[1], lower[1], ...)
        var vertices: [SIMD2<Float>] = []
        vertices.reserveCapacity(count * 2)
        for i in 0..<count {
            vertices.append(SIMD2<Float>(Float(upper[i].x), Float(upper[i].y)))
            vertices.append(SIMD2<Float>(Float(lower[i].x), Float(lower[i].y)))
        }

        let byteLength = MemoryLayout<SIMD2<Float>>.stride * vertices.count
        guard let buffer = device.makeBuffer(bytes: vertices, length: byteLength, options: .storageModeShared) else { return }

        encoder.setRenderPipelineState(pipeline)
        encoder.setVertexBuffer(buffer, offset: 0, index: 0)
        encoder.setVertexBytes(&uniforms, length: MemoryLayout<Uniforms>.stride, index: 1)

        var bandColor = color
        encoder.setFragmentBytes(&bandColor, length: MemoryLayout<SIMD4<Float>>.stride, index: 0)

        encoder.drawPrimitives(type: .triangleStrip, vertexStart: 0, vertexCount: vertices.count)
    }

    // MARK: - Fitted Curve Evaluator & Drawer

    private func drawFittedCurve(
        fit: FitResultEntry,
        xMin: Double,
        xMax: Double,
        samples: Int,
        encoder: MTLRenderCommandEncoder,
        pipeline: MTLRenderPipelineState,
        uniforms: inout Uniforms,
        device: MTLDevice
    ) {
        guard samples >= 2, xMax > xMin else { return }

        var points: [SIMD2<Float>] = []
        points.reserveCapacity(samples)

        let step = (xMax - xMin) / Double(samples - 1)
        for i in 0..<samples {
            let x = xMin + Double(i) * step
            if let y = evaluateModel(type: fit.fitType, params: fit.parameters, x: x) {
                if y.isFinite {
                    points.append(SIMD2<Float>(Float(x), Float(y)))
                }
            }
        }

        guard points.count >= 2 else { return }

        let byteLength = MemoryLayout<SIMD2<Float>>.stride * points.count
        guard let buffer = device.makeBuffer(bytes: points, length: byteLength, options: .storageModeShared) else { return }

        encoder.setRenderPipelineState(pipeline)
        encoder.setVertexBuffer(buffer, offset: 0, index: 0)
        encoder.setVertexBytes(&uniforms, length: MemoryLayout<Uniforms>.stride, index: 1)

        var strokeColor = SIMD4<Float>(0.92, 0.32, 0.25, 1.0) // Vibrant coral for active fit
        encoder.setFragmentBytes(&strokeColor, length: MemoryLayout<SIMD4<Float>>.stride, index: 0)

        encoder.drawPrimitives(type: .lineStrip, vertexStart: 0, vertexCount: points.count)
    }

    private func evaluateModel(type: FitType, params: [Double], x: Double) -> Double? {
        let p = params
        switch type {
        case .linear:
            guard p.count >= 2 else { return nil }
            return p[0] * x + p[1]
        case .quadratic:
            guard p.count >= 3 else { return nil }
            return p[0] * x * x + p[1] * x + p[2]
        case .cubic:
            guard p.count >= 4 else { return nil }
            return p[0] * pow(x, 3) + p[1] * pow(x, 2) + p[2] * x + p[3]
        case .degree4:
            guard p.count >= 5 else { return nil }
            return p[0] * pow(x, 4) + p[1] * pow(x, 3) + p[2] * pow(x, 2) + p[3] * x + p[4]
        case .degree5:
            guard p.count >= 6 else { return nil }
            return p[0] * pow(x, 5) + p[1] * pow(x, 4) + p[2] * pow(x, 3) + p[3] * pow(x, 2) + p[4] * x + p[5]
        case .exponential:
            guard p.count >= 2 else { return nil }
            return p[0] * exp(p[1] * x)
        case .logarithmic:
            guard p.count >= 2, x > 0 else { return nil }
            return p[0] + p[1] * log(x)
        case .power:
            guard p.count >= 2, x > 0 else { return nil }
            return p[0] * pow(x, p[1])
        case .sinusoidal:
            guard p.count >= 4 else { return nil }
            return p[0] * sin(p[1] * x + p[2]) + p[3]
        case .cosinusoidal:
            guard p.count >= 4 else { return nil }
            return p[0] * cos(p[1] * x + p[2]) + p[3]
        case .tangential:
            guard p.count >= 4 else { return nil }
            let ang = p[1] * x + p[2]
            if abs(cos(ang)) < 1e-3 { return nil }
            return p[0] * tan(ang) + p[3]
        case .hyperbolic:
            guard p.count >= 3 else { return nil }
            let denom = x - p[1]
            if abs(denom) < 1e-6 { return nil }
            return p[0] / denom + p[2]
        case .squareRoot:
            guard p.count >= 3 else { return nil }
            let diff = x - p[1]
            guard diff >= 0 else { return nil }
            return p[0] * sqrt(diff) + p[2]
        case .absoluteValue:
            guard p.count >= 3 else { return nil }
            return p[0] * abs(x - p[1]) + p[2]
        case .gaussian:
            guard p.count >= 4 else { return nil }
            let s = max(abs(p[2]), 1e-6)
            return p[0] * exp(-pow(x - p[1], 2) / (2.0 * s * s)) + p[3]
        case .auto:
            return nil
        }
    }
}
