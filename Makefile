default: build
hard: test

CRATE = libffi
REPO  = libffi-rs

# My system seems to want this. How can we make it portable?
export DYLD_LIBRARY_PATH=/Library/Developer/CommandLineTools/usr/lib

build:
	clear
	cargo build
	make doc

clippy:
	rustup run nightly cargo build --features=clippy

doc:
	cargo doc --no-deps
	echo "<meta http-equiv='refresh' content='0;url=$(CRATE)/'>" > target/doc/index.html

test:
	clear
	cargo test

upload-doc:
	make doc
	ghp-import -n target/doc
	git push -f https://github.com/tov/$(REPO).git gh-pages

release:
	make upload-doc
	cargo publish

clean:
	cargo clean
	$(RM) src/raw.rs
