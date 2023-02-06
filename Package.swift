// swift-tools-version:5.7
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "TauriWorkspace",
    products: [
        .library(name: "Tauri", targets: ["Tauri"]),
    ],
    dependencies: [
        .package(url: "https://github.com/lucasfernog/swift-rs", branch: "fix/sdk-name"),
    ],
    targets: [
        .target(
            name: "Tauri",
            dependencies: [
                .product(name: "SwiftRs", package: "swift-rs"),
            ],
            path: "core/tauri/ios/Sources/Tauri"
        ),
        .testTarget(
            name: "TauriTests",
            dependencies: ["Tauri"],
            path: "core/tauri/ios/Tests/TauriTests"
        ),
    ]
)
