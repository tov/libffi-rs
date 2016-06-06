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

REMOTE_HOST = login.eecs.northwestern.edu
REMOTE_PATH = public_html/code/libffi-rs

upload-doc:
	make doc
	rsync -avz --delete target/doc $(REMOTE_HOST):$(REMOTE_PATH)
	ssh $(REMOTE_HOST) chmod -R a+rX $(REMOTE_PATH)

release:
	make upload-doc
	cargo publish

clean:
	cargo clean
	$(RM) src/raw.rs
