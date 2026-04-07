# md2optex

CLI nástroj v Rustu, který převádí Markdown na TeX s makry OpTeX (OpTeX).

## Cíl projektu

Čte Markdown soubor (nebo stdin) a na stdout (nebo do souboru) vypisuje validní TeX zdrojový kód využívající makra OpTeX.

## Konvence

- Jazyk kódu: Rust (edition 2021)
- **Zdrojový kód kompletně anglicky** — komentáře, doc-stringy, názvy proměnných, chybové zprávy, vše
- **Konfigurace (`metadata.toml`) kompletně anglicky** — klíče, sekce i názvy šablon jsou anglicky
- **Názvy vestavěných šablon anglicky**: `minimal`, `book`, `academic`, `manual`
- Knihovny: preferuj standardní crates (clap pro CLI, pulldown-cmark pro parsování MD)
- Testy: unit testy v příslušném modulu (`#[cfg(test)]`), integrační testy v `tests/`
- Formátování: `rustfmt` (výchozí konfigurace)
- Linting: `clippy` bez varování

## Architektura

```
src/
  main.rs       – vstupní bod, zpracování argumentů (clap)
  parser.rs     – parsování Markdown pomocí pulldown-cmark
  renderer.rs   – převod událostí parseru na OpTeX TeX výstup
  error.rs      – vlastní chybové typy
tests/
  *.rs          – integrační testy (vstup MD → očekávaný TeX výstup)
```

## Mapování Markdown → OpTeX

| Markdown              | OpTeX TeX                        |
|-----------------------|----------------------------------|
| `# Nadpis`            | `\chap Nadpis`                   |
| `## Podnadpis`        | `\sec Podnadpis`                 |
| `### Podpodnadpis`    | `\secc Podpodnadpis`             |
| `**tučně**`           | `{\bf tučně}`                    |
| `*kurzíva*`           | `{\it kurzíva}`                  |
| `` `kód` ``           | `{\tt kód}`                      |
| Blok kódu (fenced)    | `\begtt` ... `\endtt`            |
| Odstavec              | prázdný řádek mezi odstavci      |
| `[text](url)`         | `\url{url}` nebo `\ulink[url]{text}` |
| `![alt](src)`         | `\picw=... \inspic src`          |
| Nečíslovaný seznam    | `\begitems` ... `* položka` ... `\enditems` |
| Číslovaný seznam      | `\begitems \style n` ... `\enditems` |
| Horizontální linka    | `\noindent\hrule`                |
| Citace (`>`)          | `\begcitation` ... `\endcitation` (vlastní makro) nebo odsazení |
| Tabulka (GFM)         | `\table{...}{...}` – viz níže    |
| Obrázek               | `\picw=Xcm \inspic src` – viz níže |

### Tabulky

Zdrojem je GFM tabulka (rozšíření CommonMark, podporováno pulldown-cmark s feature `ENABLE_TABLES`).

Zarovnání sloupců z GFM (`:---`, `:---:`, `---:`) se mapuje na OpTeX specifikátory `l`, `c`, `r`.
Záhlaví tabulky se ukončí `\crli` (OpTeX příkaz pro řádek s linkou pod ním):

```tex
\table{lcr}{
Hlavička 1 & Hlavička 2 & Hlavička 3 \crli
Buňka A & Buňka B & Buňka C \cr
}
```

### Obrázky

Při konverzi se obrázek fyzicky změří (crate `imagesize` – čte pouze hlavičku souboru, bez dekódování).
Pixelové rozměry se převedou na centimetry při předpokládaném rozlišení 96 DPI:

```
šířka_cm = pixel_width / 96 * 2.54
```

Pokud `šířka_cm > 15.0` (přibližná sazební šířka pro A4), použije se `\picw=\hsize`.
Jinak se použije vypočtená hodnota: `\picw=X.XXcm`.

Pokud soubor obrázku neexistuje nebo ho nelze změřit, použije se fallback `\picw=\hsize` a vypíše se varování na stderr.

```tex
\picw=12.34cm \inspic cesta/k/obrazku.png
```

## České typografické konvence

Tyto transformace se aplikují na textový obsah při renderování.

### Uvozovky

Markdown nemá standardní zápis pro uvozovky – konvertuj ASCII uvozovky na české:

- `"text"` nebo `„text"` → `\uv{text}` (OpTeX makro, vysází „text")
- Vnořené uvozovky: `"vnější ‚vnitřní' text"` → `\uv{vnější \uv{vnitřní} text}`

### Pomlčka a spojovník

| Vstup       | Výstup TeX | Význam                        |
|-------------|------------|-------------------------------|
| `-`         | `-`        | spojovník (např. česko-slovenský) |
| ` -- ` nebo `–` | `--`  | pomlčka (en dash), oddělení vět |
| ` --- ` nebo `—` | `---` | dlouhá pomlčka (em dash), řídce |

Pomlčka obklopená mezerami dostane nezlomitelnou mezeru před ní: `~--` (aby pomlčka nezůstala na konci řádku).

### Nezlomitelná mezera

Za jednopísmennými předložkami a spojkami vkládej `~` (nezlomitelnou mezeru v TeXu), aby nepřišly na konec řádku:

- Předložky: `v`, `z`, `s`, `k`, `u`, `o`, `i` (a jejich varianty `ve`, `ze`, `se`, `ke`)
- Spojky: `a`, `i`, `o`
- Vzor: slovo délky 1–2 znaky, za kterým následuje mezera na začátku nebo uvnitř věty

Příklad: `v lese` → `v~lese`, `k dispozici` → `k~dispozici`

Tato transformace se aplikuje pouze na textové uzly (ne uvnitř `\tt`, URL, atd.).

### Tři tečky (výpustka)

`...` → `\dots` (správné typografické provedení výpustky v TeXu)

### Záhlaví dokumentu

OpTeX je LuaTeX formát – preambule neobsahuje `\input optex`. Generovaný soubor začíná:

```tex
\fontfam[LM]
\uselanguage{czech}
```

Kompilace: `optex dokument.tex`

## Struktura projektu knihy

Doporučená adresářová struktura pro knihu:

```
kniha/
  metadata.toml        # metadata a nastavení sazby
  chapters/
    00_uvod.md
    01_prvni_kapitola.md
    02_druha_kapitola.md
    ...
  obrazky/
  hyphenation.txt
```

Kapitoly se zpracují v abecedním/číselném pořadí názvů souborů. Konvertor přijímá buď **jeden MD soubor** nebo **adresář** (hledá `metadata.toml` + `kapitoly/*.md`).

### metadata.toml

```toml
[book]
title  = "Book Title"
author = "First Last"
year   = 2026
isbn   = "978-80-000-0000-0"   # optional

[typesetting]
paper     = "a4"        # a4 | b5 | a5 | letter
font      = "Pagella"   # font family name for \fontfam
base_size = "11pt"      # base font size
paragraph = "indent"    # indent | noindent (first paragraph after heading)

# margins in mm (optional; defaults applied when paper is set)
margin_left   = 35
margin_right  = 25
margin_top    = 30
margin_bottom = 30

# running headers / footers (optional)
header = "{author} & {chapter} &"   # left & centre & right
footer = "& \\folio &"

[paths]
images      = "images"           # image directory (relative to metadata.toml)
hyphenation = "hyphenation.txt"  # optional, overrides --hyphenation-dict

[style]
name = "book"   # built-in: minimal | book | academic | manual
                # or a path: "./styles/my-style.tex"
```

### Styly

Styl je TeX snippet (`\input`ovaný za preambulí) který může předefinovat OpTeX makra, nastavit fonty, okraje apod.

**Pořadí hledání podle názvu** (bez přípony `.tex`):
1. `./styles/<název>.tex` – lokálně v projektu knihy
2. `~/.config/md2optex/styles/<název>.tex` – uživatelské styly
3. Vestavěné styly – embedded v binárce (`include_str!`)

**Vestavěné styly:**

| Name       | Description                                                  |
|------------|--------------------------------------------------------------|
| `minimal`  | A4 defaults, no further customisation                        |
| `book`     | fiction / prose — Pagella, B5, symmetric margins, folio      |
| `academic` | academic publication — Termes, A4, wider outer margin        |
| `manual`   | technical docs — Heros sans-serif, A4, smaller verbatim      |

Výchozí styl pokud není nic uvedeno: `minimal`.

Hodnoty z `metadata.toml` se promítnou do preambule vygenerovaného TeX souboru:

```tex
\fontfam[Palatino]
\uselanguage{czech}
\typosize[11/13]

\tit Název knihy
\author Jméno Příjmení
\maketitle
```

## Použití (plánované CLI rozhraní)

```
md2optex [OPTIONS] [INPUT]

Arguments:
  [INPUT]  Vstupní Markdown soubor (výchozí: stdin)

Options:
  -o, --output <FILE>          Výstupní TeX soubor (výchozí: stdout)
      --hyphenation-dict <FILE> Slovník dělení slov (viz níže)
      --dpi <N>                Rozlišení obrázků v DPI pro výpočet fyzické velikosti (výchozí: 96)
  -h, --help                   Zobrazí nápovědu
  -V, --version                Zobrazí verzi
```

### Slovník dělení slov (`--hyphenation-dict`)

Plaintext soubor, každý řádek jedno slovo s dělicími místy označenými pomlčkou:

```
ma-na-ge-ment
nej-ne-prav-dě-po-dob-něj-ší
Če-sko-slo-ven-sko
soft-ware
```

Prázdné řádky a řádky začínající `#` jsou ignorovány (komentáře).

Konvertor z tohoto souboru sestaví `\hyphenation{}` blok a vloží ho do preambule:

```tex
\hyphenation{
  ma-na-ge-ment
  nej-ne-prav-dě-po-dob-něj-ší
}
```

Pokud soubor nelze přečíst, konvertor skončí s chybou (ne tichým fallbackem).

## Implementation plan

### Done ✓
- CLI: `--output`, `--hyphenation-dict`, `--dpi`, `--style`, stdin/stdout
- Renderer: headings (H1–H4), paragraphs, bold, italic, inline code, fenced code blocks
- Renderer: unordered and ordered lists, block quotes, horizontal rule
- Renderer: links (`\ulink`), images (with `imagesize` measurement), tables (`\crli` header)
- Renderer: strikethrough `~~text~~` → `\strike{text}` (macro defined in preamble)
- Renderer: footnotes `[^1]` → `\fnote{text}` (two-pass: pre-scan + render)
- Renderer: task lists `- [x]` / `- [ ]` → `[{\tt x}]` / `[ ]`
- Renderer: HTML passthrough discarded
- Typo: Czech quotes (`\uv{}`), dashes (`~--`), non-breaking spaces, ellipsis (`\dots`)
- Typo: Unicode dashes `–`/`—` → `--`/`---`
- Book directory input: `metadata.toml` + `chapters/*.md` in alphabetical order
- Metadata: title, author, year, isbn, copyright, toc (front/back/true/false)
- Metadata: font, base_size, paper, margins, header, footer, paragraph, toc_depth, paths.images
- Style system: lookup chain `./styles/` → `~/.config/md2optex/styles/` → built-in
- Built-in styles: `minimal`, `book`, `academic`, `manual`
- Front matter: title page (`\maketitle`), colophon/verso, TOC with odd-page guarantee
- Page numbering reset to 1 after front matter
- `nonum`/`toc_depth`: style `book` → `\nonum` on all headings, `toc_depth=1` (chapters only)
- Hyphenation dictionary → `\hyphenation{}` block
- Image path prefix: `paths.images` applied when resolving relative image paths
- Integration tests: 69 tests in `tests/render.rs`
- Pandoc-compatible attributes: headings (`{.unnumbered}`, `{.unlisted}`, `{#id}`), code blocks (`{.numberLines}`, `startFrom="N"`), tables (`{.longtable}`, `colwidths="X% Y%"`), images (`{width=...}`), spans (`{.smallcaps}`, `{.underline}`, `{.mark}`)

### Missing / not yet implemented

#### Pandoc-compatible attributes (not yet implemented)
- [ ] **Divs** — `::: {.warning}` … `:::` — custom block containers
- [ ] **Inline code attrs** — `` `code`{.python} ``
- [ ] **Link attrs** — `[text](url){target=_blank}`

#### Renderer / Typo
- [ ] **Nested single quotes** — `‚vnitřní'` (U+201A / U+2018) → `\uv{vnitřní}`
- [ ] **Math** — `$...$` / `$$...$$` — pulldown-cmark doesn't parse it; workaround needed
- [ ] **Definition lists** — not supported by pulldown-cmark


#### Style `book` — implemented ✓

- [x] `\widowpenalty=10000 \clubpenalty=10000` — prevent widows/orphans
- [x] `\frenchspacing` — equal spacing after periods (Czech standard)
- [x] `\emergencystretch=3em` — looser line breaking
- [x] `\openright` — every chapter starts on a recto (odd) page; blank verso inserted via `\bgroup\footline={}\headline={}\null\vfil\_supereject\egroup`
- [x] Ornament under chapter title — thin centred rule (`\vrule height0.4pt width3em`)
- [x] Running headers — even: `\folio + book title`; odd: `chapter title + \folio`; chapter title stored via `\_mark{#1}` in `\_printchap`, folio moved from footer to header

## Workflow

Po každé dílčí funkční změně (nový modul, nová feature, oprava bugu) ihned vytvoř commit s výstižným popisem co bylo přidáno/opraveno. Nečekej na větší celky.

## Build & test

```bash
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```
