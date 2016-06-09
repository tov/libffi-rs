default: build
hard: test

# My system seems to want this. How can we make it portable?
export DYLD_LIBRARY_PATH=/Library/Developer/CommandLineTools/usr/lib

build:
	clear
	cargo build
	make doc

doc:
	cargo doc --no-deps
	echo '<meta http-equiv="refresh" content="0;url=libffi/">' > target/doc/index.html
	tr -d '\37' < /usr/local/share/info/libffi.info > target/doc/libffi/raw/libffi.txt

test:
	clear
	cargo test

upload-doc:
	make doc
	ghp-import -n target/doc
	git push -f https://github.com/tov/libffi-rs.git gh-pages

release:
	make upload-doc
	cargo publish

clean:
	cargo clean
	$(RM) src/raw.rs
