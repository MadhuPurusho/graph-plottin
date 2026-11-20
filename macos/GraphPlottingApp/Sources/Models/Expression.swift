// Expression.swift

import Foundation
import SwiftUI

struct ExpressionEntry: Identifiable, Hashable, Sendable {
    let id: UUID
    var source: String
    var isVisible: Bool
    var color: Color
    var lineWidth: Double
    var parameters: [ParameterBinding]

    init(
        id: UUID = UUID(),
        source: String = "",
        isVisible: Bool = true,
        color: Color = .blue,
        lineWidth: Double = 2.0,
        parameters: [ParameterBinding] = []
    ) {
        self.id = id
        self.source = source
        self.isVisible = isVisible
        self.color = color
        self.lineWidth = lineWidth
        self.parameters = parameters
    }

    static func == (lhs: ExpressionEntry, rhs: ExpressionEntry) -> Bool {
        lhs.id == rhs.id
    }

    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }
}

struct ParameterBinding: Identifiable, Hashable, Sendable {
    let id: UUID
    var name: String
    var value: Double
    var min: Double
    var max: Double
    var step: Double

    init(
        id: UUID = UUID(),
        name: String,
        value: Double = 1.0,
        min: Double = -10.0,
        max: Double = 10.0,
        step: Double = 0.1
    ) {
        self.id = id
        self.name = name
        self.value = value
        self.min = min
        self.max = max
        self.step = step
    }
}
