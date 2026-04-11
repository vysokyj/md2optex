# md2optex

Převodník Markdown → TeX pro [OpTeX](https://petr.olsak.net/optex/).

Čte Markdown soubor (nebo stdin) a vypisuje validní OpTeX zdrojový kód připravený pro překlad pomocí `optex` (LuaHBTeX formát). Podporuje českou typografii, knižní strukturu s metadaty a vestavěné sazební styly.

## Požadavky

- Rust 1.85+ (edition 2024), `cargo`
- OpTeX: balíčky `texlive-basic`, `texlive-luatex`, `texlive-langczechslovak`

```bash
# Arch Linux
sudo pacman -S texlive-basic texlive-luatex texlive-langczechslovak
```

## Instalace

```bash
git clone https://github.com/vas-login/md2optex
cd md2optex
make install          # cargo install --path . → ~/.cargo/bin/md2optex
```

## Použití

```
md2optex [OPTIONS] [INPUT]

Arguments:
  [INPUT]  Vstupní Markdown soubor nebo adresář s knihou (výchozí: stdin)

Options:
  -o, --output <FILE>           Výstupní soubor. Přípona určuje režim:
                                  .tex → TeX zdroj
                                  .pdf → přímo PDF (md2optex spustí optex sám)
                                (výchozí: TeX na stdout)
      --hyphenation-dict <FILE> Slovník dělení slov
      --dpi <N>                 Rozlišení obrázků pro výpočet fyzické velikosti [výchozí: 96]
      --style <NAME>            Přepis stylu (minimal | book | academic | manual, nebo cesta)
  -h, --help                    Zobrazí nápovědu
  -V, --version                 Zobrazí verzi
```

### Příklady

```bash
# Jeden soubor → stdout (cesty k obrázkům se nepřepisují)
md2optex dokument.md

# Soubor → TeX vedle zdroje (relativní cesty k obrázkům)
md2optex dokument.md -o dokument.tex

# Soubor → TeX jinam (absolutní cesty k obrázkům)
md2optex dokument.md -o /tmp/build/dokument.tex

# Soubor → přímo PDF (md2optex zavolá `optex` v dočasném adresáři)
md2optex dokument.md -o dokument.pdf

# Ze stdin
cat dokument.md | md2optex -o dokument.tex

# Adresář knihy (viz níže) → TeX nebo PDF
md2optex kniha/ -o kniha.tex
md2optex kniha/ -o kniha.pdf

# Se slovníkem dělení slov
md2optex dokument.md --hyphenation-dict hyphenation.txt

# Přepis stylu z příkazové řádky
md2optex dokument.md --style academic
```

### Cesty k obrázkům

md2optex vybírá strategii podle toho, kam jde výstup:

- **Bez `-o`** (stdout) — cesty k obrázkům zůstávají přesně tak, jak jsou
  napsané v MD. Uživatel si odpovídá za to, že optex je v příštím kroku
  najde.
- **`-o out.tex` vedle zdroje** (`parent(out.tex)` == adresář zdroje) —
  cesty jsou **relativní**, TeX zůstane přenositelný a čitelný.
- **`-o out.tex` jinam** — cesty se rozvinou na **absolutní**, aby TeX
  fungoval z libovolného místa.
- **`-o out.pdf`** — md2optex vygeneruje TeX do dočasného adresáře,
  spustí `optex` dvakrát (pro TOC) a výsledné PDF zkopíruje na cíl.
  Dočasný adresář se uklidí automaticky. Vyžaduje `optex` v `PATH`.

## Mapování Markdown → OpTeX

| Markdown | OpTeX |
|---|---|
| `# Nadpis` | `\chap Nadpis` |
| `## Podnadpis` | `\sec Podnadpis` |
| `### Podpodnadpis` | `\secc Podpodnadpis` |
| `**tučně**` | `{\bf tučně}` |
| `*kurzíva*` | `{\it kurzíva}` |
| `` `kód` `` | `{\tt kód}` |
| `~~přeškrtnuté~~` | `\strike{přeškrtnuté}` |
| Blok kódu (fenced) | `\begtt` … `\endtt` |
| ` ```tex ` nebo ` ```optex ` | raw TeX passthrough (bez escapování) |
| `$vzorec$` | `$vzorec$` (inline math, passthrough) |
| `$$vzorec$$` | `$$vzorec$$` (display math, passthrough) |
| `x ^2^` | `\tsuper{2}` (superscript, mezery nutné) |
| `~2~` | `\tsub{2}` (subscript) |
| Odstavec | prázdný řádek |
| `[text](url)` | `\ulink[url]{text}` |
| `![alt](src)` | `\picw=X.XXcm \inspic src` |
| Nečíslovaný seznam | `\begitems` … `\enditems` |
| Číslovaný seznam | `\begitems \style n` … `\enditems` |
| `---` | `\noindent\hrule` |
| `> citace` | `\begcitation` … `\endcitation` |
| Tabulka (GFM) | `\table{lcr}{…}` |
| `text[^1]` + `[^1]: pozn.` | `text\fnote{pozn.}` |
| `- [x] hotovo` | `* [x] hotovo` (zaškrtnutý checkbox) |
| `- [ ] todo` | `* [ ] todo` (prázdný checkbox) |
| Definiční seznam | `{\bf Pojem}\par` + odsazená definice |

### Obrázky

Pokud je v `[paths]` nastaven klíč `images`, hledá se obrázek nejprve v tomto adresáři. Tím lze v Markdownu psát jen název souboru bez prefixu:

```md
![Foto](foto.jpg)   # hledá se v images/foto.jpg
```

Fyzická šířka se vypočítá z pixelových rozměrů a zadaného DPI:

```
šířka_cm = pixel_width / dpi * 2.54
```

Pokud šířka přesáhne 15 cm, použije se `\picw=\hsize`. Pokud soubor obrázku neexistuje nebo ho nelze změřit, použije se rovněž `\picw=\hsize` a na stderr se vypíše varování.

## České typografické konvence

Automaticky aplikované transformace na textový obsah:

| Vstup | Výstup | Popis |
|---|---|---|
| `"text"` nebo `„text"` | `\uv{text}` | české uvozovky |
| ` -- ` nebo `–` | `~--` | pomlčka s nezlomitelnou mezerou |
| ` --- ` nebo `—` | `~---` | dlouhá pomlčka |
| `v lese`, `k domovu` | `v~lese`, `k~domovu` | nezlomitelná mezera za předložkami/spojkami |
| `...` | `\dots` | výpustka |

Nezlomitelná mezera se vkládá za jednopísmenné předložky (`v`, `z`, `s`, `k`, `u`, `o`, `i`) a jejich dvoupísmenné varianty (`ve`, `ze`, `se`, `ke`), a za spojky `a`, `i`, `o`.

## Struktura knihy

Místo jednoho souboru lze předat celý adresář:

```
kniha/
  metadata.toml
  chapters/
    00_uvod.md
    01_prvni_kapitola.md
    02_druha_kapitola.md
  images/
  hyphenation.txt
```

Kapitoly jsou zpracovány v abecedním pořadí názvů souborů. `metadata.toml` je volitelný.

## YAML front matter (jednosouborový režim)

Pro jednoduché dokumenty bez `metadata.toml` lze metadata uvést přímo na začátku Markdown souboru ve formátu YAML front matter:

```yaml
---
title: Název dokumentu
author: Jméno Příjmení
year: 2026
isbn: 978-80-000-0000-0
style: academic
---
```

Podporovaná pole: `title`, `author`, `year` (nebo `date`), `isbn`, `style`. Automaticky se vygeneruje titulní strana a záhlaví — stejně jako s `metadata.toml`. V adresářovém režimu (s `metadata.toml`) se YAML front matter ignoruje.

## metadata.toml

Úplný příklad se všemi podporovanými klíči:

```toml
[book]
title     = "Název knihy"
author    = "Jméno Příjmení"
year      = 2026
isbn      = "978-80-000-0000-0"
copyright = "© 2026 Jméno Příjmení"  # volitelné; jinak se generuje z year + author
toc       = true        # true = výchozí dle stylu, "front" / "back" = explicitní

[typesetting]
paper      = "a4"       # a4 | b5 | a5 | letter
font       = "Pagella"  # název rodiny pro \fontfam
base_size  = "11pt"     # základní velikost písma
paragraph  = "noindent" # noindent = \parindent=0pt; výchozí = odsazení
margin_left   = 35      # okraje v mm (volitelné)
margin_right  = 25
margin_top    = 30
margin_bottom = 30
header = "{author} & {title} & {folio}"  # záhlaví: levá & střed & pravá část
footer = "& \folio &"                    # zápatí: levá & střed & pravá část
toc_depth = 1    # hloubka obsahu: 1 = jen kapitoly, 2 = + sekce (výchozí pro book = 1, jinak = vše)

[paths]
images      = "images"           # adresář s obrázky (relativně k metadata.toml)
hyphenation = "hyphenation.txt"  # slovník dělení slov

[style]
name = "book"   # minimal | book | academic | manual, nebo cesta k .tex souboru
```

### Záhlaví a zápatí

Šablony `header` a `footer` se dělí znakem `&` na tři části (levá / střed / pravá). Podporované zástupné symboly:

| Symbol | Výstup |
|---|---|
| `{author}` | jméno autora |
| `{title}` | název knihy |
| `{folio}` | číslo stránky (`\folio`) |

### Umístění obsahu (`toc`)

| Hodnota | Chování |
|---|---|
| `toc = true` | výchozí dle stylu (styl `book` → vzadu, ostatní → vpředu) |
| `toc = "front"` | obsah vždy vpředu (za titulní stranou) |
| `toc = "back"` | obsah vždy vzadu (na konci dokumentu) |
| `toc = false` nebo vynecháno | bez obsahu |

### Kolofon / rubová strana titulu

Pokud jsou v `[book]` vyplněna pole `year`, `author` nebo `isbn`:

- **Ostatní styly**: údaje (copyright, ISBN) se vypisují na rubové straně titulu (str. 2).
- **Styl `book`**: rubová strana zůstane prázdná a kolofon (tiráž) se vygeneruje na **konci knihy** — za obsahem a TOC, jako poslední strana před `\bye`. Tiráž obsahuje název, autora, copyright a ISBN.

Pole `copyright` přepíše automaticky generovaný řádek `© rok autor`.

## Sazební styly

### Vestavěné styly

#### `minimal` (výchozí)

Jednoduchý styl pro rychlý převod bez zvláštních požadavků. A4, Latin Modern, číslo stránky v zápatí. Vhodný pro jednostránkové dokumenty, poznámky nebo jako základ pro vlastní styl.

#### `book`

Knižní sazba beletrie nebo prózy. B5, Pagella (Palatino), oboustranné symetrické okraje. Nadpisy jsou bez čísel, obsah se generuje vzadu a obsahuje jen kapitoly (`toc_depth=1`). Kapitoly začínají vždy na liché straně (`openright`). Živá záhlaví: sudá strana — název knihy, lichá strana — název kapitoly. Na konci se vygeneruje tiráž. Vhodný pro romány, povídkové sbírky, monografie.

#### `academic`

Akademické publikace, články, eseje. A4, Termes (Times New Roman), o něco širší vnější okraj pro poznámky nebo vazbu. Číslované nadpisy (výchozí OpTeX). Živá záhlaví: sudá strana — folio + jméno autora, lichá strana — název dokumentu + folio. Automatické číslované popisky:

- **Obrázky** — popisek z alt textu: `![Popisek obrázku](soubor.png)` → `\caption/f Popisek obrázku`
- **Tabulky** — odstavec hned za tabulkou začínající `: ` (Pandoc konvence) se stane popiskem (`\caption/t`):

```md
| Metoda | Přesnost |
|--------|----------|
| A      | 98 %     |

: Srovnání metod
```

Vhodný pro seminární práce, výzkumné zprávy, sborníky.

#### `manual`

Technická dokumentace. A4, Heros (Helvetica/sans-serif), verbatim bloky (`\begtt`) mají menší písmo aby se dlouhé příkazy lépe vešly. Vhodný pro uživatelské příručky, API dokumentaci, návody.

---

Výchozí styl pokud není nic uvedeno: `minimal`.

### Vlastní styly

Pořadí hledání podle názvu (bez přípony `.tex`):

1. `./styles/<název>.tex` — lokálně v adresáři projektu
2. `~/.config/md2optex/styles/<název>.tex` — uživatelské styly
3. Vestavěné styly — embedded v binárce

Styl je TeX snippet vložený za preambulí; může předefinovat OpTeX makra, nastavit fonty, okraje apod.

## Titulní strana

Pokud jsou v `[book]` vyplněna pole `title` a/nebo `author`, vygeneruje se automaticky titulní strana:

- Vycentrovaný název v 18pt tučném písmu
- Jméno autora v kurzívě
- Bez čísla stránky
- Rubová strana (str. 2): copyright + ISBN (nebo prázdná pro styl `book`), bez čísla stránky
- Obsah (pokud `toc` není `false`): garantovaně lichá strana, bez záhlaví/zápatí
- Číslování stránek se resetuje na 1 na začátku těla dokumentu

## Pandoc-kompatibilní atributy

md2optex podporuje Pandoc-kompatibilní syntaxi atributů `{#id .třída klíč=hodnota}`. Pulldown-cmark atributy nativně neparsuje — md2optex je detekuje a zpracovává vlastním pre/post-processingem.

### Atributy dle Pandoc specifikace

| Element | Syntaxe | Příklad | md2optex |
|---------|---------|---------|:--------:|
| Nadpisy | `# Text {attrs}` | `# Úvod {#intro .unnumbered}` | **ano** |
| Code blocks | `` ```lang {attrs} `` | `` ```python {.numberLines startFrom="10"} `` | **ano** |
| Obrázky | `![alt](url){attrs}` | `![Foto](f.png){width=8cm}` | **ano** |
| Tabulky | `{attrs}` na řádku za tabulkou | `{.longtable}` + auto šířky | **ano** |
| Spans | `[text]{attrs}` | `[text]{.smallcaps}` | **ano** |
| Divs | `::: {attrs}` … `:::` | `::: {.warning}` | --- |
| Inline code | `` `code`{attrs} `` | `` `x`{.python} `` | --- |
| Odkazy | `[text](url){attrs}` | `[link](url){target=_blank}` | --- |

### Speciální atributy dle Pandoc

**Nadpisy:**

| Atribut | Popis | md2optex |
|---------|-------|:--------:|
| `#id` | Identifikátor (label, kotva) | **ano** |
| `.unnumbered` nebo `-` | Potlačí číslování | **ano** |
| `.unlisted` | Vyloučí z obsahu (TOC) | **ano** |

**Code blocks:**

| Atribut | Popis | md2optex |
|---------|-------|:--------:|
| `.numberLines` / `.number-lines` | Číslování řádků | **ano** |
| `startFrom="N"` | Počáteční číslo řádku | **ano** |
| `.lineAnchors` / `.line-anchors` | Klikatelné kotvy řádků (HTML) | --- |

**Obrázky:**

| Atribut | Popis | md2optex |
|---------|-------|:--------:|
| `width=hodnota` | Šířka (cm, mm, px, %, in) | **ano** |
| `height=hodnota` | Výška | --- |
| `#id` | Label pro odkaz | --- |

**Tabulky:**

| Atribut | Popis | md2optex |
|---------|-------|:--------:|
| `.longtable` | Tabulka s lomením přes stránky | **ano** |
| *(automaticky)* | Šířky odvozené z poměrů pomlček v separátoru | **ano** |
| `#id` | Label pro odkaz | --- |

Šířky sloupců se odvozují automaticky z délky separátorů v GFM tabulce (à la Pandoc grid tables).
Pokud jsou separátory výrazně nestejné, šířky se nastaví proporcionálně:

```md
| Krátký |  Široký sloupec s delším textem  | Střední  |
|--------|----------------------------------|----------|
| a      | b                                | c        |
```

**Spans a inline formátování:**

| Atribut | Popis | md2optex |
|---------|-------|:--------:|
| `.smallcaps` | Kapitálky | **ano** |
| `.underline` | Podtržení | **ano** |
| `.mark` | Zvýraznění | **ano** |
| `{=format}` | Raw obsah pro daný formát | --- |

### Rozšíření md2optex (nad rámec Pandoc)

| Element | Atribut | Popis |
|---------|---------|-------|
| Code blocks | `` ```tex `` / `` ```optex `` | Raw TeX passthrough (bez escapování) |
| Code blocks | `` ```praxe `` | Callout blok "Z praxe" (styl `book`) |

## Slovník dělení slov

Plaintext soubor, každý řádek jedno slovo s dělicími místy označenými pomlčkou. Prázdné řádky a řádky začínající `#` jsou ignorovány.

```
# komentář
ma-na-ge-ment
soft-ware
nej-ne-prav-dě-po-dob-něj-ší
```

Výsledek v preambuli:

```tex
\hyphenation{
  ma-na-ge-ment
  soft-ware
}
```

## Vývoj

```bash
make build        # debug build
make test         # jednotkové a integrační testy
make check        # fmt + clippy + testy
make examples     # vygeneruje PDF ukázky pro všechny styly + book-sample (vyžaduje optex)
make clean        # smaže build artefakty
```

### Struktura projektu

```
src/
  main.rs      — vstupní bod, zpracování argumentů
  parser.rs    — (rezerva)
  renderer.rs  — převod MD událostí na OpTeX výstup
  metadata.rs  — deserializace metadata.toml
  styles.rs    — vestavěné styly (embed)
  typo.rs      — české typografické transformace
  error.rs     — vlastní chybové typy
src/styles/
  minimal.tex  — vestavěný styl
  book.tex
  academic.tex
  manual.tex
tests/
  render.rs    — integrační testy (MD → TeX výstup)
examples/
  sample.md          — ukázkový soubor pro testování stylů
  book-sample/       — ukázková kniha s metadata.toml
```

## Licence

MIT
