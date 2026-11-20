// swift-tools-version: 6.2
import PackageDescription

let package = Package(
    name: "GraphPlottingApp",
    platforms: [
        .macOS(.v26)
    ],
    dependencies: [
        .package(path: "../../GraphPlottingCoreFFI")
    ],
    targets: [
        .executableTarget(
            name: "GraphPlottingApp",
            dependencies: [
                .product(name: "GraphPlottingCoreFFI", package: "GraphPlottingCoreFFI")
            ],
            path: "Sources"
        ),
        .testTarget(
            name: "GraphPlottingAppTests",
            dependencies: [
                "GraphPlottingApp",
                .product(name: "GraphPlottingCoreFFI", package: "GraphPlottingCoreFFI")
            ],
            path: "Tests"
        )
    ]
)
