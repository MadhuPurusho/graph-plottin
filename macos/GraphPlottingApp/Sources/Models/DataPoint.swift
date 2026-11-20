// DataPoint.swift

import Foundation

struct DataPoint: Identifiable, Hashable, Codable, Sendable {
    let id: UUID
    var x: Double
    var y: Double
    var label: String?

    init(id: UUID = UUID(), x: Double, y: Double, label: String? = nil) {
        self.id = id
        self.x = x
        self.y = y
        self.label = label
    }

    init(from core: CoreDataPoint) {
        self.id = UUID()
        self.x = core.x
        self.y = core.y
        self.label = nil
    }

    var corePoint: CoreDataPoint {
        CoreDataPoint(x: x, y: y)
    }
}
