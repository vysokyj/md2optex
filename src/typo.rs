/// Applies Czech typographic conventions to a plain-text node.
pub fn apply(text: &str) -> String {
    let s = fix_ellipsis(text);
    let s = fix_dashes(&s);
    let s = fix_quotes(&s);
    fix_nbsp(&s)
}

/// Replaces `...` with `\dots{}`.
///
/// The empty group `{}` terminates the control-word name so that a letter
/// immediately following (e.g. `...třikrát`) is not swallowed into it.
fn fix_ellipsis(s: &str) -> String {
    s.replace("...", r"\dots{}")
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
    s.replace('\u{2013}', "--")               // – → --
}

/// Converts various quotation marks to `\uv{…}`, with full nesting support.
///
/// Recognised opening → closing pairs:
/// - `"…"` ASCII double (context-sensitive: opens after whitespace/start, closes after word)
/// - `„…"` / U+201E…U+201C  Czech double (unambiguous)
/// - `"…"` / U+201C…U+201D  English curly double (unambiguous)
/// - `‚…'` / U+201A…U+2019  Czech single / inner (unambiguous)
/// - `'…'` / U+2018…U+2019  English curly single / inner (unambiguous)
///
/// Unmatched opening quotes produce a warning on stderr.
fn fix_quotes(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut pos = 0;
    parse_quote_level(&chars, &mut pos, None)
}

/// Recursive descent over `chars` starting at `*pos`.
/// Parses until the expected closing delimiter is found (or end of input).
/// Returns the rendered content *without* the closing delimiter.
fn parse_quote_level(chars: &[char], pos: &mut usize, expected_close: Option<char>) -> String {
    let mut out = String::new();

    while *pos < chars.len() {
        let c = chars[*pos];

        // 1. Does this char close the current quote level?
        if let Some(close) = expected_close
            && is_quote_close(chars, *pos, close)
        {
            *pos += 1;
            return out;
        }

        // 2. Does this char open a new (possibly nested) quote?
        if let Some(close_ch) = quote_open_close(chars, *pos) {
            *pos += 1;
            let inner = parse_quote_level(chars, pos, Some(close_ch));
            out.push_str("\\uv{");
            out.push_str(&inner);
            out.push('}');
        } else {
            // 3. Ordinary character.
            out.push(c);
            *pos += 1;
        }
    }

    if expected_close.is_some() {
        eprintln!("md2optex: warning: unclosed quotation mark");
    }
    out
}

/// If `chars[pos]` is a recognised opening quotation mark, returns the
/// expected closing character; otherwise returns `None`.
fn quote_open_close(chars: &[char], pos: usize) -> Option<char> {
    match chars[pos] {
        '„' => Some('\u{201C}'), // „ (U+201E) → " (Czech double)
        '\u{201C}' => Some('\u{201D}'),        // " → " (English curly double)
        '\u{201A}' => Some('\u{2019}'),        // ‚ → ' (Czech single / inner)
        '\u{2018}' => Some('\u{2019}'),        // ' → ' (English curly single)
        // ASCII double: opens only when preceded by whitespace or at start
        '"' if is_before_content(chars, pos) => Some('"'),
        _ => None,
    }
}

/// Returns `true` if `chars[pos]` is the expected closing quote.
///
/// For a Czech „ opener (expected = U+201C) all three common closing variants
/// are accepted: U+201C (traditional Czech), U+201D (modern editor pairing),
/// and ASCII `"` in a non-opening position (user typed „…").
/// For ASCII `"` openers the close is also ASCII `"` in a non-opening position.
/// All other Unicode closing marks are unambiguous and always close.
fn is_quote_close(chars: &[char], pos: usize, expected: char) -> bool {
    let c = chars[pos];
    match expected {
        // Czech „ opener: accept " U+201C, " U+201D, or ASCII " after content
        '\u{201C}' => {
            c == '\u{201C}'
                || c == '\u{201D}'
                || (c == '"' && !is_before_content(chars, pos))
        }
        // ASCII opener: close with ASCII " after content
        '"' => c == '"' && !is_before_content(chars, pos),
        // All other Unicode closers: exact match
        _ => c == expected,
    }
}

/// True when `chars[pos]` is at the start of content: position 0, or
/// immediately after whitespace or an opening bracket.
fn is_before_content(chars: &[char], pos: usize) -> bool {
    pos == 0 || matches!(chars[pos - 1], ' ' | '\t' | '\n' | '(' | '[' | '{')
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
                .is_some_and(|c| c == ' ' || c == '\n' || c == '(');

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
        assert_eq!(fix_ellipsis("a...b"), r"a\dots{}b");
        assert_eq!(fix_ellipsis("konec..."), r"konec\dots{}");
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
    fn test_quotes_ascii() {
        assert_eq!(fix_quotes(r#"řekl "ahoj" a šel"#), r"řekl \uv{ahoj} a šel");
        // Opening at start of string
        assert_eq!(fix_quotes(r#""hello""#), r"\uv{hello}");
    }

    #[test]
    fn test_quotes_czech_unicode() {
        // „ (U+201E) … " (U+201C) — traditional Czech
        assert_eq!(fix_quotes("„ahoj\u{201C}"), r"\uv{ahoj}");
        // „ (U+201E) … " (ASCII) — user types „ then closes with " key
        assert_eq!(fix_quotes(r#"„Poslouchej svůj hlas.""#), r"\uv{Poslouchej svůj hlas.}");
        // „ (U+201E) … " (U+201D) — modern editor auto-pairing
        assert_eq!(fix_quotes("„ahoj\u{201D}"), r"\uv{ahoj}");
        // with em-dash inside
        assert_eq!(
            fix_quotes(r#"„Poslouchej svůj hlas — vypadá to na odpor.""#),
            r"\uv{Poslouchej svůj hlas — vypadá to na odpor.}"
        );
    }

    #[test]
    fn test_quotes_english_curly() {
        // " (U+201C) … " (U+201D)
        assert_eq!(fix_quotes("\u{201C}hello\u{201D}"), r"\uv{hello}");
    }

    #[test]
    fn test_quotes_nested() {
        // Czech double with Czech single nested: „outer ‚inner' text"
        let input = "„outer \u{201A}inner\u{2019} text\u{201C}";
        assert_eq!(fix_quotes(input), r"\uv{outer \uv{inner} text}");
    }

    #[test]
    fn test_quotes_ascii_nested() {
        // ASCII double with ASCII double nested (context-driven)
        assert_eq!(fix_quotes(r#""outer "inner" text""#), r"\uv{outer \uv{inner} text}");
    }

    #[test]
    fn test_quotes_unmatched() {
        // Unmatched — should not panic; content is still wrapped
        let out = fix_quotes("\"unclosed");
        assert!(out.contains("\\uv{"));
        assert!(out.contains("unclosed"));
    }

    #[test]
    fn test_nbsp() {
        assert_eq!(fix_nbsp("v lese"), "v~lese");
        assert_eq!(fix_nbsp("a to bylo"), "a~to bylo");
        assert_eq!(fix_nbsp("ve škole"), "ve~škole");
    }
}
