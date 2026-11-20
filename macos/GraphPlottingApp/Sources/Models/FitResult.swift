// FitResult.swift

import Foundation

struct FitResultEntry: Identifiable, Sendable {
    let id: UUID
    let fitType: FitType
    let parameters: [Double]
    let parameterNames: [String]
    let standardErrors: [Double]
    let equation: String
    let latexEquation: String
    let rSquared: Double
    let adjustedRSquared: Double
    let rmse: Double
    let mae: Double
    let aicc: Double
    let bic: Double
    let loocvRmse: Double
    let residuals: [Double]
    let cooksDistances: [Double]
    let confidenceBandUpper: [DataPoint]
    let confidenceBandLower: [DataPoint]
    let predictionBandUpper: [DataPoint]
    let predictionBandLower: [DataPoint]

    init(id: UUID = UUID(), from core: CoreFitResult) {
        self.id = id
        self.fitType = core.fitType
        self.parameters = core.parameters
        self.parameterNames = core.parameterNames
        self.standardErrors = core.standardErrors
        self.equation = core.equation
        self.latexEquation = core.latexEquation
        self.rSquared = core.rSquared
        self.adjustedRSquared = core.adjustedRSquared
        self.rmse = core.rmse
        self.mae = core.mae
        self.aicc = core.aicc
        self.bic = core.bic
        self.loocvRmse = core.loocvRmse
        self.residuals = core.residuals
        self.cooksDistances = core.cooksDistances
        self.confidenceBandUpper = core.confidenceBandUpper.map { DataPoint(from: $0) }
        self.confidenceBandLower = core.confidenceBandLower.map { DataPoint(from: $0) }
        self.predictionBandUpper = core.predictionBandUpper.map { DataPoint(from: $0) }
        self.predictionBandLower = core.predictionBandLower.map { DataPoint(from: $0) }
    }

    var formattedR2: String { String(format: "%.6f", rSquared) }
    var formattedRMSE: String { String(format: "%.4f", rmse) }
    var formattedAICc: String { String(format: "%.2f", aicc) }
}
