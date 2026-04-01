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

## Poznámky pod čarou

Text s poznámkou[^1] a další poznámkou[^2].

[^1]: Toto je první poznámka pod čarou s delším vysvětlením.
[^2]: Druhá poznámka s `kódem` uvnitř.

## Obrázek

Obrázek se automaticky změří a vloží do sazby:

![Ukázkový obrázek](example.png)

Pokud soubor neexistuje, vypíše se varování na stderr a použije se `\picw=\hsize`.
