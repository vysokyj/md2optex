BINARY      = md2optex
EXAMPLE     ?= examples/sample.md
BOOK_SAMPLE  = examples/book-sample
BUILDDIR     = target/examples
STYLES       = minimal book academic manual

.PHONY: all build release install uninstall test check fmt lint clean examples book-sample

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
			-o $(CURDIR)/$(BUILDDIR)/sample-$$style.tex; \
		cd $(CURDIR)/$(BUILDDIR) && \
		optex -interaction=batchmode sample-$$style.tex \
			>sample-$$style.stdout 2>&1 \
		|| { echo "OpTeX failed for style $$style:"; \
		     grep "^!" sample-$$style.log || cat sample-$$style.log; \
		     cd $(CURDIR); exit 1; }; \
		cd $(CURDIR); \
		echo "Output: $(BUILDDIR)/sample-$$style.pdf"; \
	done
	@echo "Done — PDFs in $(BUILDDIR)/:"
	@ls $(BUILDDIR)/sample-*.pdf

book-sample:
	RUSTFLAGS="-A warnings" cargo build --quiet
	mkdir -p $(BUILDDIR)
	./target/debug/$(BINARY) $(BOOK_SAMPLE) -o $(CURDIR)/$(BUILDDIR)/book-sample.tex
	cd $(CURDIR)/$(BUILDDIR) && \
	optex -interaction=batchmode book-sample.tex >book-sample.stdout 2>&1 \
	|| { echo "OpTeX failed:"; grep "^!" book-sample.log || cat book-sample.log; cd $(CURDIR); exit 1; }
	@echo "Output: $(BUILDDIR)/book-sample.pdf"

clean:
	cargo clean
	rm -rf $(BUILDDIR)
