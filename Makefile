CARGO=cargo
CARGO_FLAGS=
PREFIX?=/usr

ifneq ($(MODE),debug)
	TARGET=target/release/effitask
	CARGO_FLAGS+=--release
else
	TARGET=target/debug/effitask
endif

all: build

build: gtk4
	$(CARGO) build $(CARGO_FLAGS)

gtk4:
	@if ! pkg-config $@; then \
		printf '%s not installed\n' "$@" >&2; \
		exit 1; \
	fi

install:
	install --directory $(PREFIX)/bin
	install $(TARGET) $(PREFIX)/bin/

test:
	$(CARGO) test $(CARGO_FLAGS)

.PHONY: all build install test
