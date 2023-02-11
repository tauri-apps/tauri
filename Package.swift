// swift-tools-version:5.7
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "TauriWorkspace",
    products: [
        .library(name: "Tauri", targets: ["Tauri"]),
    ],
    dependencies: [
        .package(url: "https://github.com/Brendonovich/swift-rs", revision: "eb6de914ad57501da5019154d476d45660559999"),
    ],
    targets: [
        .target(
            name: "Tauri",
            dependencies: [
                .product(name: "SwiftRs", package: "swift-rs"),
            ],
            path: "core/tauri/mobile/ios-api/Sources/Tauri"
        ),
        .testTarget(
            name: "TauriTests",
            dependencies: ["Tauri"],
            path: "core/tauri/mobile/ios-api/Tests/TauriTests"
        ),
    ]
)
