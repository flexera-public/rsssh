prefix = /usr/local/bin

build: target/release/rsssh

run:
	@cargo run

test:
	@cargo test

install: build
	@cp target/release/rsssh $(prefix)/

uninstall:
	@rm $(prefix)/rsssh

target/release/rsssh: src
	@cargo build --release

.PHONY: build run test clean install uninstall
