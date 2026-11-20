// ViewportState.swift

import Foundation

@Observable
final class ViewportState: @unchecked Sendable {
    var xMin: Double
    var xMax: Double
    var yMin: Double
    var yMax: Double

    var xRange: Double { xMax - xMin }
    var yRange: Double { yMax - yMin }
    var centerX: Double { (xMin + xMax) / 2.0 }
    var centerY: Double { (yMin + yMax) / 2.0 }

    init(xMin: Double = -10, xMax: Double = 10, yMin: Double = -10, yMax: Double = 10) {
        self.xMin = xMin
        self.xMax = xMax
        self.yMin = yMin
        self.yMax = yMax
    }

    func reset() {
        xMin = -10
        xMax = 10
        yMin = -10
        yMax = 10
    }

    func zoom(by factor: Double, centerX cx: Double? = nil, centerY cy: Double? = nil) {
        let ax = cx ?? centerX
        let ay = cy ?? centerY
        let newHalfW = xRange * factor / 2.0
        let newHalfH = yRange * factor / 2.0
        xMin = ax - newHalfW
        xMax = ax + newHalfW
        yMin = ay - newHalfH
        yMax = ay + newHalfH
    }

    func pan(dx: Double, dy: Double) {
        xMin += dx
        xMax += dx
        yMin += dy
        yMax += dy
    }

    func fitToData(points: [DataPoint], padding: Double = 0.1) {
        guard !points.isEmpty else { return }
        let xs = points.map(\.x)
        let ys = points.map(\.y)
        let minX = xs.min()!
        let maxX = xs.max()!
        let minY = ys.min()!
        let maxY = ys.max()!
        let padX = max((maxX - minX) * padding, 1.0)
        let padY = max((maxY - minY) * padding, 1.0)
        xMin = minX - padX
        xMax = maxX + padX
        yMin = minY - padY
        yMax = maxY + padY
    }

    var formattedRange: String {
        let fmt = { (v: Double) -> String in
            String(format: "%.2f", v)
        }
        return "x: [\(fmt(xMin)), \(fmt(xMax))]  y: [\(fmt(yMin)), \(fmt(yMax))]"
    }
}
