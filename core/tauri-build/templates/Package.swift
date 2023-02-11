// swift-tools-version:5.7
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "PluginWorkspace",
    products: [
        .library(name: "$PLUGIN_PACKAGE_NAME", type: .static, targets: ["$PLUGIN_PACKAGE_NAME"]),
    ],
    dependencies: [
        .package(name: "Tauri", path: "$TAURI_PATH"),
    ],
    targets: [
        .target(
            name: "$PLUGIN_PACKAGE_NAME",
            dependencies: [
                .byName(name: "Tauri")
            ],
            path: "$PLUGIN_PACKAGE_SRC_PATH"
        ),
    ]
)
