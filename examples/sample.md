# Ukázkový dokument

Toto je ukázkový Markdown soubor pro testování nástroje md2opmac.
Obsahuje příklady všech podporovaných konstrukcí.

## Formátování textu

Tento odstavec obsahuje **tučný text** a *kurzívu*. Lze také kombinovat:
**tučná _tučná kurzíva_ tučná**. Inline kód vypadá takto: `int main()`.

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

## Seznamy

Nečíslovaný seznam:

* první položka
* druhá položka s **tučným** textem
* třetí položka s `kódem`

Číslovaný seznam:

1. první krok
2. druhý krok
3. třetí krok

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

Příklad bez specifikace jazyka:

```
$ cargo build --release
$ ./target/release/md2opmac vstup.md -o vystup.tex
```

## Citace

> „Filozofie je příprava na smrt," napsal Platón.
> Přijmi ji, i když se ti zdá teoretická.

> „Kdo se bojí, nesmí do lesa," říká staré přísloví.
> Ale „odvaha není absence strachu -- je to rozhodnutí, že něco jiného je důležitější než strach."

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

## Obrázek

Obrázek se automaticky změří a vloží do sazby:

![Ukázkový obrázek](example.png)

Pokud soubor neexistuje, vypíše se varování na stderr a použije se `\picw=\hsize`.
