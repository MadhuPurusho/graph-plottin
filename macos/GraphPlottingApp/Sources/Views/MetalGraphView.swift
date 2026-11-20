// MetalGraphView.swift
// Native AppKit MTKView host with 120 FPS ProMotion, inertial trackpad pan/zoom, and magnetic probe snapping.

import SwiftUI
import MetalKit

final class InteractiveMTKView: MTKView {
    weak var appState: AppState?
    private var trackingAreaRef: NSTrackingArea?
    private var lastMousePoint: CGPoint?

    override var intrinsicContentSize: NSSize {
        NSSize(width: 700, height: 500)
    }

    override func updateTrackingAreas() {
        super.updateTrackingAreas()
        if let existing = trackingAreaRef {
            removeTrackingArea(existing)
        }
        let area = NSTrackingArea(
            rect: bounds,
            options: [.mouseMoved, .mouseEnteredAndExited, .activeInKeyWindow, .inVisibleRect],
            owner: self,
            userInfo: nil
        )
        addTrackingArea(area)
        self.trackingAreaRef = area
    }

    // MARK: - Pan via Dragging

    override func mouseDown(with event: NSEvent) {
        lastMousePoint = convert(event.locationInWindow, from: nil)
    }

    override func mouseDragged(with event: NSEvent) {
        guard let appState = appState, let last = lastMousePoint else { return }
        let current = convert(event.locationInWindow, from: nil)
        let dxScreen = Double(current.x - last.x)
        let dyScreen = Double(current.y - last.y)

        let w = Double(bounds.width)
        let h = Double(bounds.height)
        guard w > 0 && h > 0 else { return }

        let dxWorld = -dxScreen / w * appState.viewport.xRange
        let dyWorld = -dyScreen / h * appState.viewport.yRange

        appState.viewport.pan(dx: dxWorld, dy: dyWorld)
        appState.needsRedraw = true
        self.lastMousePoint = current
        setNeedsDisplay(bounds)
    }

    override func mouseUp(with event: NSEvent) {
        lastMousePoint = nil
    }

    // MARK: - Pan & Zoom via Scroll Wheel / Trackpad

    override func scrollWheel(with event: NSEvent) {
        guard let appState = appState else { return }
        let w = Double(bounds.width)
        let h = Double(bounds.height)
        guard w > 0 && h > 0 else { return }

        if event.modifierFlags.contains(.command) || event.modifierFlags.contains(.control) {
            // Zoom centered on cursor
            let loc = convert(event.locationInWindow, from: nil)
            let normX = Double(loc.x / bounds.width)
            let normY = Double(loc.y / bounds.height)
            let cursorWorldX = appState.viewport.xMin + normX * appState.viewport.xRange
            let cursorWorldY = appState.viewport.yMin + normY * appState.viewport.yRange

            let zoomFactor = event.deltaY > 0 ? 0.92 : 1.08
            appState.viewport.zoom(by: zoomFactor, centerX: cursorWorldX, centerY: cursorWorldY)
        } else {
            // Pan
            let dxWorld = -Double(event.scrollingDeltaX) / w * appState.viewport.xRange * 0.4
            let dyWorld = -Double(event.scrollingDeltaY) / h * appState.viewport.yRange * 0.4
            appState.viewport.pan(dx: dxWorld, dy: dyWorld)
        }

        appState.needsRedraw = true
        setNeedsDisplay(bounds)
    }

    // MARK: - Pinch-to-Zoom Gesture

    override func magnify(with event: NSEvent) {
        guard let appState = appState else { return }
        let loc = convert(event.locationInWindow, from: nil)
        let normX = Double(loc.x / bounds.width)
        let normY = Double(loc.y / bounds.height)
        let cursorWorldX = appState.viewport.xMin + normX * appState.viewport.xRange
        let cursorWorldY = appState.viewport.yMin + normY * appState.viewport.yRange

        let factor = 1.0 - Double(event.magnification)
        if factor > 0.1 && factor < 10.0 {
            appState.viewport.zoom(by: factor, centerX: cursorWorldX, centerY: cursorWorldY)
            appState.needsRedraw = true
            setNeedsDisplay(bounds)
        }
    }

    // MARK: - Mouse Hover & Magnetic Snapping

    override func mouseMoved(with event: NSEvent) {
        guard let appState = appState else { return }
        let loc = convert(event.locationInWindow, from: nil)
        let w = Double(bounds.width)
        let h = Double(bounds.height)
        guard w > 0 && h > 0 else { return }

        let normX = Double(loc.x / bounds.width)
        let normY = Double(loc.y / bounds.height)
        let rawWorldX = appState.viewport.xMin + normX * appState.viewport.xRange
        let rawWorldY = appState.viewport.yMin + normY * appState.viewport.yRange

        var snappedX = rawWorldX
        var snappedY = rawWorldY

        // Magnetic snap to nearest data point within 20 screen pixels
        let magneticRadiusScreen = 20.0
        var minDist = magneticRadiusScreen

        for pt in appState.points {
            let ptScreenX = (pt.x - appState.viewport.xMin) / appState.viewport.xRange * w
            let ptScreenY = (pt.y - appState.viewport.yMin) / appState.viewport.yRange * h
            let dist = hypot(Double(loc.x) - ptScreenX, Double(loc.y) - ptScreenY)
            if dist < minDist {
                minDist = dist
                snappedX = pt.x
                snappedY = pt.y
            }
        }

        appState.probeX = snappedX
        appState.probeY = snappedY
        appState.showProbe = true
    }

    override func mouseExited(with event: NSEvent) {
        appState?.showProbe = false
    }
}

struct MetalGraphView: NSViewRepresentable {
    @Environment(AppState.self) private var appState

    func makeNSView(context: Context) -> InteractiveMTKView {
        let mtkView = InteractiveMTKView()
        mtkView.device = MTLCreateSystemDefaultDevice()
        mtkView.delegate = context.coordinator
        mtkView.isPaused = true
        mtkView.enableSetNeedsDisplay = true
        mtkView.preferredFramesPerSecond = 120
        mtkView.colorPixelFormat = .bgra8Unorm_srgb
        mtkView.depthStencilPixelFormat = .invalid
        mtkView.clearColor = MTLClearColor(red: 0.98, green: 0.98, blue: 0.99, alpha: 1.0)
        mtkView.layer?.isOpaque = true
        mtkView.appState = appState

        if let device = mtkView.device {
            context.coordinator.setup(device: device, view: mtkView)
        }

        return mtkView
    }

    func updateNSView(_ mtkView: InteractiveMTKView, context: Context) {
        mtkView.appState = appState
        context.coordinator.appState = appState
        if appState.needsRedraw {
            mtkView.setNeedsDisplay(mtkView.bounds)
            Task { @MainActor in
                appState.needsRedraw = false
            }
        }
    }

    func makeCoordinator() -> GraphRenderer {
        GraphRenderer()
    }
}
