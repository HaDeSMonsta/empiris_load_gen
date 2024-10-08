target := "x86_64-unknown-linux-musl"

default:
	@just --list
run:
	cargo run
build:
	cargo build
clean:
	cargo clean
release:
	cargo build --release --target {{target}}

