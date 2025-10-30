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
	./build_libtika/target/debug/build_libtika
	cargo build --release
	cp target/release/extract-cli ./

clean:
	yes | rm -r graalvm_jdk
	cd build_libtika && cargo clean
	cargo clean
	$(RM_CMD) $(RM_LIBS)
test: build
	./extract-cli test/hello.pdf test/out
	rm test/out

uselib:
	cd use_lib && cargo b
	find ./use_lib -name 'libtika_native.so' -exec sudo mv {} /usr/lib64 \;
	./use_lib/target/debug/use_lib ./test/hello.pdf