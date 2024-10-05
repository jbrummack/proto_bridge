// swift-tools-version:5.3
import PackageDescription

let package = Package(
    name: "proto_bridge",
    products: [
        .library(name: "proto_bridge", targets: ["proto_bridge"])
    ],
    dependencies: [
        .package(url: "https://github.com/apple/swift-protobuf.git", from: "1.27.0")
    ],
    targets: [
        .target(
            name: "proto_bridge",
            dependencies: [.product(name: "SwiftProtobuf", package: "swift-protobuf")]
        ),
        .binaryTarget(name: "proto_bridge_rust", path: "./proto_bridge.xcframework"),
    ]
)
