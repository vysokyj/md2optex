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
  -o, --output <FILE>           Výstupní TeX soubor (výchozí: stdout)
      --hyphenation-dict <FILE> Slovník dělení slov
      --dpi <N>                 Rozlišení obrázků pro výpočet fyzické velikosti [výchozí: 96]
      --style <NAME>            Přepis stylu (minimal | book | academic | manual, nebo cesta)
  -h, --help                    Zobrazí nápovědu
  -V, --version                 Zobrazí verzi
```

### Příklady

```bash
# Jeden soubor → stdout
md2optex dokument.md

# Soubor → soubor
md2optex dokument.md -o dokument.tex

# Ze stdin
cat dokument.md | md2optex -o dokument.tex

# Adresář knihy (viz níže)
md2optex kniha/ -o kniha.tex

# Se slovníkem dělení slov
md2optex dokument.md --hyphenation-dict hyphenation.txt

# Přepis stylu z příkazové řádky
md2optex dokument.md --style academic

# Kompilace do PDF
optex dokument.tex
```

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
| Odstavec | prázdný řádek |
| `[text](url)` | `\ulink[url]{text}` |
| `![alt](src)` | `\picw=X.XXcm \inspic src` |
| Nečíslovaný seznam | `\begitems` … `\enditems` |
| Číslovaný seznam | `\begitems \style n` … `\enditems` |
| `---` | `\noindent\hrule` |
| `> citace` | `\begcitation` … `\endcitation` |
| Tabulka (GFM) | `\table{lcr}{…}` |
| `text[^1]` + `[^1]: pozn.` | `text\fnote{pozn.}` |

### Obrázky

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

Pokud jsou v `[book]` vyplněna pole `year`, `author` nebo `isbn`, vygeneruje se automaticky rubová strana titulní strany s údaji o copyrightu a ISBN. Pole `copyright` přepíše automaticky generovaný řádek `© rok autor`.

## Sazební styly

### Vestavěné styly

| Název | Popis |
|---|---|
| `minimal` | A4, výchozí písmo LM, jednoduché zápatí s číslem stránky |
| `book` | B5, oboustranné okraje, Pagella, záhlaví prázdné, zápatí s foliem |
| `academic` | A4, Termes, širší vnější okraj |
| `manual` | A4, Heros (bezpatkové), menší verbatim |

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
- Rubová strana (str. 2): copyright + ISBN (nebo prázdná), bez čísla stránky
- Obsah (pokud `toc` není `false`): garantovaně lichá strana, bez záhlaví/zápatí

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
make examples     # vygeneruje PDF ukázky pro všechny styly (vyžaduje optex)
make book-sample  # zkompiluje příklad knihy (examples/book-sample/)
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
