# md2optex

Převodník Markdown → TeX pro [OpTeX](https://petr.olsak.net/optex/).

Čte Markdown soubor (nebo stdin) a vypisuje validní OpTeX zdrojový kód připravený pro překlad pomocí `optex` (LuaHBTeX formát).

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
make install          # zkompiluje release build a zkopíruje do /usr/local/bin
```

Vlastní cílový adresář:

```bash
make install DESTDIR=~/.local/bin
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
| Blok kódu (fenced) | `\begtt` … `\endtt` |
| Odstavec | prázdný řádek |
| `[text](url)` | `\ulink[url]{text}` |
| `![alt](src)` | `\picw=X.XXcm \inspic src` |
| Nečíslovaný seznam | `\begitems` … `\enditems` |
| Číslovaný seznam | `\begitems \style n` … `\enditems` |
| `---` | `\noindent\hrule` |
| `> citace` | `\begcitation` … `\endcitation` |
| Tabulka (GFM) | `\table{lcr}{…}` |

### Obrázky

Fyzická šířka obrázku se vypočítá z pixelových rozměrů a zadaného DPI:

```
šířka_cm = pixel_width / dpi * 2.54
```

Pokud šířka přesáhne 15 cm (sazební šířka A4), použije se `\picw=\hsize`.
Pokud soubor obrázku neexistuje nebo ho nelze změřit, použije se rovněž `\picw=\hsize` a na stderr se vypíše varování.

## České typografické konvence

Automaticky aplikované transformace na textový obsah:

| Vstup | Výstup |
|---|---|
| `"text"` | `\uv{text}` |
| ` -- ` | `~-- ` (nezlomitelná mezera před pomlčkou) |
| ` --- ` | `~--- ` |
| `v lese`, `k domovu` | `v~lese`, `k~domovu` |
| `...` | `\dots` |

Nezlomitelná mezera se vkládá za jednopísmenné předložky (`v`, `z`, `s`, `k`, `u`, `o`) a jejich dvoupísmenné varianty (`ve`, `ze`, `se`, `ke`), a za spojky `a`, `i`, `o`.

## Struktura knihy

Místo jednoho souboru lze předat celý adresář:

```
kniha/
  metadata.toml
  kapitoly/
    00_uvod.md
    01_prvni_kapitola.md
    02_druha_kapitola.md
  obrazky/
```

Kapitoly jsou zpracovány v abecedním pořadí názvů souborů. `metadata.toml` je volitelný.

### metadata.toml

```toml
[kniha]
nazev  = "Název knihy"
autor  = "Jméno Příjmení"
rok    = 2026

[sazba]
papir        = "a4"
font         = "Palatino"
zakladni_vel = "11pt"
okraj_vlevo  = 35
okraj_vpravo = 25
okraj_nahore = 30
okraj_dole   = 30

[cesty]
obrazky    = "obrazky"
hyphenation = "hyphenation.txt"
```

### Slovník dělení slov

Plaintext soubor, každý řádek jedno slovo s dělicími místy označenými pomlčkou.
Prázdné řádky a řádky začínající `#` jsou ignorovány.

```
# komentář
ma-na-ge-ment
soft-ware
nej-ne-prav-dě-po-dob-něj-ší
```

## Vývoj

```bash
make build    # debug build
make test     # testy
make check    # fmt + clippy + testy
make clean    # smaže build artefakty i složku build/
```

### Rychlé vyzkoušení

Výstup se generuje do složky `build/` (ta je v `.gitignore` a lze ji kdykoli smazat).

```bash
make tex      # MD → TeX  (build/ukazka.tex)
make pdf      # MD → TeX → PDF  (build/ukazka.pdf), vyžaduje optex
make preview  # totéž + otevře PDF v prohlížeči
```

Vlastní vstupní soubor:

```bash
make pdf EXAMPLE=muj-dokument.md
```

## Licence

MIT
