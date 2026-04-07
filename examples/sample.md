---
title: Ukázkový dokument
author: Jan Novák
year: 2026
style: minimal
---

# Ukázkový dokument

Toto je ukázkový Markdown soubor pro testování nástroje md2optex.
Obsahuje příklady všech podporovaných konstrukcí.

## Formátování textu

Tento odstavec obsahuje **tučný text** a *kurzívu*. Lze také kombinovat:
**tučná _tučná kurzíva_ tučná**. Inline kód vypadá takto: `int main()`.
Přeškrtnutý text: ~~zastaralá funkce~~.

Druhý odstavec v tomto oddíle. Text může obsahovat `proměnné` nebo `funkce()`.

## České typografické konvence

Nástroj automaticky aplikuje české typografické pravidla.

Uvozovky: "toto je v uvozovkách" a také „toto je již česky".

Uvozovky se seznamem a pomlčkou:

- „Poslouchej svůj hlas — vypadá to na odpor, ale je v něm touha."
- „Druhá položka s pomlčkou — taky funguje."

Pomlčka odděluje části věty -- například takto. Nebo pomocí em-dash --- takhle.

Jednopísmenné předložky: v lese, z kopce, s kamarádem, k domovu, u řeky.
Spojky: Jan a Marie, Petr i Pavel, slunce o půlnoci.

Výpustka na konci věty...

### Podpodnadpis s příkladem

Text na úrovni třetí nadpisové úrovně. Zde je odkaz na
[stránku projektu OpTeX](https://petr.olsak.net/optex/) a ještě jeden
[odkaz s titulkem](https://example.com).

## Matematika

Inline matematika: Platí $E = mc^2$ a obvod kružnice je $C = 2\pi r$.

Bloková matematika na samostatném řádku:

$$\int_0^\infty e^{-x}\,dx = 1$$

Více rovnic:

$$\sum_{n=1}^{\infty} \frac{1}{n^2} = \frac{\pi^2}{6}$$

## Horní a dolní index

Horní index (superscript): x ^2^, e ^iπ^, 1 ^st^ place.

Dolní index (subscript): ~2~, CO ~2~, H ~2~ O.

Poznámka: `^text^` a `~text~` musí být odděleny mezerou od okolního textu.

## Definiční seznam

Markdown
: Odlehčený značkovací jazyk navržený pro snadnou čitelnost v textové podobě.

OpTeX
: Moderní plainTeX formát od Petra Olšáka běžící nad LuaHBTeXem.
  Plná podpora UTF-8, moderní fonty, česká typografie.

LuaTeX
: Rozšíření TeXu s interpretem jazyka Lua pro programovatelné sazby.

## Seznamy

Nečíslovaný seznam:

* první položka
* druhá položka s **tučným** textem
* třetí položka s `kódem`

Číslovaný seznam:

1. první krok
2. druhý krok
3. třetí krok

Úkolový seznam:

- [x] Implementovat matematiku
- [x] Přidat definičníí seznam
- [ ] Zvážit podporu citací (BibTeX)

## Blok kódu

Příklad Rust kódu:

```rust
fn main() {
    println!("Ahoj, světe!");
    let x = 42;
    let y = x * 2;
    println!("Výsledek: {}", y);
}
```

## Raw TeX passthrough

Bloky označené jako `tex` nebo `optex` se vloží přímo do výstupu bez escapování.
Hodí se pro vlastní OpTeX makra nebo příkazy:

```tex
\vskip 2cm
\noindent\hrule\vskip 1mm\hrule
\vskip 2cm
```

Inline raw TeX nelze — použijte raw blok s jedním příkazem.

## Citace

> „Filozofie je příprava na smrt," napsal Platón.
> Přijmi ji, i když se ti zdá teoretická.

> Druhý odstavec citace obsahuje *formátování* a `kód`.

## Horizontální linka

Text před oddělovačem.

---

Text po oddělovači.

## Tabulka

Ukázka GFM tabulky se zarovnáním sloupců:

| Název       | Typ     | Hodnota |
|:------------|:-------:|--------:|
| alfa        | string  |       1 |
| beta        | integer |      42 |
| gama        | boolean |       0 |

## Tabulka ASCII znaků

Kompletní tabulka tisknutelných ASCII znaků (32--126) s kódem, znakem, popisem a kategorií:

| Dec | Hex | Oct | Znak | Popis | Kategorie |
|----:|----:|----:|:----:|:------|:----------|
| 32 | 20 | 040 | | Space | Whitespace |
| 33 | 21 | 041 | ! | Exclamation mark | Punctuation |
| 34 | 22 | 042 | " | Quotation mark | Punctuation |
| 35 | 23 | 043 | # | Number sign | Symbol |
| 36 | 24 | 044 | $ | Dollar sign | Symbol |
| 37 | 25 | 045 | % | Percent sign | Symbol |
| 38 | 26 | 046 | & | Ampersand | Symbol |
| 39 | 27 | 047 | ' | Apostrophe | Punctuation |
| 40 | 28 | 050 | ( | Left parenthesis | Bracket |
| 41 | 29 | 051 | ) | Right parenthesis | Bracket |
| 42 | 2A | 052 | * | Asterisk | Symbol |
| 43 | 2B | 053 | + | Plus sign | Math |
| 44 | 2C | 054 | , | Comma | Punctuation |
| 45 | 2D | 055 | - | Hyphen-minus | Punctuation |
| 46 | 2E | 056 | . | Full stop | Punctuation |
| 47 | 2F | 057 | / | Solidus | Symbol |
| 48 | 30 | 060 | 0 | Digit zero | Digit |
| 49 | 31 | 061 | 1 | Digit one | Digit |
| 50 | 32 | 062 | 2 | Digit two | Digit |
| 51 | 33 | 063 | 3 | Digit three | Digit |
| 52 | 34 | 064 | 4 | Digit four | Digit |
| 53 | 35 | 065 | 5 | Digit five | Digit |
| 54 | 36 | 066 | 6 | Digit six | Digit |
| 55 | 37 | 067 | 7 | Digit seven | Digit |
| 56 | 38 | 070 | 8 | Digit eight | Digit |
| 57 | 39 | 071 | 9 | Digit nine | Digit |
| 58 | 3A | 072 | : | Colon | Punctuation |
| 59 | 3B | 073 | ; | Semicolon | Punctuation |
| 60 | 3C | 074 | < | Less-than sign | Math |
| 61 | 3D | 075 | = | Equals sign | Math |
| 62 | 3E | 076 | > | Greater-than sign | Math |
| 63 | 3F | 077 | ? | Question mark | Punctuation |
| 64 | 40 | 100 | @ | Commercial at | Symbol |
| 65 | 41 | 101 | A | Latin capital letter A | Uppercase |
| 66 | 42 | 102 | B | Latin capital letter B | Uppercase |
| 67 | 43 | 103 | C | Latin capital letter C | Uppercase |
| 68 | 44 | 104 | D | Latin capital letter D | Uppercase |
| 69 | 45 | 105 | E | Latin capital letter E | Uppercase |
| 70 | 46 | 106 | F | Latin capital letter F | Uppercase |
| 71 | 47 | 107 | G | Latin capital letter G | Uppercase |
| 72 | 48 | 110 | H | Latin capital letter H | Uppercase |
| 73 | 49 | 111 | I | Latin capital letter I | Uppercase |
| 74 | 4A | 112 | J | Latin capital letter J | Uppercase |
| 75 | 4B | 113 | K | Latin capital letter K | Uppercase |
| 76 | 4C | 114 | L | Latin capital letter L | Uppercase |
| 77 | 4D | 115 | M | Latin capital letter M | Uppercase |
| 78 | 4E | 116 | N | Latin capital letter N | Uppercase |
| 79 | 4F | 117 | O | Latin capital letter O | Uppercase |
| 80 | 50 | 120 | P | Latin capital letter P | Uppercase |
| 81 | 51 | 121 | Q | Latin capital letter Q | Uppercase |
| 82 | 52 | 122 | R | Latin capital letter R | Uppercase |
| 83 | 53 | 123 | S | Latin capital letter S | Uppercase |
| 84 | 54 | 124 | T | Latin capital letter T | Uppercase |
| 85 | 55 | 125 | U | Latin capital letter U | Uppercase |
| 86 | 56 | 126 | V | Latin capital letter V | Uppercase |
| 87 | 57 | 127 | W | Latin capital letter W | Uppercase |
| 88 | 58 | 130 | X | Latin capital letter X | Uppercase |
| 89 | 59 | 131 | Y | Latin capital letter Y | Uppercase |
| 90 | 5A | 132 | Z | Latin capital letter Z | Uppercase |
| 91 | 5B | 133 | [ | Left square bracket | Bracket |
| 92 | 5C | 134 | \ | Reverse solidus | Symbol |
| 93 | 5D | 135 | ] | Right square bracket | Bracket |
| 94 | 5E | 136 | ^ | Circumflex accent | Symbol |
| 95 | 5F | 137 | _ | Low line | Symbol |
| 96 | 60 | 140 | ` | Grave accent | Symbol |
| 97 | 61 | 141 | a | Latin small letter a | Lowercase |
| 98 | 62 | 142 | b | Latin small letter b | Lowercase |
| 99 | 63 | 143 | c | Latin small letter c | Lowercase |
| 100 | 64 | 144 | d | Latin small letter d | Lowercase |
| 101 | 65 | 145 | e | Latin small letter e | Lowercase |
| 102 | 66 | 146 | f | Latin small letter f | Lowercase |
| 103 | 67 | 147 | g | Latin small letter g | Lowercase |
| 104 | 68 | 150 | h | Latin small letter h | Lowercase |
| 105 | 69 | 151 | i | Latin small letter i | Lowercase |
| 106 | 6A | 152 | j | Latin small letter j | Lowercase |
| 107 | 6B | 153 | k | Latin small letter k | Lowercase |
| 108 | 6C | 154 | l | Latin small letter l | Lowercase |
| 109 | 6D | 155 | m | Latin small letter m | Lowercase |
| 110 | 6E | 156 | n | Latin small letter n | Lowercase |
| 111 | 6F | 157 | o | Latin small letter o | Lowercase |
| 112 | 70 | 160 | p | Latin small letter p | Lowercase |
| 113 | 71 | 161 | q | Latin small letter q | Lowercase |
| 114 | 72 | 162 | r | Latin small letter r | Lowercase |
| 115 | 73 | 163 | s | Latin small letter s | Lowercase |
| 116 | 74 | 164 | t | Latin small letter t | Lowercase |
| 117 | 75 | 165 | u | Latin small letter u | Lowercase |
| 118 | 76 | 166 | v | Latin small letter v | Lowercase |
| 119 | 77 | 167 | w | Latin small letter w | Lowercase |
| 120 | 78 | 170 | x | Latin small letter x | Lowercase |
| 121 | 79 | 171 | y | Latin small letter y | Lowercase |
| 122 | 7A | 172 | z | Latin small letter z | Lowercase |
| 123 | 7B | 173 | { | Left curly bracket | Bracket |
| 124 | 7C | 174 | \| | Vertical line | Symbol |
| 125 | 7D | 175 | } | Right curly bracket | Bracket |
| 126 | 7E | 176 | ~ | Tilde | Symbol |
{.longtable}

## Poznámky pod čarou

Text s poznámkou[^1] a další poznámkou[^2].

[^1]: Toto je první poznámka pod čarou s delším vysvětlením.
[^2]: Druhá poznámka s `kódem` uvnitř.

## Obrázek

Obrázek se automaticky změří a vloží do sazby:

![Ukázkový obrázek](sample.png)

Pokud soubor neexistuje, vypíše se varování na stderr a použije se `\picw=\hsize`.

## Pandoc-kompatibilní atributy

### Atributy nadpisů

#### Nečíslovaný nadpis {.unnumbered}

Tento nadpis nemá číslo, protože má atribut `{.unnumbered}`.

#### Nadpis mimo obsah {.unlisted}

Tento nadpis se nezobrazí v obsahu dokumentu.

#### Nadpis s identifikátorem {#muj-nadpis}

Na tento nadpis se lze odkázat pomocí `\ref[muj-nadpis]`.

### Číslované bloky kódu

Kód s číslováním řádků:

```python {.numberLines}
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)
```

Kód s číslováním od řádku 10:

```rust {.numberLines startFrom="10"}
fn main() {
    let x = 42;
    println!("Odpověď: {}", x);
}
```

### Longtable (tabulka přes stránky)

Tabulka ASCII znaků je příliš dlouhá, aby se vešla na jednu stránku.
Přidáním `{.longtable}` za tabulku povolíme její zalomení přes stránky:

| Dec | Hex | Znak | Popis |
|----:|----:|:----:|:------|
| 33 | 21 | ! | Vykřičník |
| 34 | 22 | " | Uvozovka |
| 35 | 23 | # | Křížek |
| 36 | 24 | $ | Dolar |
| 37 | 25 | % | Procento |
| 38 | 26 | & | Ampersand |
| 39 | 27 | ' | Apostrof |
| 40 | 28 | ( | Levá závorka |
| 41 | 29 | ) | Pravá závorka |
| 42 | 2A | * | Hvězdička |
{.longtable}

### Span atributy

Text s [kapitálkami]{.smallcaps} a [podtrženým slovem]{.underline} a [zvýrazněným textem]{.mark} ve větě.
