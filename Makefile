BINARY   = md2optex
DESTDIR  ?= /usr/local/bin
EXAMPLE  ?= examples/ukazka.md
BUILDDIR  = build

.PHONY: all build release test check fmt lint install uninstall clean preview pdf tex

all: build

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

check: fmt lint test

fmt:
	cargo fmt --check

lint:
	cargo clippy -- -D warnings

install: release
	install -Dm755 target/release/$(BINARY) $(DESTDIR)/$(BINARY)

uninstall:
	rm -f $(DESTDIR)/$(BINARY)

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

clean:
	cargo clean
	rm -rf $(BUILDDIR)
