// swift-tools-version: 5.7
// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import PackageDescription

let package = Package(
	name: "Tauri",
	products: [
		// Products define the executables and libraries a package produces, and make them visible to other packages.
		.library(
			name: "Tauri",
			type: .static,
			targets: ["Tauri"]),
	],
	dependencies: [
		// Dependencies declare other packages that this package depends on.
		.package(url: "https://github.com/Brendonovich/swift-rs", revision: "56b14aa4aa61e93d0fddf695d0cad78b6dd392b4"),
	],
	targets: [
		// Targets are the basic building blocks of a package. A target can define a module or a test suite.
		// Targets can depend on other targets in this package, and on products in packages this package depends on.
		.target(
			name: "Tauri",
			dependencies: [
					.product(name: "SwiftRs", package: "swift-rs"),
			],
			path: "Sources"
		),
		.testTarget(
			name: "TauriTests",
			dependencies: ["Tauri"]
		),
	]
)
