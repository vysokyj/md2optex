/// Applies Czech typographic conventions to a plain-text node.
pub fn apply(text: &str) -> String {
    let s = fix_ellipsis(text);
    let s = fix_dashes(&s);
    let s = fix_quotes(&s);
    let s = fix_nbsp(&s);
    s
}

/// Replaces `...` with `\dots`.
fn fix_ellipsis(s: &str) -> String {
    s.replace("...", r"\dots")
}

/// Normalises dashes and inserts a non-breaking tilde before them
/// so they cannot be left at the end of a line.
///
/// Both ASCII sequences (` -- `, ` --- `) and Unicode characters
/// (U+2013 en-dash, U+2014 em-dash) are accepted as input.
/// When surrounded by spaces the leading space becomes `~`.
/// Bare Unicode dashes (no surrounding spaces) are replaced without spacing.
fn fix_dashes(s: &str) -> String {
    // Spaced variants first — both ASCII and Unicode, with non-breaking space
    let s = s.replace(" \u{2014} ", "~--- "); // " — " → ~---
    let s = s.replace(" \u{2013} ", "~-- ");  // " – " → ~--
    let s = s.replace(" --- ", "~--- ");
    let s = s.replace(" -- ", "~-- ");
    // Bare Unicode dashes (no surrounding spaces)
    let s = s.replace('\u{2014}', "---");      // — → ---
    let s = s.replace('\u{2013}', "--");       // – → --
    s
}

/// Converts ASCII and Czech Unicode quotes to `\uv{}`.
///
/// ASCII `"`: first occurrence opens (`\uv{`), second closes (`}`).
/// Unicode „ (U+201E) → opening, " (U+201C) → closing.
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
                // „ — Czech opening double quote
                result.push_str(r"\uv{");
                open = true;
            }
            '\u{201C}' => {
                // " — Czech closing double quote
                result.push('}');
                open = false;
            }
            _ => result.push(c),
        }
    }
    // Unclosed quote — leave as-is (should not occur in well-formed input)
    result
}

/// Inserts a non-breaking space (`~`) after single- and two-letter
/// Czech prepositions and conjunctions to prevent them from appearing
/// at the end of a line.
fn fix_nbsp(s: &str) -> String {
    // Prepositions and conjunctions of length 1–2 characters
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

/// Replaces `pat` with `replacement` only at a word boundary
/// (preceded by a space, newline, `(`, or start of string).
fn replace_word_boundary(s: &str, pat: &str, replacement: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut rest = s;

    while let Some(pos) = rest.find(pat) {
        let before = &rest[..pos];
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
        // Unicode en-dash and em-dash with spaces
        assert_eq!(fix_dashes("foo \u{2013} bar"), "foo~-- bar");
        assert_eq!(fix_dashes("foo \u{2014} bar"), "foo~--- bar");
        // Bare Unicode dashes (no surrounding spaces)
        assert_eq!(fix_dashes("foo\u{2013}bar"), "foo--bar");
        assert_eq!(fix_dashes("foo\u{2014}bar"), "foo---bar");
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
