ifeq ($(OS),Windows_NT)
	PLATFORM := windows
else
	uname := $(shell uname)
	ifeq ($(uname),Darwin)
		PLATFORM := macos
	else ifeq ($(uname),Linux)
		PLATFORM := linux
	endif
endif

build:
	cd build_libtika && cargo build
	if [ ! -f gradle-8.10-bin.zip ]; then curl -L -o gradle-8.10-bin.zip https://services.gradle.org/distributions/gradle-8.10-bin.zip; fi
	./build_libtika/target/debug/build_libtika
	cargo build --release
	cp target/release/extract-cli ./
	# Packaging
	mkdir pkg
ifeq ($(PLATFORM),macos)
	mv extract-cli pkg/
	mv *.dylib pkg/
else ifeq ($(PLATFORM),linux)
	mv extract-cli pkg/
	mv *.so pkg/
else ifeq ($(PLATFORM),windows)
	mv extract-cli.exe pkg/
	mv *.dll pkg
endif

clean:
	yes | rm -rf graalvm_jdk
	rm -f gradle-8.10-bin.zip
	cd build_libtika && cargo clean
	cargo clean
	rm -rf pkg
	rm -f libtika_native.lib # On needed on Windows


test:
	./pkg/extract-cli test_dir/hello.pdf test_dir/out
	rm test_dir/out