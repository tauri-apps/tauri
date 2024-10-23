// swift-tools-version:5.3
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import PackageDescription

let package = Package(
    name: "tauri-plugin-sample",
    platforms: [
        .macOS(.v10_13),
        .iOS(.v13),
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
        .package(name: "Tauri", path: "../../../../../crates/tauri/mobile/ios-api")
    ],
    targets: [
        // Targets are the basic building blocks of a package. A target can define a module or a test suite.
        // Targets can depend on other targets in this package, and on products in packages this package depends on.
        .target(
            name: "tauri-plugin-sample",
            dependencies: [
                .byName(name: "Tauri")
            ],
            path: "Sources")
    ]
)
