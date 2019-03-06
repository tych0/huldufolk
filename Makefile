TEST?=$(patsubst test/%.bats,%,$(wildcard test/*.bats))

.PHONY: all
all:
	cargo build

.PHONY: check
check:
	# need to force a rebuild for DEFAULT_CONFIG_PATH
	cargo clean -p usermode-helper
	DEFAULT_CONFIG_PATH=./usermode-helper.conf cargo build
	bats -t $(patsubst %,test/%.bats,$(TEST))

.PHONY: clean
clean:
	cargo clean
