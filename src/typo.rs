/// Aplikuje české typografické konvence na textový uzel.
pub fn apply(text: &str) -> String {
    let s = fix_ellipsis(text);
    let s = fix_dashes(&s);
    let s = fix_quotes(&s);
    let s = fix_nbsp(&s);
    s
}

/// `...` → `\dots`
fn fix_ellipsis(s: &str) -> String {
    s.replace("...", r"\dots")
}

/// Pomlčky: ` --- ` nebo ` -- ` → ` --` s nezlomitelnou mezerou před ní (`~--`)
fn fix_dashes(s: &str) -> String {
    // em dash (---) i en dash (--)
    let s = s.replace(" --- ", "~--- ");
    let s = s.replace(" -- ", "~-- ");
    s
}

/// Převede ASCII i české uvozovky na \uv{}.
/// ASCII `"`: první je otevírací, druhá zavírací.
/// Unicode „ (U+201E) → otevírací, " (U+201C) → zavírací.
fn fix_quotes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut open = false;

    for c in s.chars() {
        match c {
            '"' => {
                if !open {
                    result.push_str(r"\uv{");
                    open = true;
                } else {
                    result.push('}');
                    open = false;
                }
            }
            '\u{201E}' => {
                // „ – česká otevírací uvozovka
                result.push_str(r"\uv{");
                open = true;
            }
            '\u{201C}' => {
                // " – česká zavírací uvozovka
                result.push('}');
                open = false;
            }
            _ => result.push(c),
        }
    }
    // Nezavřená uvozovka – nech jak je (neměla by nastat)
    result
}

/// Nezlomitelná mezera za jednopísmennými předložkami a spojkami.
fn fix_nbsp(s: &str) -> String {
    // Předložky a spojky délky 1–2 znaky
    static PATTERNS: &[&str] = &[
        "ve ", "ze ", "se ", "ke ", "ku ",
        "v ", "z ", "s ", "k ", "u ", "o ", "i ", "a ",
    ];

    let mut result = s.to_owned();
    for pat in PATTERNS {
        let replacement = format!("{}~", pat.trim_end());
        result = replace_word_boundary(&result, pat, &replacement);
    }
    result
}

/// Nahradí `pat` za `replacement` pouze na slovní hranici (předcházený mezerou nebo začátkem).
fn replace_word_boundary(s: &str, pat: &str, replacement: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut rest = s;

    while let Some(pos) = rest.find(pat) {
        let before = &rest[..pos];
        // Platná hranice: začátek řetězce nebo předcházen mezerou / interpunkcí
        let is_boundary = pos == 0
            || before
                .chars()
                .last()
                .map_or(false, |c| c == ' ' || c == '\n' || c == '(');

        result.push_str(before);
        if is_boundary {
            result.push_str(replacement);
        } else {
            result.push_str(pat);
        }
        rest = &rest[pos + pat.len()..];
    }
    result.push_str(rest);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ellipsis() {
        assert_eq!(fix_ellipsis("a...b"), r"a\dotsb");
        assert_eq!(fix_ellipsis("konec..."), r"konec\dots");
    }

    #[test]
    fn test_dashes() {
        assert_eq!(fix_dashes("foo -- bar"), "foo~-- bar");
        assert_eq!(fix_dashes("foo --- bar"), "foo~--- bar");
    }

    #[test]
    fn test_quotes() {
        assert_eq!(fix_quotes(r#"řekl "ahoj" a šel"#), r"řekl \uv{ahoj} a šel");
    }

    #[test]
    fn test_nbsp() {
        assert_eq!(fix_nbsp("v lese"), "v~lese");
        assert_eq!(fix_nbsp("a to bylo"), "a~to bylo");
        assert_eq!(fix_nbsp("ve škole"), "ve~škole");
    }
}
