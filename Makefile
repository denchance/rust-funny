
.PHONY: build debug release run run-release test fmt clean

BIN := rust-funny


debug:
	cargo build
	cp target/debug/$(BIN) ./$(BIN)
	cargo clean


release:
	cargo build --release
	cp target/release/$(BIN) ./$(BIN)
	cargo clean

build: debug

run:
	cargo run

run-release:
	cargo run --release

test:
	cargo test

fmt:
	cargo fmt

clean:
	cargo clean
	rm -f ./$(BIN)
