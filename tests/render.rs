use md2optex::renderer::{render_body, render_body_captions};

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
    normalise(&render_body(md, 96, None, None))
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
    assert!(body("---").contains("\\ornsep"));
}

// ── Links ────────────────────────────────────────────────────────────────────

#[test]
fn link() {
    let out = body("[click](https://example.com)");
    assert!(out.contains("\\ulink[https://example.com]{click}"));
}

// ── Images ───────────────────────────────────────────────────────────────────

#[test]
fn image_with_base_dir_uses_absolute_path() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let out = md2optex::renderer::render_body("![alt](sample.png)", 96, Some(&dir), None);
    // Path should be absolute (starts with /)
    assert!(
        out.contains("/examples/sample.png"),
        "expected absolute path in: {out}"
    );
    // Image is 1024px wide → > 15 cm at 96 DPI → \hsize
    assert!(
        out.contains("\\centimage{\\hsize}"),
        "expected \\hsize for wide image in: {out}"
    );
}

#[test]
fn image_without_base_dir_keeps_path() {
    let out = md2optex::renderer::render_body("![alt](img/photo.png)", 96, None, None);
    assert!(
        out.contains("\\centimage{\\hsize}{img/photo.png}"),
        "expected original path in: {out}"
    );
}

#[test]
fn image_with_images_dir_resolves_via_images_dir() {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let images_dir = base.join("examples");
    let out =
        md2optex::renderer::render_body("![alt](sample.png)", 96, Some(base), Some(&images_dir));
    assert!(
        out.contains("/examples/sample.png"),
        "expected images_dir path in: {out}"
    );
}

// ── Tables ───────────────────────────────────────────────────────────────────

#[test]
fn table_alignment_spec() {
    let md = "| L | C | R |\n|:--|:-:|--:|\n| a | b | c |";
    let out = body(md);
    // Three wrapping columns: each gets \hsize/3.
    assert!(out.contains("\\table{p{\\dimexpr(\\hsize - 3em)/3\\relax} p{\\dimexpr(\\hsize - 3em)/3\\relax} p{\\dimexpr(\\hsize - 3em)/3\\relax}}{"), "got: {out}");
}

#[test]
fn table_center_alignment_hfil() {
    let md = "| L | C | R |\n|:--|:-:|--:|\n| a | b | c |";
    let out = body(md);
    // Center column: \hfil prefix + suffix; right column: \hfill prefix only.
    assert!(out.contains("\\hfil b \\hfil"), "got: {out}");
    assert!(out.contains("\\hfill c"), "got: {out}");
}

#[test]
fn table_header_ends_with_crli() {
    let md = "| A | B |\n|---|---|\n| x | y |";
    let out = render_body(md, 96, None, None);
    assert!(out.contains("\\crli"));
}

#[test]
fn table_data_row_ends_with_cr() {
    let md = "| A |\n|---|\n| x |";
    let out = render_body(md, 96, None, None);
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

// ── Footnotes ────────────────────────────────────────────────────────────────

#[test]
fn footnote_inline() {
    let md = "Text[^1] here.\n\n[^1]: The footnote text.\n";
    assert!(body(md).contains(r"\fnote{The footnote text.}"));
}

#[test]
fn footnote_reference_replaced() {
    let md = "See[^note] this.\n\n[^note]: Explanation here.\n";
    let out = body(md);
    // Reference is replaced by \fnote, definition block is not emitted
    assert!(out.contains(r"\fnote{Explanation here.}"));
    assert!(!out.contains("note]"));
}

// ── Strikethrough ─────────────────────────────────────────────────────────────

#[test]
fn strikethrough() {
    assert!(body("~~přeškrtnutý~~").contains(r"\strike{přeškrtnutý}"));
}

// ── Task lists ───────────────────────────────────────────────────────────────

#[test]
fn task_list_checked() {
    assert!(body("- [x] hotovo").contains(r"[{\tt x}]\ "));
}

#[test]
fn task_list_unchecked() {
    assert!(body("- [ ] todo").contains(r"[\ ]\ "));
}

// ── Heading nonum / notoc ────────────────────────────────────────────────────

#[test]
fn heading_default_no_nonum() {
    // Without book style, headings have no \nonum prefix
    assert!(!body("# Nadpis").contains("\\nonum"));
}

// ── Table captions (academic/captions mode) ──────────────────────────────────

fn body_captions(md: &str) -> String {
    normalise(&render_body_captions(md, 96, None, None))
}

#[test]
fn table_caption_pandoc_style() {
    let md = "| A | B |\n|---|---|\n| 1 | 2 |\n\n: Výsledky měření\n";
    let out = body_captions(md);
    assert!(
        out.contains(r"\caption/t Výsledky měření"),
        "expected \\caption/t, got: {out}"
    );
    assert!(
        !out.contains(": Výsledky"),
        "colon prefix should be stripped, got: {out}"
    );
}

#[test]
fn table_caption_colon_only() {
    let md = "| A | B |\n|---|---|\n| 1 | 2 |\n\n:Přehled bez mezery\n";
    let out = body_captions(md);
    assert!(out.contains(r"\caption/t Přehled bez mezery"), "got: {out}");
}

#[test]
fn table_caption_no_prefix_emits_paragraph() {
    let md = "| A | B |\n|---|---|\n| 1 | 2 |\n\nNormální text.\n";
    let out = body_captions(md);
    assert!(
        !out.contains(r"\caption/t"),
        "should not emit caption, got: {out}"
    );
    assert!(
        out.contains("Normální text."),
        "paragraph should be present, got: {out}"
    );
}

#[test]
fn table_caption_not_emitted_without_captions_mode() {
    let md = "| A | B |\n|---|---|\n| 1 | 2 |\n\n: Výsledky\n";
    let out = body(md);
    assert!(
        !out.contains(r"\caption/t"),
        "caption should not appear without captions mode, got: {out}"
    );
}

// ── Math ─────────────────────────────────────────────────────────────────────

#[test]
fn inline_math() {
    assert!(body("Platí $E = mc^2$ v praxi.").contains("$E = mc^2$"));
}

#[test]
fn display_math() {
    let out = body("$$\\int_0^\\infty e^{-x}\\,dx = 1$$");
    assert!(out.contains("$$\\int_0^\\infty e^{-x}\\,dx = 1$$"));
}

// ── Raw TeX passthrough ───────────────────────────────────────────────────────

#[test]
fn raw_tex_block_passthrough() {
    let md = "```tex\n\\vfil\\eject\n```\n";
    let out = body(md);
    assert!(out.contains("\\vfil\\eject"), "got: {out}");
    assert!(
        !out.contains("\\begtt"),
        "should not wrap in begtt, got: {out}"
    );
}

#[test]
fn raw_optex_block_passthrough() {
    let md = "```optex\n\\nonum\\chap Test\n```\n";
    let out = body(md);
    assert!(out.contains("\\nonum\\chap Test"), "got: {out}");
}

// ── Superscript / subscript ───────────────────────────────────────────────────

#[test]
fn superscript() {
    // pulldown-cmark requires ^..^ to be space-delimited when adjacent to text
    let out = body("x ^2^ test");
    assert!(out.contains("\\tsuper{2}"), "got: {out}");
}

#[test]
fn subscript() {
    let out = body("~2~");
    assert!(out.contains("\\tsub{2}"), "got: {out}");
}

// ── Definition lists ──────────────────────────────────────────────────────────

#[test]
fn definition_list_title_bold() {
    let md = "Pojem\n: Definice pojmu\n";
    let out = body(md);
    assert!(out.contains("{\\bf Pojem}"), "got: {out}");
}

#[test]
fn definition_list_indented_definition() {
    let md = "Pojem\n: Definice pojmu\n";
    let out = body(md);
    assert!(out.contains("\\advance\\leftskip by 2em"), "got: {out}");
    assert!(out.contains("\\advance\\leftskip by -2em"), "got: {out}");
}

// ── Pandoc-compatible attributes ────────────────────────────────────────────

#[test]
fn heading_unnumbered_attr() {
    let out = body("## Title {.unnumbered}\n");
    assert!(out.contains("\\nonum \\sec Title"), "got: {out}");
}

#[test]
fn heading_unnumbered_shorthand() {
    let out = body("## Title {.-}\n");
    assert!(out.contains("\\nonum \\sec Title"), "got: {out}");
}

#[test]
fn heading_unlisted_attr() {
    let out = body("## Title {.unlisted}\n");
    assert!(out.contains("\\notoc \\sec Title"), "got: {out}");
}

#[test]
fn heading_id_attr() {
    let out = body("## Title {#my-id}\n");
    assert!(out.contains("\\label[my-id]"), "got: {out}");
}

#[test]
fn code_block_number_lines() {
    let md = "```python {.numberLines}\ndef f():\n    pass\n```\n";
    let out = body(md);
    assert!(out.contains("\\ttline=1 \\begtt"), "got: {out}");
    assert!(out.contains("\\ttline=-1"), "reset after endtt, got: {out}");
}

#[test]
fn code_block_number_lines_start_from() {
    let md = "```rust {.numberLines startFrom=\"10\"}\nlet x = 1;\n```\n";
    let out = body(md);
    assert!(out.contains("\\ttline=10 \\begtt"), "got: {out}");
}

#[test]
fn code_block_without_attrs_no_ttline() {
    let md = "```rust\nlet x = 1;\n```\n";
    let out = body(md);
    assert!(!out.contains("\\ttline"), "got: {out}");
}

#[test]
fn table_longtable_uses_halign() {
    let md = "| A | B |\n|---|---|\n| 1 | 2 |\n\n{.longtable}\n";
    let out = body(md);
    assert!(out.contains("\\halign"), "should use \\halign, got: {out}");
    assert!(
        !out.contains("\\table"),
        "should not use \\table, got: {out}"
    );
}

#[test]
fn table_col_widths_derived_from_separator() {
    // Separator |-----|--------------------|---------| has cells of length 5, 20, 9 → proportional
    let md = "| A | B | C |\n|-----|--------------------|---------|\n| 1 | 2 | 3 |\n";
    let out = body(md);
    // Should NOT use equal-width formula (which has /N pattern)
    assert!(
        !out.contains("/3\\relax"),
        "should use derived widths, not equal formula, got: {out}"
    );
    // Should contain proportional \hsize values with tabskip subtraction
    assert!(
        out.contains("\\hsize-"),
        "should use proportional \\hsize widths, got: {out}"
    );
}

#[test]
fn table_col_widths_equal_separator_uses_default() {
    // All separator cells equal length → use default equal distribution
    let md = "| A | B | C |\n|---|---|---|\n| 1 | 2 | 3 |\n";
    let out = body(md);
    assert!(
        out.contains("\\dimexpr"),
        "equal separators should use default formula, got: {out}"
    );
}


#[test]
fn table_without_longtable_uses_table() {
    let md = "| A | B |\n|---|---|\n| 1 | 2 |\n";
    let out = body(md);
    assert!(out.contains("\\table"), "should use \\table, got: {out}");
    assert!(
        !out.contains("\\halign"),
        "should not use \\halign, got: {out}"
    );
}

#[test]
fn span_smallcaps() {
    let out = body("[text]{.smallcaps}\n");
    assert!(out.contains("{\\caps text}"), "got: {out}");
}

#[test]
fn span_underline() {
    let out = body("[word]{.underline}\n");
    assert!(out.contains("\\underbar{word}"), "got: {out}");
}

#[test]
fn span_mark() {
    let out = body("[highlighted]{.mark}\n");
    assert!(out.contains("\\highlight{highlighted}"), "got: {out}");
}

#[test]
fn span_in_sentence() {
    let out = body("Before [caps]{.smallcaps} after.\n");
    assert!(out.contains("Before {\\caps caps} after."), "got: {out}");
}
