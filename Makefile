default: build
hard: test

build:
	clear
	cargo build
	make doc

doc:
	cargo doc --no-deps

test:
	clear
	cargo test

HOST = login.eecs.northwestern.edu
PATH = public_html/code/libffi-rs

upload-doc:
	make doc
	rsync -avz --delete target/doc $(HOST):$(PATH)
	ssh $(HOST) chmod -R a+rX $(PATH)

release:
	make upload-doc
	cargo publish

clean:
	cargo clean
	$(RM) src/raw.rs
