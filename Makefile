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
	wget https://services.gradle.org/distributions/gradle-8.10-bin.zip
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
	yes | rm -r graalvm_jdk
	rm gradle-8.10-bin.zip
	cd build_libtika && cargo clean
	cargo clean
	$(RM_CMD) $(RM_LIBS)
	rm pkg

test: build
	./pkg/extract-cli test/hello.pdf test/out
	rm test/out