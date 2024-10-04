# protoc --swift_out=./proto_bridge messaging.proto
libname = librustcore
lib = $(libname).a
crate = mobile_integration_test
framework_name = RustCore

ANDROID_HOME = /Users/$$USER/Library/Android/sdk
NDK_HOME=$(ANDROID_HOME)/ndk/

#modulemap:



show_ndk:
	@cd $(NDK_HOME) && ls

setup_toolchain_android:
	@rustup target add aarch64-linux-android
	@rustup target add armv7-linux-androideabi
	@rustup target add i686-linux-android
	@rustup target add x86_64-linux-android
	@cargo install cargo-ndk

setup_toolchain_apple:
	@brew install swift-protobuf
	@rustup target add aarch64-apple-darwin
	@rustup target add aarch64-apple-ios
	@rustup target add aarch64-apple-ios-sim
	@rustup target add x86_64-apple-ios
	@rustup component add rust-src --toolchain nightly-aarch64-apple-darwin

setup_project:
	@mkdir include
	@mkdir libs
	@mkdir .cargo
	@cd include
	@echo "module $(framework_name) {\n header \"$(libname).h\"\n export *\n}" > module.modulemap

macos:
	@cbindgen --config cbindgen.toml --crate $(crate) --output include/$(libname).h
	@cargo build --release --lib --target aarch64-apple-darwin
	@cargo build --release --lib --target x86_64-apple-darwin
	@cargo +nightly build -Z build-std --release --lib --target aarch64-apple-ios-macabi
	@cargo +nightly build -Z build-std --release --lib --target x86_64-apple-ios-macabi
	@$(RM) -rf libs/$(libname)-macos.a
	@$(RM) -rf libs/$(libname)-maccatalyst.a
	@lipo -create -output libs/$(libname)-macos.a \
            target/aarch64-apple-darwin/release/$(lib) \
            target/x86_64-apple-darwin/release/$(lib)
	@lipo -create -output libs/$(libname)-maccatalyst.a \
            target/aarch64-apple-ios-macabi/release/$(lib) \
            target/x86_64-apple-ios-macabi/release/$(lib)
ios:
	@cbindgen --config cbindgen.toml --crate $(crate) --output include/$(libname).h
	@cargo build --release --lib --target aarch64-apple-ios
	@cargo build --release --lib --target aarch64-apple-ios-sim
	@cargo build --release --lib --target x86_64-apple-ios
	@$(RM) -rf libs/$(libname)-ios.a
	@$(RM) -rf libs/$(libname)-ios-sim.a
	@cp target/aarch64-apple-ios/release/$(lib) libs/$(libname)-ios.a
	@lipo -create -output libs/$(libname)-ios-sim.a target/aarch64-apple-ios-sim/release/$(lib) target/x86_64-apple-ios/release/$(lib)


create_framework:
	@$(RM) -rf $(framework_name).xcframework
	@xcodebuild -create-xcframework \
	   -library libs/$(libname)-macos.a \
		-headers ./include/ \
		-library libs/$(libname)-ios-sim.a \
		-headers ./include/ \
		-library libs/$(libname)-maccatalyst.a \
		-headers ./include/ \
		-library libs/$(libname)-ios.a \
		-headers ./include/ \
		-output $(framework_name).xcframework


android:
	@cargo ndk -t armeabi-v7a -t arm64-v8a -o ./jniLibs build --release
