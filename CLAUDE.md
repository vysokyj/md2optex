# md2opmac

CLI nástroj v Rustu, který převádí Markdown na TeX s makry OPmac (OpTeX).

## Cíl projektu

Čte Markdown soubor (nebo stdin) a na stdout (nebo do souboru) vypisuje validní TeX zdrojový kód využívající makra OPmac.

## Konvence

- Jazyk kódu: Rust (edition 2021)
- Chybové zprávy a výstup do terminálu: česky nebo anglicky, konzistentně
- Knihovny: preferuj standardní crates (clap pro CLI, pulldown-cmark pro parsování MD)
- Testy: unit testy v příslušném modulu (`#[cfg(test)]`), integrační testy v `tests/`
- Formátování: `rustfmt` (výchozí konfigurace)
- Linting: `clippy` bez varování

## Architektura

```
src/
  main.rs       – vstupní bod, zpracování argumentů (clap)
  parser.rs     – parsování Markdown pomocí pulldown-cmark
  renderer.rs   – převod událostí parseru na OPmac TeX výstup
  error.rs      – vlastní chybové typy
tests/
  *.rs          – integrační testy (vstup MD → očekávaný TeX výstup)
```

## Mapování Markdown → OPmac

| Markdown              | OPmac TeX                        |
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

Zarovnání sloupců z GFM (`:---`, `:---:`, `---:`) se mapuje na OPmac specifikátory `l`, `c`, `r`.
Záhlaví tabulky se oddělí `\hline` nad i pod:

```tex
\table{lcr}{
\hline
Hlavička 1 & Hlavička 2 & Hlavička 3 \cr
\hline
Buňka A & Buňka B & Buňka C \cr
\hline
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

- `"text"` nebo `„text"` → `\uv{text}` (OPmac makro, vysází „text")
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

Generovaný TeX soubor začíná:

```tex
\input opmac
\language\czech   % české dělení slov
```

## Struktura projektu knihy

Doporučená adresářová struktura pro knihu:

```
kniha/
  metadata.toml        # metadata a nastavení sazby
  kapitoly/
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
[kniha]
nazev   = "Název knihy"
autor   = "Jméno Příjmení"
rok     = 2026
isbn    = "978-80-000-0000-0"   # volitelné

[sazba]
papir        = "a4"             # a4 | b5 | a5
font         = "palatino"       # název fontu pro \fontfam
zakladni_vel = "11pt"           # základní velikost písma
odstavec     = "indent"         # indent | noindent (první odstavec po nadpisu)

# okraje v mm (volitelné, jinak OPmac výchozí)
okraj_vlevo  = 35
okraj_vpravo = 25
okraj_nahore = 30
okraj_dole   = 30

# záhlaví a zápatí (volitelné)
zahlaví      = "{autor} & {nazev_kapitoly} &"   # levý & střed & pravý
zapati       = "& \\folio &"

[cesty]
obrazky      = "obrazky"        # adresář s obrázky (relativně k metadata.toml)
hyphenation  = "hyphenation.txt" # volitelné, přebije --hyphenation-dict

[styl]
styl = "kniha"                  # název vestavěného stylu nebo uživatelského (viz níže)
# styl = "./styles/muj-styl.tex"  # explicitní cesta
```

### Styly

Styl je TeX snippet (`\input`ovaný za preambulí) který může předefinovat OPmac makra, nastavit fonty, okraje apod.

**Pořadí hledání podle názvu** (bez přípony `.tex`):
1. `./styles/<název>.tex` – lokálně v projektu knihy
2. `~/.config/md2opmac/styles/<název>.tex` – uživatelské styly
3. Vestavěné styly – embedded v binárce (`include_str!`)

**Vestavěné styly:**

| Název      | Popis                                                        |
|------------|--------------------------------------------------------------|
| `kniha`    | beletrie – patičkový font, symetrické okraje, živá záhlaví  |
| `odborny`  | odborná publikace – širší vnější okraj pro poznámky          |
| `manual`   | technická dokumentace – výraznější bloky kódu, sans-serif    |
| `minimal`  | holé OPmac výchozí hodnoty bez úprav                         |

Výchozí styl pokud není nic uvedeno: `minimal`.

Hodnoty z `metadata.toml` se promítnou do preambule vygenerovaného TeX souboru:

```tex
\input opmac
\language\czech
\fontfam[palatino]
\typosize[11/13]
\bookfont

\tit Název knihy
\author Jméno Příjmení
\maketitle
```

## Použití (plánované CLI rozhraní)

```
md2opmac [OPTIONS] [INPUT]

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

## Workflow

Po každé dílčí funkční změně (nový modul, nová feature, oprava bugu) ihned vytvoř commit s výstižným popisem co bylo přidáno/opraveno. Nečekej na větší celky.

## Build & test

```bash
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```
