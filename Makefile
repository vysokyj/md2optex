BINARY   = md2optex
EXAMPLE  ?= examples/ukazka.md
BUILDDIR  = target/examples
STYLES    = minimal book academic manual

.PHONY: all build release install uninstall test check fmt lint clean examples

all: build

build:
	cargo build

release:
	cargo build --release

install:
	cargo install --path .

uninstall:
	cargo uninstall $(BINARY)

test:
	cargo test

check: fmt lint test

fmt:
	cargo fmt --check

lint:
	cargo clippy -- -D warnings

# Build the binary quietly, then generate TeX + PDF for every built-in style.
examples:
	RUSTFLAGS="-A warnings" cargo build --quiet
	mkdir -p $(BUILDDIR)
	@for style in $(STYLES); do \
		echo "--- $$style ---"; \
		./target/debug/$(BINARY) --style $$style $(EXAMPLE) \
			-o $(CURDIR)/$(BUILDDIR)/ukazka-$$style.tex; \
		cd $(CURDIR)/$(BUILDDIR) && \
		optex -interaction=batchmode ukazka-$$style.tex \
			>ukazka-$$style.stdout 2>&1 \
		|| { echo "OpTeX failed for style $$style:"; \
		     grep "^!" ukazka-$$style.log || cat ukazka-$$style.log; \
		     cd $(CURDIR); exit 1; }; \
		cd $(CURDIR); \
		echo "Output: $(BUILDDIR)/ukazka-$$style.pdf"; \
	done
	@echo "Done — PDFs in $(BUILDDIR)/:"
	@ls $(BUILDDIR)/ukazka-*.pdf

clean:
	cargo clean
	rm -rf $(BUILDDIR)
