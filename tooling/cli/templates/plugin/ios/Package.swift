// swift-tools-version:5.7
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "tauri-plugin-{{ plugin_name }}",
    platforms: [
        .iOS(.v11),
    ],
    products: [
        // Products define the executables and libraries a package produces, and make them visible to other packages.
        .library(
            name: "tauri-plugin-{{ plugin_name }}",
            type: .static,
            targets: ["tauri-plugin-{{ plugin_name }}"]),
    ],
    dependencies: [
        // Dependencies declare other packages that this package depends on.
        .package(url: "https://github.com/lucasfernog/tauri", branch: "feat/ios-plugins"),
    ],
    targets: [
        // Targets are the basic building blocks of a package. A target can define a module or a test suite.
        // Targets can depend on other targets in this package, and on products in packages this package depends on.
        .target(
            name: "tauri-plugin-{{ plugin_name }}",
            dependencies: [
                .product(name: "Tauri", package: "tauri")
            ],
            path: "Sources")
    ]
)
