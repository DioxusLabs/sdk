// swift-tools-version:5.7

import PackageDescription

let package = Package(
  name: "haptics-plugin",
  platforms: [
    .iOS(.v13)
  ],
  products: [
    .library(
      name: "HapticsPlugin",
      targets: ["HapticsPlugin"]
    )
  ],
  targets: [
    .target(
      name: "HapticsPlugin",
      path: "Sources/HapticsPlugin"
    )
  ]
)
