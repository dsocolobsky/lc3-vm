.PHONY: run
run:
	cargo run

.PHONY: build
build:
	cargo build

.PHONY: clean
clean:
	cargo clean && rm *.sym

.PHONY: test
test:
	cargo test
