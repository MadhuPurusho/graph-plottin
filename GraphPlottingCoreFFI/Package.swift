// swift-tools-version: 6.2
// GraphPlottingCoreFFI – Local Swift package wrapping the Rust XCFramework.

import PackageDescription

let package = Package(
    name: "GraphPlottingCoreFFI",
    platforms: [
        .macOS(.v26)
    ],
    products: [
        .library(
            name: "GraphPlottingCoreFFI",
            targets: ["GraphPlottingCoreFFI"]
        ),
    ],
    targets: [
        .binaryTarget(
            name: "GraphPlottingCoreFFI",
            path: "GraphPlottingCoreFFI.xcframework"
        ),
    ]
)
