default: build
hard: test

build:
	clear
	cargo build
#	cargo doc --no-deps

run:
	clear
	cargo run

test:
	clear
	cargo test

clean:
	cargo clean

LIBFFI_CFLAGS = $$(pkg-config --cflags libffi)
BINDGEN_ENV   = DYLD_LIBRARY_PATH=/Library/Developer/CommandLineTools/usr/lib

src/ffi/bindgen.rs: src/c/include_ffi.h
	$(BINDGEN_ENV) bindgen $(LIBFFI_CFLAGS) $< > $@

clean:
	cargo clean
	$(RM) src/ffi/bindgen.rs
