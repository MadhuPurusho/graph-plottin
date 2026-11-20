// CurvePalette.swift
// Okabe-Ito colorblind-safe palette for curves and data points.

import SwiftUI

enum CurvePalette {
    // Okabe-Ito palette
    static let colors: [Color] = [
        Color(red: 0.000, green: 0.447, blue: 0.698),  // Blue
        Color(red: 0.902, green: 0.624, blue: 0.000),  // Orange
        Color(red: 0.000, green: 0.620, blue: 0.451),  // Bluish green
        Color(red: 0.800, green: 0.475, blue: 0.655),  // Reddish purple
        Color(red: 0.337, green: 0.706, blue: 0.914),  // Sky blue
        Color(red: 0.835, green: 0.369, blue: 0.000),  // Vermillion
        Color(red: 0.941, green: 0.894, blue: 0.259),  // Yellow
    ]

    static func color(at index: Int) -> Color {
        colors[index % colors.count]
    }

    // SIMD colors for Metal shaders
    static func simdColor(at index: Int) -> SIMD4<Float> {
        // Extract from known values
        let vals: [(Float, Float, Float)] = [
            (0.000, 0.447, 0.698),
            (0.902, 0.624, 0.000),
            (0.000, 0.620, 0.451),
            (0.800, 0.475, 0.655),
            (0.337, 0.706, 0.914),
            (0.835, 0.369, 0.000),
            (0.941, 0.894, 0.259),
        ]
        let v = vals[index % vals.count]
        return SIMD4<Float>(v.0, v.1, v.2, 1.0)
    }
}
