BINARY   = md2opmac
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

# Vygeneruje TeX z ukázkového MD souboru do build/
tex: build
	mkdir -p $(BUILDDIR)
	./target/debug/$(BINARY) $(EXAMPLE) -o $(BUILDDIR)/ukazka.tex
	@echo "Výstup: $(BUILDDIR)/ukazka.tex"

# Vygeneruje TeX a přeloží ho do PDF pomocí OpTeX
pdf: tex
	cd $(BUILDDIR) && luacsplain -interaction=nonstopmode ukazka.tex
	@echo "Výstup: $(BUILDDIR)/ukazka.pdf"

# Otevře PDF v prohlížeči (xdg-open)
preview: pdf
	xdg-open $(BUILDDIR)/ukazka.pdf

clean:
	cargo clean
	rm -rf $(BUILDDIR)
