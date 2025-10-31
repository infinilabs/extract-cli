RM_CMD = rm -f
RM_LIBS =

ifeq ($(OS),Windows_NT)
  RM_CMD = del
  RM_LIBS = libtika_native.dll
else
  UNAME_S := $(shell uname -s)
  ifeq ($(UNAME_S),Darwin)
    RM_LIBS = libtika_native.dylib libtika_native.so
  else ifeq ($(UNAME_S),Linux)
    RM_LIBS = libtika_native.so
  else
    RM_LIBS = libtika_native.dylib libtika_native.so libtika_native.dll
  endif
endif


build:
	cd build_libtika && cargo build
	wget -nc https://services.gradle.org/distributions/gradle-8.10-bin.zip
	./build_libtika/target/debug/build_libtika
	cargo build --release
	cp target/release/extract-cli ./
	# Packaging
	mkdir pkg
	mv extract-cli pkg/
ifeq ($(UNAME_S),Darwin)
	mv *.dylib pkg/
else ifeq ($(UNAME_S),Linux)
	mv *.so pkg/
endif

clean:
	yes | rm -rf graalvm_jdk
	rm -f gradle-8.10-bin.zip
	cd build_libtika && cargo clean
	cargo clean
	$(RM_CMD) -f $(RM_LIBS)
	rm -rf pkg

test:
	./pkg/extract-cli test_dir/hello.pdf test_dir/out
	rm test_dir/out