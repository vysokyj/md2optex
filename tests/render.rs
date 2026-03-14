use md2optex::renderer::render_body;

// Convenience: strip leading/trailing whitespace from every line and
// drop blank lines so that tests are not sensitive to exact spacing.
fn normalise(s: &str) -> String {
    s.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn body(md: &str) -> String {
    normalise(&render_body(md, 96))
}

// ── Headings ────────────────────────────────────────────────────────────────

#[test]
fn heading_h1() {
    assert_eq!(body("# Hello"), r"\chap Hello");
}

#[test]
fn heading_h2() {
    assert_eq!(body("## Hello"), r"\sec Hello");
}

#[test]
fn heading_h3() {
    assert_eq!(body("### Hello"), r"\secc Hello");
}

#[test]
fn heading_h4_and_deeper() {
    assert_eq!(body("#### Hello"), r"\seccc Hello");
}

// ── Inline formatting ────────────────────────────────────────────────────────

#[test]
fn bold() {
    assert!(body("**bold**").contains(r"{\bf bold}"));
}

#[test]
fn italic() {
    assert!(body("*italic*").contains(r"{\it italic}"));
}

#[test]
fn inline_code() {
    assert!(body("`code`").contains(r"{\tt code}"));
}

#[test]
fn inline_code_escapes_backslash() {
    assert!(body(r"`a\b`").contains(r"\char92"));
}

// ── Code blocks ─────────────────────────────────────────────────────────────

#[test]
fn fenced_code_block() {
    let out = body("```\nfoo\nbar\n```");
    assert!(out.contains("\\begtt"));
    assert!(out.contains("foo"));
    assert!(out.contains("bar"));
    assert!(out.contains("\\endtt"));
}

#[test]
fn fenced_code_block_with_language() {
    let out = body("```rust\nfn main() {}\n```");
    assert!(out.contains("\\begtt"));
    assert!(out.contains("fn main() {}"));
    assert!(out.contains("\\endtt"));
}

#[test]
fn code_block_content_not_escaped() {
    // Special TeX characters inside \begtt must not be escaped
    let out = body("```\na & b\n```");
    assert!(out.contains("a & b"));
}

// ── Lists ────────────────────────────────────────────────────────────────────

#[test]
fn unordered_list() {
    let out = body("- alpha\n- beta");
    assert!(out.contains("\\begitems"));
    assert!(out.contains("* alpha"));
    assert!(out.contains("* beta"));
    assert!(out.contains("\\enditems"));
}

#[test]
fn ordered_list() {
    let out = body("1. first\n2. second");
    assert!(out.contains("\\begitems \\style n"));
    assert!(out.contains("* first"));
    assert!(out.contains("\\enditems"));
}

// ── Block quote ──────────────────────────────────────────────────────────────

#[test]
fn block_quote() {
    let out = body("> quoted text");
    assert!(out.contains("\\begcitation"));
    assert!(out.contains("quoted text"));
    assert!(out.contains("\\endcitation"));
}

// ── Horizontal rule ──────────────────────────────────────────────────────────

#[test]
fn horizontal_rule() {
    assert!(body("---").contains("\\noindent\\hrule"));
}

// ── Links ────────────────────────────────────────────────────────────────────

#[test]
fn link() {
    let out = body("[click](https://example.com)");
    assert!(out.contains("\\ulink[https://example.com]{click}"));
}

// ── Tables ───────────────────────────────────────────────────────────────────

#[test]
fn table_alignment_spec() {
    let md = "| L | C | R |\n|:--|:-:|--:|\n| a | b | c |";
    let out = body(md);
    assert!(out.contains("\\table{lcr}{"));
}

#[test]
fn table_header_ends_with_crli() {
    let md = "| A | B |\n|---|---|\n| x | y |";
    let out = render_body(md, 96);
    assert!(out.contains("\\crli"));
}

#[test]
fn table_data_row_ends_with_cr() {
    let md = "| A |\n|---|\n| x |";
    let out = render_body(md, 96);
    // data row ends with \cr (not \crli)
    assert!(out.contains("x \\cr"));
}

#[test]
fn table_cells_separated_by_ampersand() {
    let md = "| A | B |\n|---|---|\n| x | y |";
    let out = body(md);
    assert!(out.contains("A & B"));
    assert!(out.contains("x & y"));
}

// ── TeX special character escaping ───────────────────────────────────────────

#[test]
fn escape_ampersand_in_text() {
    // "a" is a Czech conjunction, so it gets a non-breaking space → "a~\& b"
    assert!(body("a & b").contains(r"\&"));
    assert!(!body("foo & bar").contains(" & "));
}

#[test]
fn escape_percent_in_text() {
    assert!(body("100%").contains(r"100\%"));
}

#[test]
fn escape_dollar_in_text() {
    assert!(body("$5").contains(r"\$5"));
}

#[test]
fn escape_hash_in_text() {
    assert!(body("#tag outside heading").contains(r"\#tag"));
}

// ── Czech typographic transformations ────────────────────────────────────────

#[test]
fn typo_ellipsis() {
    assert!(body("wait...").contains(r"wait\dots{}"));
}

#[test]
fn typo_ascii_en_dash() {
    assert!(body("a -- b").contains("a~-- b"));
}

#[test]
fn typo_ascii_em_dash() {
    assert!(body("a --- b").contains("a~--- b"));
}

#[test]
fn typo_unicode_en_dash_spaced() {
    assert!(body("a \u{2013} b").contains("a~-- b"));
}

#[test]
fn typo_unicode_em_dash_spaced() {
    assert!(body("a \u{2014} b").contains("a~--- b"));
}

#[test]
fn typo_unicode_en_dash_bare() {
    // Bare dash without spaces → no tilde
    assert!(body("x\u{2013}y").contains("x--y"));
}

#[test]
fn typo_quotes_ascii() {
    assert!(body(r#""hello""#).contains(r"\uv{hello}"));
}

#[test]
fn typo_quotes_unicode() {
    assert!(body("\u{201E}hello\u{201C}").contains(r"\uv{hello}"));
}

#[test]
fn typo_nbsp_preposition() {
    assert!(body("v lese").contains("v~lese"));
}

#[test]
fn typo_nbsp_conjunction() {
    assert!(body("Jan a Marie").contains("a~Marie"));
}
