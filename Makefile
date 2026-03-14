BINARY   = md2optex
EXAMPLE  ?= examples/ukazka.md
BUILDDIR  = build
STYLES    = minimal book academic manual

.PHONY: all build release install uninstall test check fmt lint clean tex pdf preview \
        pdf-styles tex-minimal tex-book tex-academic tex-manual \
        pdf-minimal pdf-book pdf-academic pdf-manual

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

# Generate TeX from the example Markdown file into build/
tex:
	RUSTFLAGS="-A warnings" cargo build --quiet
	mkdir -p $(BUILDDIR)
	./target/debug/$(BINARY) $(EXAMPLE) -o $(BUILDDIR)/ukazka.tex
	@echo "Output: $(BUILDDIR)/ukazka.tex"

# Generate TeX and compile to PDF with OpTeX (silent; log in build/ukazka.log)
pdf: tex
	cd $(BUILDDIR) && optex -interaction=batchmode ukazka.tex >ukazka.stdout 2>&1 \
		|| { echo "OpTeX failed — see $(BUILDDIR)/ukazka.log:"; \
		     grep "^!" $(BUILDDIR)/ukazka.log || cat $(BUILDDIR)/ukazka.log; \
		     exit 1; }
	@echo "Output: $(BUILDDIR)/ukazka.pdf"

# Open PDF in the default viewer
preview: pdf
	xdg-open $(BUILDDIR)/ukazka.pdf

# Per-style TeX generation: make tex-book
tex-minimal tex-book tex-academic tex-manual:
	RUSTFLAGS="-A warnings" cargo build --quiet
	mkdir -p $(BUILDDIR)
	./target/debug/$(BINARY) --style $(subst tex-,,$@) $(EXAMPLE) \
		-o $(BUILDDIR)/ukazka-$(subst tex-,,$@).tex
	@echo "Output: $(BUILDDIR)/ukazka-$(subst tex-,,$@).tex"

# Per-style PDF compilation: make pdf-book
pdf-minimal: tex-minimal
	cd $(BUILDDIR) && optex -interaction=batchmode ukazka-minimal.tex >ukazka-minimal.stdout 2>&1 \
		|| { echo "OpTeX failed:"; grep "^!" ukazka-minimal.log || cat ukazka-minimal.log; exit 1; }
	@echo "Output: $(BUILDDIR)/ukazka-minimal.pdf"

pdf-book: tex-book
	cd $(BUILDDIR) && optex -interaction=batchmode ukazka-book.tex >ukazka-book.stdout 2>&1 \
		|| { echo "OpTeX failed:"; grep "^!" ukazka-book.log || cat ukazka-book.log; exit 1; }
	@echo "Output: $(BUILDDIR)/ukazka-book.pdf"

pdf-academic: tex-academic
	cd $(BUILDDIR) && optex -interaction=batchmode ukazka-academic.tex >ukazka-academic.stdout 2>&1 \
		|| { echo "OpTeX failed:"; grep "^!" ukazka-academic.log || cat ukazka-academic.log; exit 1; }
	@echo "Output: $(BUILDDIR)/ukazka-academic.pdf"

pdf-manual: tex-manual
	cd $(BUILDDIR) && optex -interaction=batchmode ukazka-manual.tex >ukazka-manual.stdout 2>&1 \
		|| { echo "OpTeX failed:"; grep "^!" ukazka-manual.log || cat ukazka-manual.log; exit 1; }
	@echo "Output: $(BUILDDIR)/ukazka-manual.pdf"

# Generate PDFs for all built-in styles
pdf-styles: pdf-minimal pdf-book pdf-academic pdf-manual
	@echo "All style PDFs generated in $(BUILDDIR)/"

clean:
	cargo clean
	rm -rf $(BUILDDIR)
