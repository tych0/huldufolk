TEST?=$(patsubst test/%.bats,%,$(wildcard test/*.bats))

CARGO_FLAGS?=
ifndef DEBUG
	# Unfortunately, this seems to be the only way to produce a statically linked rust binary.
	CARGO_FLAGS+="--target x86_64-unknown-linux-musl"
	CARGO_FLAGS+="--release"
endif

.PHONY: all
all:
	rustfmt src/*
	cargo build $(CARGO_FLAGS)
ifndef DEBUG
	strip ./target/release/usermode-helper
endif

.PHONY: check
check:
	# need to force a rebuild for DEFAULT_CONFIG_PATH
	cargo clean -p usermode-helper
	DEFAULT_CONFIG_PATH=./usermode-helper.conf cargo build $(CARGO_FLAGS)
	bats -t $(patsubst %,test/%.bats,$(TEST))

.PHONY: check-dmesg
check-dmesg:
	sudo bats -t test/dmesg.bats

.PHONY: clean
clean:
	cargo clean
