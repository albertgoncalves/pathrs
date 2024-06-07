MAKEFLAGS += --silent
SHARED = \
	--color always \
	--edition 2021
RUSTC = \
	-C debuginfo=2 \
	-C llvm-args="-ffast-math" \
	-C opt-level=3 \
	-C overflow-checks=yes \
	-C panic=unwind \
	-C target-cpu=native \
	-W absolute-paths-not-starting-with-crate \
	-W anonymous-parameters \
	-W deprecated-in-future \
	-W elided-lifetimes-in-paths \
	-W explicit-outlives-requirements \
	-W future-incompatible \
	-W indirect-structural-match \
	-W keyword-idents \
	-W let-underscore \
	-W macro-use-extern-crate \
	-W meta-variable-misuse \
	-W non-ascii-idents \
	-W nonstandard-style \
	-W rust-2018-compatibility \
	-W rust-2018-idioms \
	-W rust-2021-compatibility \
	-W rust-2024-compatibility \
	-W trivial-casts \
	-W trivial-numeric-casts \
	-W unused
CLIPPY = \
	-D warnings \
	-W clippy::all \
	-W clippy::complexity \
	-W clippy::correctness \
	-W clippy::nursery \
	-W clippy::pedantic \
	-W clippy::perf \
	-W clippy::suspicious \
	-A clippy::similar-names \
	-A clippy::too_many_lines
LIBS = \
	-lGL \
	-lglfw

.PHONY: all
all: bin/main bin/test

.PHONY: clean
clean:
	rm -rf bin/

bin/main: src/*
	mkdir -p bin/
	clang-format -i src/*.glsl
	rustfmt $(SHARED) src/*.rs
	mold -run clippy-driver $(SHARED) $(RUSTC) $(CLIPPY) $(LIBS) -o ./bin/main src/main.rs

.PHONY: run
run: bin/main
	RUST_BACKTRACE=1 ./bin/main

bin/test: src/*
	mkdir -p bin/
	clang-format -i src/*.glsl
	rustfmt $(SHARED) src/*.rs
	mold -run clippy-driver $(SHARED) $(RUSTC) $(CLIPPY) $(LIBS) --test -o ./bin/test src/main.rs

.PHONY: test
test: bin/test
	RUST_BACKTRACE=1 ./bin/test
