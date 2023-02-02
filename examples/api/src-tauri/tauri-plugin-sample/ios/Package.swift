// swift-tools-version:5.7
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "tauri-plugin-sample",
    platforms: [
        .iOS(.v11),
    ],
    products: [
        // Products define the executables and libraries a package produces, and make them visible to other packages.
        .library(
            name: "tauri-plugin-sample",
            type: .static,
            targets: ["tauri-plugin-sample"]),
    ],
    dependencies: [
        // Dependencies declare other packages that this package depends on.
        .package(url: "https://github.com/lucasfernog/swift-rs", branch: "fix/sdk-name"),
        .package(name: "Tauri", path: "../../../../../tooling/cli/mobile/tauri-ios/"),
    ],
    targets: [
        // Targets are the basic building blocks of a package. A target can define a module or a test suite.
        // Targets can depend on other targets in this package, and on products in packages this package depends on.
        .target(
            name: "tauri-plugin-sample",
            dependencies: [
                .product(name: "SwiftRs", package: "swift-rs"),
                "Tauri"
            ],
            path: "Sources")
    ]
)
