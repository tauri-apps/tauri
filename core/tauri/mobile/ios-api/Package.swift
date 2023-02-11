// swift-tools-version: 5.7
// The swift-tools-version declares the minimum version of Swift required to build this package.

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
		.package(url: "https://github.com/Brendonovich/swift-rs", revision: "eb6de914ad57501da5019154d476d45660559999"),
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
