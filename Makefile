BINARY      = md2optex
BOOK_SAMPLE  = examples/book-sample
BUILDDIR     = target/examples
STYLES       = minimal book academic manual
EXAMPLES    = $(wildcard examples/*.md)

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

# Build the binary quietly, then generate TeX + PDF for every example × style combination,
# plus each example with default style.
examples:
	RUSTFLAGS="-A warnings" cargo build --quiet
	mkdir -p $(BUILDDIR)
	@for md in $(EXAMPLES); do \
		name=$$(basename $$md .md); \
		for style in $(STYLES); do \
			echo "--- $$name / $$style ---"; \
			./target/debug/$(BINARY) --style $$style $$md \
				-o $(CURDIR)/$(BUILDDIR)/$$name-$$style.tex; \
			cd $(CURDIR)/$(BUILDDIR) && \
			optex -interaction=batchmode $$name-$$style.tex \
				>$$name-$$style.stdout 2>&1 \
			|| { echo "OpTeX failed for $$name/$$style:"; \
			     grep "^!" $$name-$$style.log || cat $$name-$$style.log; \
			     cd $(CURDIR); exit 1; }; \
			cd $(CURDIR); \
			echo "Output: $(BUILDDIR)/$$name-$$style.pdf"; \
		done; \
	done
	@echo "Done — PDFs in $(BUILDDIR)/:"
	@ls $(BUILDDIR)/*.pdf

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
