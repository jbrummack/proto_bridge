// swift-tools-version:5.3
import PackageDescription

let package = Package(
    name: "proto_bridge",
    platforms: [
        .macOS(.v10_14), .iOS(.v13), .tvOS(.v13),
    ],
    products: [
        .library(name: "proto_bridge", targets: ["proto_bridge", "proto_bridge_rust"])
    ],
    dependencies: [
        .package(url: "https://github.com/apple/swift-protobuf.git", from: "1.27.0")
    ],
    targets: [
        .target(
            name: "proto_bridge",
            dependencies: [
                .product(name: "SwiftProtobuf", package: "swift-protobuf"),
                .target(name: "proto_bridge_rust"),
            ]
        ),
        .binaryTarget(name: "proto_bridge_rust", path: "./proto_bridge.xcframework"),
    ]
)
