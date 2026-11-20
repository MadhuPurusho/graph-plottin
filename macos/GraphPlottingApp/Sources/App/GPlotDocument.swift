// GPlotDocument.swift

import SwiftUI
import UniformTypeIdentifiers

extension UTType {
    static let gplot = UTType(exportedAs: "com.graphplotting.gplot", conformingTo: .json)
}

struct GPlotDocumentData: Codable {
    var points: [DataPoint]
    var expressions: [String]
    var viewport: ViewportData
    var mode: String

    struct ViewportData: Codable {
        var xMin: Double
        var xMax: Double
        var yMin: Double
        var yMax: Double
    }

    static var empty: GPlotDocumentData {
        GPlotDocumentData(
            points: [],
            expressions: [],
            viewport: ViewportData(xMin: -10, xMax: 10, yMin: -10, yMax: 10),
            mode: "plotPoints"
        )
    }
}

struct GPlotDocument: FileDocument {
    static var readableContentTypes: [UTType] { [.gplot, .json] }
    static var writableContentTypes: [UTType] { [.gplot] }

    var data: GPlotDocumentData

    init(data: GPlotDocumentData = .empty) {
        self.data = data
    }

    init(configuration: ReadConfiguration) throws {
        guard let fileData = configuration.file.regularFileContents else {
            throw CocoaError(.fileReadCorruptFile)
        }
        let decoder = JSONDecoder()
        self.data = try decoder.decode(GPlotDocumentData.self, from: fileData)
    }

    func fileWrapper(configuration: WriteConfiguration) throws -> FileWrapper {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        let jsonData = try encoder.encode(data)
        return FileWrapper(regularFileWithContents: jsonData)
    }
}
