# proto_bridge
This project is a template for interop between mobile platforms and Rust.
This fills the same niche as Kotlin MP, you can share business logic between Android and iOS.

## dependencies
- ```cargo```
- ```protoc```
- ```make```


## iOS
### Setup toolchain:
- ```make setup_toolchain_apple```

### Build template:
- Add the proto_bridge directory as SPM Package to your XCode Project
- ```make apple```
- import proto_bridge


## Planned features
- Android
- macros/codegen for automatic bridging and better ergonomics
