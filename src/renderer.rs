use std::collections::HashMap;
use std::path::{Path, PathBuf};

use pulldown_cmark::{Alignment, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::error::Error;
use crate::metadata::{
    Metadata, Page, TocValue, normalize_paper, parse_length_mm, parse_margin_shorthand,
};
use crate::styles;
use crate::typo;

/// Per-table attributes extracted from Pandoc-compatible attribute blocks.
#[derive(Debug, Clone, Default)]
struct TableAttrs {
    longtable: bool,
    /// Column widths as fractions of `\hsize` (e.g. 0.30 for 30%).
    col_widths: Option<Vec<f64>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum TocPlacement {
    Front,
    Back,
}

/// Resolves TOC placement from metadata + style default.
/// Style "book" defaults to Back; all others default to Front.
/// Values: `false`/`"off"` = none, `true`/`"front"` = style default, `"back"`.
fn resolve_toc(toc: Option<&TocValue>, style_name: Option<&str>) -> Option<TocPlacement> {
    let style_default = match style_name {
        Some("book") => TocPlacement::Back,
        _ => TocPlacement::Front,
    };
    match toc {
        None | Some(TocValue::Bool(false)) => None,
        Some(TocValue::Bool(true)) => Some(style_default),
        Some(TocValue::Position(s)) => match s.to_ascii_lowercase().as_str() {
            "off" | "none" | "false" => None,
            "back" => Some(TocPlacement::Back),
            _ => Some(TocPlacement::Front),
        },
    }
}

/// Renders the TOC block (heading + \maketoc) with suppressed header/footer.
fn toc_block(placement: TocPlacement) -> String {
    let mut s = String::new();
    if placement == TocPlacement::Front {
        // Ensure recto page for front TOC.
        s.push_str("\\ifodd\\pageno\\else\\null\\vfil\\eject\\fi\n");
    } else {
        // Back TOC: just start a new page.
        s.push_str("\\vfil\\supereject\n");
    }
    s.push_str("\\bgroup\\footline={}\\headline={}\n");
    s.push_str("\\centerline{\\typosize[14/17]\\bf Obsah}\\bigskip\n");
    s.push_str("\\maketoc\n");
    s.push_str("\\egroup\n");
    s
}

pub fn render(
    markdown: &str,
    metadata: Option<&Metadata>,
    hyphenation: &[String],
    dpi: u32,
    style: Option<&str>,
    base_dir: Option<&Path>,
    output_dir: Option<&Path>,
) -> Result<String, Error> {
    // Extract YAML front matter when no external metadata is provided (single-file mode).
    // yaml_meta_owned must outlive effective_metadata (which may borrow it).
    #[allow(unused_assignments)]
    let mut yaml_meta_owned: Option<Metadata> = None;
    let (metadata, markdown) = if metadata.is_none() {
        let (ym, rest) = extract_yaml_front_matter(markdown);
        yaml_meta_owned = ym;
        (yaml_meta_owned.as_ref(), rest)
    } else {
        (metadata, markdown)
    };

    let style_name = style.or_else(|| metadata.and_then(|m| m.style.as_deref()));
    let toc = metadata
        .and_then(|m| m.options.as_ref())
        .and_then(|o| o.toc.as_ref());
    let toc_placement = resolve_toc(toc, style_name);

    // nonum: suppress heading numbers (book style convention)
    let nonum = style_name == Some("book");
    // toc_depth: max heading level included in TOC (book default = 1, others = no limit)
    let toc_depth = metadata
        .and_then(|m| m.options.as_ref())
        .and_then(|o| o.toc_depth)
        .unwrap_or(if nonum { 1 } else { u32::MAX });

    let is_book = style_name == Some("book");

    // Drop cap: enabled by default for book style, disabled for others;
    // metadata.toml `options.drop-cap` always wins.
    let drop_cap_enabled = metadata
        .and_then(|m| m.options.as_ref())
        .and_then(|o| o.drop_cap)
        .unwrap_or(is_book);

    let mut out = String::new();
    out.push_str(&build_preamble(
        metadata,
        hyphenation,
        style,
        toc_placement,
        is_book,
        base_dir,
    )?);
    let images_dir = metadata
        .and_then(|m| m.paths.as_ref())
        .and_then(|p| p.images.as_deref())
        .and_then(|rel| base_dir.map(|b| b.join(rel)));
    let captions = style_name == Some("academic");
    out.push_str(&render_body_impl(
        markdown,
        dpi,
        base_dir,
        images_dir.as_deref(),
        output_dir,
        nonum,
        toc_depth,
        captions,
        drop_cap_enabled,
    ));
    if toc_placement == Some(TocPlacement::Back) {
        out.push_str(&toc_block(TocPlacement::Back));
    }
    // Back colophon is emitted only when no half-title was shown — otherwise
    // the colophon already sits on the verso of the title page.
    if is_book
        && let Some(meta) = metadata
        && !meta
            .options
            .as_ref()
            .and_then(|o| o.half_title)
            .unwrap_or(true)
    {
        out.push_str(&back_colophon_block(meta));
    }
    out.push_str("\n\\bye\n");
    Ok(out)
}

/// Renders only the document body (no preamble, no `\bye`).
/// Used by integration tests; uses neutral defaults (no nonum, unlimited TOC depth).
/// Passes `output_dir = None` → Passthrough mode (image paths unchanged).
/// Drop cap is disabled — tests exercising it should use `render_body_book`.
#[allow(dead_code)]
pub fn render_body(
    markdown: &str,
    dpi: u32,
    base_dir: Option<&Path>,
    images_dir: Option<&Path>,
) -> String {
    render_body_impl(
        markdown,
        dpi,
        base_dir,
        images_dir,
        None,
        false,
        u32::MAX,
        false,
        false,
    )
}

/// Like `render_body` but with an explicit output_dir (enables path rewrite).
#[allow(dead_code)]
pub fn render_body_with_output(
    markdown: &str,
    dpi: u32,
    base_dir: Option<&Path>,
    images_dir: Option<&Path>,
    output_dir: Option<&Path>,
) -> String {
    render_body_impl(
        markdown,
        dpi,
        base_dir,
        images_dir,
        output_dir,
        false,
        u32::MAX,
        false,
        false,
    )
}

/// Like `render_body` but with captions enabled (academic style convention).
#[allow(dead_code)]
pub fn render_body_captions(
    markdown: &str,
    dpi: u32,
    base_dir: Option<&Path>,
    images_dir: Option<&Path>,
) -> String {
    render_body_impl(
        markdown,
        dpi,
        base_dir,
        images_dir,
        None,
        false,
        u32::MAX,
        true,
        false,
    )
}

/// Like `render_body` with drop cap enabled (book-style behaviour).
#[allow(dead_code)]
pub fn render_body_book(markdown: &str, dpi: u32) -> String {
    render_body_impl(
        markdown,
        dpi,
        None,
        None,
        None,
        false,
        u32::MAX,
        false,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
fn render_body_impl(
    markdown: &str,
    dpi: u32,
    base_dir: Option<&Path>,
    images_dir: Option<&Path>,
    output_dir: Option<&Path>,
    nonum: bool,
    toc_depth: u32,
    captions: bool,
    drop_cap_enabled: bool,
) -> String {
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_MATH
        | Options::ENABLE_SUPERSCRIPT
        | Options::ENABLE_SUBSCRIPT
        | Options::ENABLE_DEFINITION_LIST
        | Options::ENABLE_HEADING_ATTRIBUTES;

    let preprocessed = preprocess_image_attrs(markdown);
    let preprocessed = preprocess_span_attrs(&preprocessed);
    let (preprocessed, table_attrs) = preprocess_table_attrs(&preprocessed);
    let markdown = preprocessed.as_str();
    let footnotes = collect_footnotes(markdown, opts);
    let parser = Parser::new_ext(markdown, opts);
    let mut ctx = Context::new(
        dpi,
        base_dir,
        images_dir,
        output_dir,
        nonum,
        toc_depth,
        captions,
        drop_cap_enabled,
        footnotes,
        table_attrs,
    );
    let mut out = String::new();

    for event in parser {
        ctx.handle_event(event, &mut out);
    }
    // Merge consecutive blockquotes (separated only by blank lines) into one
    // continuous block so the left rule spans the full dialogue/citation.
    let out = out.replace("\\endcitation\n\n\\begcitation\n", "\n");
    out.replace("\\enddialogue\n\n\\begdialogue\n", "\n")
}

/// Pre-scans markdown and returns a map of footnote label → rendered TeX body.
fn collect_footnotes(markdown: &str, opts: Options) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut current_label: Option<String> = None;
    let mut current_body = String::new();
    let mut depth = 0usize;

    for event in Parser::new_ext(markdown, opts) {
        match event {
            Event::Start(Tag::FootnoteDefinition(label)) => {
                current_label = Some(label.to_string());
                current_body.clear();
                depth = 0;
            }
            Event::End(TagEnd::FootnoteDefinition) => {
                if let Some(label) = current_label.take() {
                    // Strip trailing whitespace/newlines added by paragraph end
                    map.insert(label, current_body.trim_end().to_owned());
                }
            }
            // Collect the text content inside the footnote definition.
            // We ignore nested block structure (paragraphs, lists) and render
            // inline content only — sufficient for typical footnote usage.
            _ if current_label.is_some() => match &event {
                Event::Start(_) => depth += 1,
                Event::End(_) => {
                    depth = depth.saturating_sub(1);
                }
                Event::Text(t) => {
                    let escaped = tex_escape(t);
                    let processed = typo::apply(&escaped);
                    current_body.push_str(&processed);
                }
                Event::Code(t) => current_body.push_str(&format!("{{\\tt {}}}", tex_escape(t))),
                Event::SoftBreak => current_body.push(' '),
                Event::HardBreak => current_body.push(' '),
                _ => {}
            },
            _ => {}
        }
    }
    map
}

/// Converts a header/footer template string (`left & center & right`) into
/// an OpTeX `\headline`/`\footline` body.
/// Recognises `{author}`, `{title}`, `{folio}` placeholders.
fn running_line(template: &str) -> String {
    let parts: Vec<&str> = template.splitn(3, '&').collect();
    let left = parts.first().map(|s| s.trim()).unwrap_or("");
    let center = parts.get(1).map(|s| s.trim()).unwrap_or("");
    let right = parts.get(2).map(|s| s.trim()).unwrap_or("");
    let subst = |s: &str| {
        s.replace("{author}", "\\theauthor")
            .replace("{title}", "\\thetitle")
            .replace("{folio}", "\\folio")
    };
    format!(
        "{}\\hfil {}\\hfil {}",
        subst(left),
        subst(center),
        subst(right)
    )
}

/// Emits the half-title page (polotitul): centred title only, blank verso.
/// Headline/footline suppressed via `\bgroup ... \egroup` so running headers
/// stay clean throughout the front matter.
fn half_title_block() -> String {
    let mut s = String::new();
    s.push_str("\\bgroup\\footline={}\\headline={}\n");
    // Recto: title centred ~1/3 down the page.
    s.push_str("\\null\\vskip0.3\\vsize\n");
    s.push_str("\\centerline{{\\typosize[16/20]\\bf\\thetitle}}\n");
    s.push_str("\\vfil\\eject\n");
    // Verso: blank.
    s.push_str("\\null\\vfil\\eject\n");
    s.push_str("\\egroup\n");
    s
}

/// Returns `(inner, outer, top, bottom)` margins in millimetres for the given
/// paper size, following Tschichold's canon: asymmetric, outer and bottom
/// margins larger than inner and top. Supports the paper sizes understood by
/// OpTeX `\margins`. Unknown papers fall back to generic proportions.
fn tschichold_margins(paper: &str) -> (u32, u32, u32, u32) {
    match paper.to_ascii_lowercase().as_str() {
        // (inner, outer, top, bottom) — proportions 2:3:4:6, derived from canon.
        "b5" => (18, 36, 22, 44),     // 176×250 mm
        "a5" => (15, 30, 18, 36),     // 148×210 mm
        "a4" => (25, 50, 30, 60),     // 210×297 mm
        "letter" => (22, 44, 26, 52), // 216×279 mm
        _ => (20, 40, 25, 50),
    }
}

/// Produces the OpTeX `\margins` line from a `[page]` section, or `None` if
/// the section has nothing worth emitting. Honours `canon = "tschichold"` for
/// asymmetric two-sided margins; otherwise parses CSS-style margin shorthand
/// and per-side overrides.
fn page_margins_line(page: &Page) -> Option<String> {
    let has_size = page.size.is_some();
    let has_margin = page.margin.is_some()
        || page.margin_top.is_some()
        || page.margin_right.is_some()
        || page.margin_bottom.is_some()
        || page.margin_left.is_some();
    let use_canon = page.canon.as_deref() == Some("tschichold");
    if !has_size && !has_margin && !use_canon {
        return None;
    }

    let paper = page
        .size
        .as_deref()
        .map(normalize_paper)
        .unwrap_or_else(|| "a4".to_string());

    if use_canon {
        let (inner, outer, top, bottom) = tschichold_margins(&paper);
        return Some(format!(
            "\\margins/2 {paper} ({inner},{outer},{top},{bottom})mm\n"
        ));
    }

    // CSS shorthand starts the baseline; per-side overrides replace individual values.
    let (mut top, mut right, mut bottom, mut left) = page
        .margin
        .as_deref()
        .and_then(parse_margin_shorthand)
        .unwrap_or((30.0, 25.0, 30.0, 25.0));
    if let Some(s) = &page.margin_top
        && let Some(v) = parse_length_mm(s)
    {
        top = v;
    }
    if let Some(s) = &page.margin_right
        && let Some(v) = parse_length_mm(s)
    {
        right = v;
    }
    if let Some(s) = &page.margin_bottom
        && let Some(v) = parse_length_mm(s)
    {
        bottom = v;
    }
    if let Some(s) = &page.margin_left
        && let Some(v) = parse_length_mm(s)
    {
        left = v;
    }
    let (l, r, t, b) = (
        left.round() as u32,
        right.round() as u32,
        top.round() as u32,
        bottom.round() as u32,
    );
    Some(format!("\\margins/1 {paper} ({l},{r},{t},{b})mm\n"))
}

/// Generates the back colophon (tiráž) for book style — placed at the very end
/// of the document, before `\bye`. The content is pushed to the bottom of the page.
/// Only emitted when at least one of copyright/year/isbn is present in metadata.
fn back_colophon_block(meta: &Metadata) -> String {
    let has_content = meta.copyright.is_some() || meta.year.is_some() || meta.isbn.is_some();
    if !has_content {
        return String::new();
    }

    let mut s = String::new();
    s.push_str("\n\\vfil\\supereject\n");
    s.push_str("\\bgroup\\footline={}\\headline={}\n");
    s.push_str("\\null\\vfil\n");

    if meta.title.is_some() || meta.author.is_some() {
        if meta.title.is_some() {
            s.push_str("\\noindent {\\bf\\thetitle}\\par\n");
        }
        if meta.author.is_some() {
            s.push_str("\\noindent {\\it\\theauthor}\\par\n");
        }
        s.push_str("\\smallskip\n");
    }

    if let Some(cr) = &meta.copyright {
        s.push_str(&format!("\\noindent {cr}\\par\n"));
    } else if let (Some(year), Some(author)) = (&meta.year, &meta.author) {
        s.push_str(&format!("\\noindent \\char169 \\ {year} {author}\\par\n"));
    } else if let Some(year) = &meta.year {
        s.push_str(&format!("\\noindent \\char169 \\ {year}\\par\n"));
    }

    if let Some(isbn) = &meta.isbn {
        s.push_str(&format!("\\noindent ISBN: {isbn}\\par\n"));
    }

    s.push_str("\\eject\\egroup\n");
    s
}

fn build_preamble(
    metadata: Option<&Metadata>,
    hyphenation: &[String],
    style: Option<&str>,
    toc_placement: Option<TocPlacement>,
    is_book: bool,
    base_dir: Option<&Path>,
) -> Result<String, Error> {
    let mut s = String::new();

    // OpTeX is a LuaTeX format — no \input optex needed, it is pre-loaded by the engine.
    s.push_str("\\fontfam[LM]\n"); // Latin Modern Unicode — required for Czech characters
    s.push_str("\\uselanguage{czech}\n");
    // \begcitation/\endcitation: regular blockquotes (non-dialogue).
    // \begdialogue/\enddialogue: speaker-label dialogues (uses wider left indent for \citelabel llap).
    s.push_str("\\def\\begcitation{\\par\\medskip\\leftskip=2em\\rightskip=2em\\noindent}\n");
    s.push_str("\\def\\endcitation{\\par\\leftskip=0em\\rightskip=0em\\medskip\\_firstnoindent}\n");
    s.push_str("\\def\\begdialogue{\\par\\medskip\\leftskip=5em\\rightskip=2em\\noindent}\n");
    s.push_str("\\def\\enddialogue{\\par\\leftskip=0em\\rightskip=0em\\medskip\\_firstnoindent}\n");
    // \maketitle is not built into OpTeX; define it here.
    // vertical fill, title (via \tit), author in italics, vertical fill, page break.
    s.push_str("\\def\\maketitle{\\bgroup\\footline={}\\headline={}\\vglue0pt plus1fill\\centerline{{\\typosize[18/22]\\bf\\thetitle}}\\medskip\\centerline{{\\it\\theauthor}}\\vglue0pt plus2fill\\eject\\egroup}\n");
    // \strike: strikethrough using \hbox with \localcolor to avoid space-eating issues.
    s.push_str("\\def\\strike#1{\\leavevmode\\hbox{\\setbox0=\\hbox{#1}\\localcolor\\Black\\rlap{\\vrule height0.55em depth-0.45em width\\wd0}\\box0}}\n");
    // \highlight: yellow background highlight using OpTeX \localcolor for correct spacing.
    s.push_str("\\def\\highlight#1{\\leavevmode\\hbox{\\setbox0=\\hbox{\\kern1pt#1\\kern1pt}\\dimen0=\\ht0\\advance\\dimen0 by1pt\\dimen1=\\dp0\\advance\\dimen1 by1pt\\localcolor\\Yellow\\vrule height\\dimen0 depth\\dimen1 width\\wd0\\kern-\\wd0\\localcolor\\Black\\box0}}\n");
    // \tsuper / \tsub: text-mode superscript / subscript via math mode with roman font.
    s.push_str("\\def\\tsuper#1{$^{\\rm #1}$}\n");
    s.push_str("\\def\\tsub#1{$_{\\rm #1}$}\n");
    // Common LaTeX math macros missing from plain TeX / OpTeX.
    s.push_str("\\def\\frac#1#2{{#1\\over#2}}\n");
    s.push_str("\\def\\dfrac#1#2{{\\displaystyle{#1\\over#2}}}\n");
    s.push_str("\\def\\tfrac#1#2{{\\textstyle{#1\\over#2}}}\n");

    // \ornsep: ornamental separator for horizontal rules.
    s.push_str("\\def\\ornsep{\\bigskip\\centerline{*\\enspace*\\enspace*}\\bigskip}\n");
    // \centimage{width}{path}: centred image.
    s.push_str("\\def\\centimage#1#2{\\medskip\\centerline{\\pdfximage width#1{#2}\\pdfrefximage\\pdflastximage}\\medskip}\n");
    // \chapterimage{width}{path}: full-width image for chapter openers.
    s.push_str("\\def\\chapterimage#1#2{\\centerline{\\pdfximage width#1{#2}\\pdfrefximage\\pdflastximage}\\medskip}\n");
    // \IC{X}{rest}: drop cap (initial capital) — first letter enlarged, rest of text follows.
    s.push_str("\\def\\IC#1#2{{\\font\\ICfont=\\fontname\\tenrm\\space at 2.5em\\relax\\leavevmode\\hbox{\\ICfont #1}\\kern-.05em}#2}\n");

    // Resolve and inject style: CLI --style takes priority over metadata top-level `style`.
    let style_name = style.or_else(|| metadata.and_then(|m| m.style.as_deref()));
    if let Some(name) = style_name {
        match styles::resolve(name, base_dir) {
            Some(content) => s.push_str(&content),
            None => eprintln!("md2optex: warning: style '{name}' not found, using defaults"),
        }
    }

    // Metadata overrides: applied after the style so they take precedence.
    if let Some(meta) = metadata {
        if let Some(opts) = &meta.options {
            if let Some(font) = &opts.font {
                s.push_str(&format!("\\fontfam[{}]\n", font));
            }
            if let Some(size) = &opts.base_size {
                // e.g. "11pt" → \typosize[11/13]
                let pt: u32 = size.trim_end_matches("pt").parse().unwrap_or(10);
                let leading = pt * 13 / 10;
                s.push_str(&format!("\\typosize[{pt}/{leading}]\n"));
            }
        }

        // Page setup: emit \margins when any size/margin/canon is specified.
        if let Some(page) = &meta.page
            && let Some(line) = page_margins_line(page)
        {
            s.push_str(&line);
        }

        if let Some(opts) = &meta.options {
            if let Some(header) = &opts.header {
                s.push_str(&format!("\\headline={{{}}}\n", running_line(header)));
            }
            if let Some(footer) = &opts.footer {
                s.push_str(&format!("\\footline={{{}}}\n", running_line(footer)));
            }
            if opts.paragraph.as_deref() == Some("noindent") {
                s.push_str("\\parindent=0pt\n");
            }
        }
    }

    if !hyphenation.is_empty() {
        s.push_str("\\hyphenation{\n");
        for w in hyphenation {
            s.push_str(&format!("  {w}\n"));
        }
        s.push_str("}\n");
    }

    s.push('\n');

    if let Some(meta) = metadata {
        if let Some(title) = &meta.title {
            s.push_str(&format!("\\gdef\\thetitle{{{title}}}\n"));
        }
        if let Some(author) = &meta.author {
            s.push_str(&format!("\\gdef\\theauthor{{{author}}}\n"));
        }
        // Half-title: enabled for book style by default; opt-in for others.
        // Requires at least a title to show.
        let half_title_opt = meta.options.as_ref().and_then(|o| o.half_title);
        let half_title_enabled = half_title_opt.unwrap_or(is_book) && meta.title.is_some();
        // When half-title is shown, the colophon (copyright / ISBN / rok) moves
        // to the verso of the full title page instead of the back of the book.
        let has_colophon = meta.copyright.is_some() || meta.year.is_some() || meta.isbn.is_some();
        let colophon_on_title_verso = half_title_enabled && has_colophon;

        if half_title_enabled {
            s.push_str(&half_title_block());
        }

        if meta.title.is_some() || meta.author.is_some() {
            s.push_str("\\maketitle\n");
            // Verso of title page.
            let emit_colophon_here = colophon_on_title_verso || (!is_book && has_colophon);
            if emit_colophon_here {
                s.push_str("\\bgroup\\footline={}\\headline={}\n");
                s.push_str("\\null\\vfil\n");
                if let Some(cr) = &meta.copyright {
                    s.push_str(&format!("\\noindent {cr}\\par\n"));
                } else if let (Some(year), Some(author)) = (&meta.year, &meta.author) {
                    s.push_str(&format!("\\noindent \\char169 \\ {year} {author}\\par\n"));
                }
                if let Some(isbn) = &meta.isbn {
                    s.push_str(&format!("\\noindent ISBN: {isbn}\\par\n"));
                }
                s.push_str("\\vfil\\eject\n");
                s.push_str("\\egroup\n");
            } else {
                s.push_str("\\bgroup\\footline={}\\headline={}\\null\\vfil\\eject\\egroup\n");
            }
        }
        if toc_placement == Some(TocPlacement::Front) {
            s.push_str(&toc_block(TocPlacement::Front));
        }
        // Reset page counter to 1 so body text starts at page 1,
        // regardless of how many front-matter pages preceded it.
        if meta.title.is_some() || meta.author.is_some() {
            s.push_str("\\pageno=1\n");
        }
    }

    s.push('\n');
    Ok(s)
}

/// Escapes special TeX characters in plain text (outside math mode).
fn tex_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str(r"\&"),
            '%' => out.push_str(r"\%"),
            '$' => out.push_str(r"\$"),
            '#' => out.push_str(r"\#"),
            '_' => out.push_str(r"\_"),
            '{' => out.push_str(r"\{"),
            '}' => out.push_str(r"\}"),
            '~' => out.push_str(r"\char126 "),
            '^' => out.push_str(r"\char94 "),
            '\\' => out.push_str(r"\char92 "),
            c => out.push(c),
        }
    }
    out
}

struct Context {
    dpi: u32,
    base_dir: Option<PathBuf>,
    images_dir: Option<PathBuf>,
    /// Directory where the resulting TeX file will live. `None` = Passthrough
    /// mode: image paths are written verbatim as they appear in the Markdown.
    /// `Some(dir)` = Rewrite mode: paths are resolved to absolute, or to a
    /// path relative to `dir` when `dir` equals the source base_dir.
    output_dir: Option<PathBuf>,
    nonum: bool,
    toc_depth: u32,
    /// Emit \caption/f{alt} after images and \caption/t after tables (academic style).
    captions: bool,
    footnotes: HashMap<String, String>,
    list_depth: u32,
    in_code_block: bool,
    in_image: bool,
    image_alt: String,
    in_footnote_def: bool,
    in_table_head: bool,
    col_alignments: Vec<Alignment>,
    col_index: usize,
    row_count: usize,
    /// True while inside a raw TeX code block (lang `tex`, `=tex`, `optex`).
    in_raw_tex: bool,
    /// True while inside a `praxe` callout block.
    in_callout: bool,
    /// Buffered text content of the current callout block.
    callout_buf: String,
    /// Set after a table ends (captions mode only); cleared by the next paragraph start.
    after_table: bool,
    /// True while buffering a potential table-caption paragraph.
    caption_para: bool,
    /// Byte offset in `out` where the current caption paragraph started.
    caption_start: usize,
    /// Raw text collected in the current caption paragraph (for prefix detection).
    caption_text: String,
    /// Resolved image path waiting for end_tag (deferred output).
    image_pending_path: Option<PathBuf>,
    /// Measured image width waiting for end_tag (deferred output).
    image_pending_width: Option<String>,
    /// Nesting depth of blockquote (0 = not inside any blockquote).
    in_blockquote: u32,
    /// True immediately after Tag::Paragraph starts inside a blockquote,
    /// before any content is emitted. Used to detect speaker labels (**D:** / **s:**).
    bq_para_start: bool,
    /// True while rendering a citelabel strong group (\citelabel{...}).
    in_cite_label: bool,
    /// Per-nesting-level flag: true if this blockquote contains speaker labels (= dialogue).
    bq_is_dialogue: Vec<bool>,
    /// Per-nesting-level byte offset in `out` where \begcitation was written.
    /// Used to retroactively replace it with \begdialogue when the first label is detected.
    bq_open_pos: Vec<usize>,
    /// Drop-cap feature master switch. When `false`, H1 headings do not trigger
    /// the \IC drop cap on the first paragraph.
    drop_cap_enabled: bool,
    /// True after H1 heading end, until the first paragraph of the chapter begins.
    drop_cap_pending: bool,
    /// True for the first Text event of the first body paragraph of a chapter.
    in_drop_cap_para: bool,
    /// True while inside a `part` fenced code block.
    in_part_block: bool,
    /// Buffered content of the current `part` fenced code block.
    part_buf: String,
    /// Pending `\label[id]` to emit after the current heading ends.
    pending_label: Option<String>,
    /// True when the current code block has line numbering enabled (\ttline).
    code_numbered: bool,
    /// Pre-scanned table attributes for each table (in order).
    table_attrs: Vec<TableAttrs>,
    /// Index into table_attrs, incremented for each table encountered.
    table_index: usize,
    /// True if the currently rendering table is a longtable.
    in_longtable: bool,
    /// Column widths (fractions of \hsize) for the current table, if specified.
    current_col_widths: Option<Vec<f64>>,
}

impl Context {
    #[allow(clippy::too_many_arguments)]
    fn new(
        dpi: u32,
        base_dir: Option<&Path>,
        images_dir: Option<&Path>,
        output_dir: Option<&Path>,
        nonum: bool,
        toc_depth: u32,
        captions: bool,
        drop_cap_enabled: bool,
        footnotes: HashMap<String, String>,
        table_attrs: Vec<TableAttrs>,
    ) -> Self {
        Self {
            dpi,
            base_dir: base_dir.map(|p| p.to_path_buf()),
            images_dir: images_dir.map(|p| p.to_path_buf()),
            output_dir: output_dir.map(|p| p.to_path_buf()),
            nonum,
            toc_depth,
            captions,
            drop_cap_enabled,
            footnotes,
            list_depth: 0,
            in_code_block: false,
            in_image: false,
            image_alt: String::new(),
            in_footnote_def: false,
            in_table_head: false,
            col_alignments: vec![],
            col_index: 0,
            row_count: 0,
            in_raw_tex: false,
            in_callout: false,
            callout_buf: String::new(),
            after_table: false,
            caption_para: false,
            caption_start: 0,
            caption_text: String::new(),
            image_pending_path: None,
            image_pending_width: None,
            in_blockquote: 0,
            bq_para_start: false,
            in_cite_label: false,
            bq_is_dialogue: Vec::new(),
            bq_open_pos: Vec::new(),
            drop_cap_pending: false,
            in_drop_cap_para: false,
            in_part_block: false,
            part_buf: String::new(),
            pending_label: None,
            code_numbered: false,
            table_attrs,
            table_index: 0,
            in_longtable: false,
            current_col_widths: None,
        }
    }

    /// Resolves an image path based on path-rewrite mode:
    ///
    /// * Passthrough (`output_dir = None`): return the path verbatim. Used
    ///   when writing to stdout — we don't know where the TeX will live, so
    ///   we leave the author's path alone.
    /// * Rewrite (`output_dir = Some(dir)`): produce a path that works from
    ///   `dir`. If `dir` equals the source `base_dir` (typical single-file
    ///   "TeX next to MD"), use a clean relative path. Otherwise use
    ///   an absolute path derived from `base_dir` / `images_dir`.
    fn resolve_image_path(&self, path: &str) -> PathBuf {
        let p = Path::new(path);

        let Some(output_dir) = self.output_dir.as_deref() else {
            return p.to_path_buf();
        };

        if p.is_absolute() {
            return p.to_path_buf();
        }

        let abs = self.locate_absolute(p);

        let same_dir = self
            .base_dir
            .as_deref()
            .and_then(|b| std::fs::canonicalize(b).ok())
            .zip(std::fs::canonicalize(output_dir).ok())
            .is_some_and(|(b, o)| b == o);

        if same_dir && let Some(abs) = abs.as_deref() {
            return pathdiff::diff_paths(abs, output_dir).unwrap_or_else(|| abs.to_path_buf());
        }
        abs.unwrap_or_else(|| p.to_path_buf())
    }

    /// Produces an absolute path for a relative image path by joining it with
    /// `images_dir` (preferred, if it exists) or `base_dir`. Returns `None`
    /// when neither yields anything usable.
    fn locate_absolute(&self, p: &Path) -> Option<PathBuf> {
        if let Some(img_dir) = &self.images_dir {
            let candidate = img_dir.join(p);
            if candidate.exists() {
                return Some(std::fs::canonicalize(&candidate).unwrap_or(candidate));
            }
        }
        if let Some(base) = &self.base_dir {
            let joined = base.join(p);
            return Some(std::fs::canonicalize(&joined).unwrap_or(joined));
        }
        None
    }

    fn handle_event(&mut self, event: Event, out: &mut String) {
        // Footnote definitions are collected in a pre-scan; skip them during rendering.
        if self.in_footnote_def {
            if let Event::End(TagEnd::FootnoteDefinition) = event {
                self.in_footnote_def = false;
            }
            return;
        }
        match event {
            Event::Start(tag) => self.start_tag(tag, out),
            Event::End(tag) => self.end_tag(tag, out),
            Event::Text(t) => {
                self.bq_para_start = false;
                if self.in_image {
                    self.image_alt.push_str(&t);
                } else if self.in_part_block {
                    self.part_buf.push_str(&tex_escape(&t));
                } else if self.in_callout {
                    let escaped = tex_escape(&t);
                    let processed = typo::apply(&escaped);
                    self.callout_buf.push_str(&processed);
                } else if self.in_raw_tex || self.in_code_block {
                    out.push_str(&t);
                } else {
                    if self.caption_para {
                        self.caption_text.push_str(&t);
                    }
                    if t.contains('\x0F') {
                        emit_text_with_spans(&t, out);
                    } else {
                        let escaped = tex_escape(&t);
                        let processed = typo::apply(&escaped);
                        if self.in_drop_cap_para {
                            self.in_drop_cap_para = false;
                            let raw = t.as_ref();
                            let mut chars = raw.chars();
                            if let Some(first) = chars.next() {
                                let rest = &raw[first.len_utf8()..];
                                let first_tex = tex_escape(&first.to_string());
                                let rest_tex = typo::apply(&tex_escape(rest));
                                out.push_str(&format!("\\IC{{{}}}{}", first_tex, rest_tex));
                            } else {
                                out.push_str(&processed);
                            }
                        } else {
                            out.push_str(&processed);
                        }
                    }
                }
            }
            Event::InlineMath(s) => {
                // Pass math content through verbatim — OpTeX handles $...$ natively.
                out.push_str(&format!("${s}$"));
            }
            Event::DisplayMath(s) => {
                // Display math: $$...$$ on its own lines.
                out.push_str(&format!("\n$${}$$\n\n", s.trim()));
            }
            Event::Code(t) => {
                out.push_str(&format!("{{\\tt {}}}", tex_escape(&t)));
            }
            Event::FootnoteReference(label) => {
                let body = self
                    .footnotes
                    .get(label.as_ref())
                    .cloned()
                    .unwrap_or_else(|| format!("?{label}"));
                out.push_str(&format!("\\fnote{{{body}}}"));
            }
            Event::TaskListMarker(checked) => {
                out.push_str(if checked { "[{\\tt x}]\\ " } else { "[\\ ]\\ " });
            }
            Event::SoftBreak => {
                if self.in_blockquote > 0 {
                    // Each line in a dialogue blockquote is a separate speaker turn.
                    // Treat soft breaks as paragraph breaks so every label gets \citelabel.
                    out.push_str("\\par\n");
                    self.bq_para_start = true;
                } else {
                    out.push('\n');
                }
            }
            Event::HardBreak => {
                if self.in_blockquote > 0 {
                    out.push_str("\\par\n");
                    self.bq_para_start = true;
                } else {
                    out.push_str("\\hfil\\break\n");
                }
            }
            Event::Rule => out.push_str("\\ornsep\n\n"),
            Event::Html(_) | Event::InlineHtml(_) => {} // discarded
        }
    }

    fn start_tag(&mut self, tag: Tag, out: &mut String) {
        match tag {
            Tag::Heading {
                level, id, classes, ..
            } => {
                let cmd = heading_cmd(level);
                let depth = heading_depth(level);
                let has_unnumbered = classes
                    .iter()
                    .any(|c| c.as_ref() == "unnumbered" || c.as_ref() == "-");
                let has_unlisted = classes.iter().any(|c| c.as_ref() == "unlisted");
                out.push('\n');
                if self.nonum || has_unnumbered {
                    out.push_str("\\nonum ");
                }
                if depth > self.toc_depth || has_unlisted {
                    out.push_str("\\notoc ");
                }
                out.push_str(&format!("{cmd} "));
                if let Some(id) = id {
                    self.pending_label = Some(id.to_string());
                }
            }
            Tag::Paragraph => {
                if self.captions && self.after_table {
                    self.caption_para = true;
                    self.caption_start = out.len();
                    self.caption_text.clear();
                }
                self.after_table = false;
                if self.in_blockquote > 0 {
                    self.bq_para_start = true;
                } else if self.drop_cap_pending {
                    self.drop_cap_pending = false;
                    self.in_drop_cap_para = true;
                }
            }
            Tag::Strong => {
                if self.bq_para_start && self.in_blockquote > 0 {
                    // First speaker label in this blockquote — mark it as dialogue and
                    // retroactively replace \begcitation with \begdialogue at the saved position.
                    // Both macros are exactly 12 ASCII bytes so the replacement is length-preserving.
                    if let Some(is_dia) = self.bq_is_dialogue.last_mut()
                        && !*is_dia
                    {
                        *is_dia = true;
                        if let Some(&pos) = self.bq_open_pos.last() {
                            out.replace_range(pos..pos + 12, "\\begdialogue");
                        }
                    }
                    self.in_cite_label = true;
                    out.push_str("\\citelabel{");
                } else {
                    out.push_str("{\\bf ");
                }
                self.bq_para_start = false;
            }
            Tag::Emphasis => out.push_str("{\\it "),
            Tag::Strikethrough => out.push_str("\\strike{"),
            Tag::CodeBlock(CodeBlockKind::Fenced(ref lang)) => {
                let (base_lang, cb_attrs) = split_code_block_attrs(lang);
                match base_lang {
                    "tex" | "=tex" | "optex" | "=optex" => {
                        self.in_raw_tex = true;
                    }
                    "praxe" => {
                        self.in_callout = true;
                        self.callout_buf.clear();
                    }
                    "part" => {
                        self.in_part_block = true;
                        self.part_buf.clear();
                    }
                    _ => {
                        self.in_code_block = true;
                        let has_number_lines = cb_attrs
                            .split_whitespace()
                            .any(|a| a == ".numberLines" || a == ".number-lines");
                        if has_number_lines {
                            let start_from = cb_attrs
                                .split_whitespace()
                                .find(|a| a.starts_with("startFrom="))
                                .and_then(|a| {
                                    let v = &a["startFrom=".len()..];
                                    v.trim_matches('"').parse::<i32>().ok()
                                })
                                .unwrap_or(1);
                            self.code_numbered = true;
                            out.push_str(&format!("\\ttline={start_from} \\begtt\n"));
                        } else {
                            out.push_str("\\begtt\n");
                        }
                    }
                }
            }
            Tag::CodeBlock(_) => {
                self.in_code_block = true;
                out.push_str("\\begtt\n");
            }
            Tag::List(None) => {
                self.list_depth += 1;
                out.push_str("\\begitems\n");
            }
            Tag::List(Some(_)) => {
                self.list_depth += 1;
                out.push_str("\\begitems \\style n\n");
            }
            Tag::Item => out.push_str("* "),
            Tag::BlockQuote(_) => {
                self.in_blockquote += 1;
                self.bq_open_pos.push(out.len());
                self.bq_is_dialogue.push(false);
                out.push_str("\\begcitation\n");
            }
            Tag::Link {
                dest_url,
                title: _,
                id: _,
                ..
            } => {
                out.push_str(&format!("\\ulink[{}]{{", dest_url));
            }
            Tag::Image { dest_url, .. } => {
                self.in_image = true;
                self.image_alt.clear();
                let resolved = self.resolve_image_path(&dest_url);
                let width = measure_image(&resolved, self.dpi);
                self.image_pending_path = Some(resolved);
                self.image_pending_width = Some(width);
            }
            Tag::Table(alignments) => {
                self.col_alignments = alignments;
                self.col_index = 0;
                self.row_count = 0;
                let attrs = self
                    .table_attrs
                    .get(self.table_index)
                    .cloned()
                    .unwrap_or_default();
                self.in_longtable = attrs.longtable;
                self.current_col_widths = attrs.col_widths;
                self.table_index += 1;
                let n = self.col_alignments.len().max(1);
                if self.in_longtable {
                    // Long table using \halign directly (allows page breaks between rows).
                    let halign_spec =
                        build_halign_spec(&self.col_alignments, n, &self.current_col_widths);
                    out.push_str(&format!(
                        "\\par\\medskip\n\\halign to\\hsize{{{}\\cr\n\\noalign{{\\hrule\\smallskip}}\n",
                        halign_spec
                    ));
                } else {
                    // Use p{dim} columns so cell text wraps instead of overflowing.
                    let spec = build_table_spec(&self.col_alignments, n, &self.current_col_widths);
                    out.push_str(&format!("\\par\\medskip\\noindent\n\\table{{{spec}}}{{\\noalign{{\\hrule\\smallskip}}\n"));
                }
            }
            Tag::TableHead => {
                self.in_table_head = true;
                self.col_index = 0;
            }
            Tag::TableRow => {
                self.col_index = 0;
                self.row_count += 1;
            }
            Tag::TableCell => {
                if self.col_index > 0 {
                    out.push_str(" & ");
                }
                // In p{} columns text wraps as a paragraph (left-aligned by default).
                // Add \hfil prefix/suffix to restore center/right alignment within the cell.
                match self.col_alignments.get(self.col_index) {
                    Some(Alignment::Center) => out.push_str("\\hfil "),
                    Some(Alignment::Right) => out.push_str("\\hfill "),
                    _ => {}
                }
            }
            Tag::Superscript => out.push_str("\\tsuper{"),
            Tag::Subscript => out.push_str("\\tsub{"),
            Tag::DefinitionList => out.push_str("\\par\\medskip\n"),
            Tag::DefinitionListTitle => out.push_str("\\noindent{\\bf "),
            Tag::DefinitionListDefinition => out.push_str("\\advance\\leftskip by 2em\\noindent "),
            Tag::FootnoteDefinition(_) => {
                self.in_footnote_def = true;
            }
            _ => {}
        }
    }

    fn end_tag(&mut self, tag: TagEnd, out: &mut String) {
        match tag {
            TagEnd::Heading(level) => {
                out.push('\n');
                if let Some(id) = self.pending_label.take() {
                    out.push_str(&format!("\\label[{}]\n", id));
                }
                if level == HeadingLevel::H1 && self.drop_cap_enabled {
                    self.drop_cap_pending = true;
                    self.in_drop_cap_para = false;
                }
            }
            TagEnd::Paragraph => {
                if self.caption_para {
                    self.caption_para = false;
                    let text = std::mem::take(&mut self.caption_text);
                    if let Some(body) = strip_caption_prefix(text.trim()) {
                        // Replace the buffered paragraph TeX with \caption/t.
                        out.truncate(self.caption_start);
                        out.push_str(&format!("\\caption/t {body}\n"));
                    } else {
                        out.push_str("\n\n");
                    }
                } else {
                    out.push_str("\n\n");
                }
            }
            TagEnd::Strong => {
                if self.in_cite_label {
                    self.in_cite_label = false;
                    // A speaker label must end with ':' (e.g. "DOM:" / "sub:").
                    // If it doesn't (e.g. "Varování."), undo the dialogue detection
                    // and fall back to a plain bold group.
                    if out.ends_with(':') {
                        out.push('}'); // close \citelabel{...}
                    } else {
                        // Undo \citelabel{ → {\bf  (different lengths — do before bq_pos fix)
                        if let Some(cite_pos) = out.rfind("\\citelabel{") {
                            out.replace_range(cite_pos..cite_pos + 11, "{\\bf ");
                        }
                        // Undo \begdialogue → \begcitation (same length: 12 chars each)
                        if let Some(is_dia) = self.bq_is_dialogue.last_mut()
                            && *is_dia
                        {
                            *is_dia = false;
                            if let Some(&bq_pos) = self.bq_open_pos.last()
                                && out[bq_pos..].starts_with("\\begdialogue")
                            {
                                out.replace_range(bq_pos..bq_pos + 12, "\\begcitation");
                            }
                        }
                        out.push('}'); // close {\bf ...}
                    }
                } else {
                    out.push('}');
                }
            }
            TagEnd::Emphasis | TagEnd::Strikethrough => out.push('}'),
            TagEnd::CodeBlock => {
                if self.in_raw_tex {
                    self.in_raw_tex = false;
                } else if self.in_part_block {
                    self.in_part_block = false;
                    let body = std::mem::take(&mut self.part_buf);
                    out.push_str(&format!("\\partpage{{{}}}\n\n", body.trim()));
                } else if self.in_callout {
                    self.in_callout = false;
                    let body = std::mem::take(&mut self.callout_buf);
                    // Attach \fnote inline to the preceding paragraph by stripping
                    // the trailing blank line that TagEnd::Paragraph already emitted.
                    let trimmed = out.trim_end_matches('\n').len();
                    out.truncate(trimmed);
                    out.push_str(&format!("\\fnote{{{}}}\n\n", body.trim()));
                } else {
                    self.in_code_block = false;
                    out.push_str("\\endtt\n");
                    if self.code_numbered {
                        self.code_numbered = false;
                        out.push_str("\\ttline=-1\n");
                    }
                    out.push('\n');
                }
            }
            TagEnd::List(_) => {
                self.list_depth -= 1;
                out.push_str("\\enditems\n\n");
            }
            TagEnd::Item => out.push('\n'),
            TagEnd::BlockQuote(_) => {
                if self.in_blockquote > 0 {
                    self.in_blockquote -= 1;
                }
                self.bq_para_start = false;
                let is_dia = self.bq_is_dialogue.pop().unwrap_or(false);
                self.bq_open_pos.pop();
                if is_dia {
                    out.push_str("\\enddialogue\n\n");
                } else {
                    out.push_str("\\endcitation\n\n");
                }
            }
            TagEnd::Link => out.push('}'),
            TagEnd::Superscript => out.push('}'),
            TagEnd::Subscript => out.push('}'),
            TagEnd::DefinitionListTitle => {
                out.push_str("}\\par\\nobreak\n");
            }
            TagEnd::DefinitionListDefinition => {
                out.push_str("\\par\\advance\\leftskip by -2em\n");
            }
            TagEnd::DefinitionList => out.push_str("\\par\\medskip\n\n"),
            TagEnd::Image => {
                self.in_image = false;
                let raw_alt = std::mem::take(&mut self.image_alt);
                let (alt, attrs) = split_image_alt(&raw_alt);
                let is_fullpage = attrs
                    .split_whitespace()
                    .any(|a| a == ".fullpage" || a == "fullpage");
                let is_chapter = attrs
                    .split_whitespace()
                    .any(|a| a == ".chapter" || a == "chapter");
                // Parse optional width= attribute: width=8cm or width=70%
                let attr_width: Option<String> = attrs
                    .split_whitespace()
                    .find(|a| a.starts_with("width="))
                    .map(|a| {
                        let v = &a["width=".len()..];
                        if let Some(pct_str) = v.strip_suffix('%') {
                            let pct: f64 = pct_str.parse().unwrap_or(100.0);
                            format!("{:.4}\\hsize", pct / 100.0)
                        } else {
                            v.to_owned()
                        }
                    });
                if let (Some(path), Some(auto_width)) = (
                    self.image_pending_path.take(),
                    self.image_pending_width.take(),
                ) {
                    let width = attr_width.as_deref().unwrap_or(&auto_width);
                    if is_fullpage {
                        out.push_str(&format!(
                            "\\vfil\\eject\n\
                             \\bgroup\\footline={{}}\\headline={{}}\n\
                             \\vbox to\\vsize{{\\vfil\\picw=\\hsize \\inspic {} \\vfil}}\n\
                             \\eject\n\
                             \\egroup\n",
                            path.display()
                        ));
                    } else if is_chapter {
                        out.push_str(&format!(
                            "\\chapterimage{{{width}}}{{{}}}\n",
                            path.display()
                        ));
                    } else {
                        out.push_str(&format!("\\centimage{{{width}}}{{{}}}\n", path.display()));
                        if self.captions && !alt.is_empty() {
                            out.push_str(&format!("\\caption/f {alt}\n"));
                        }
                    }
                }
            }
            TagEnd::TableHead => {
                self.in_table_head = false;
                if self.in_longtable {
                    out.push_str(" \\cr\\noalign{\\smallskip\\hrule\\smallskip}\n");
                } else {
                    out.push_str(" \\crli\n");
                }
            }
            TagEnd::TableRow => {
                out.push_str(" \\cr\n");
            }
            TagEnd::TableCell => {
                if let Some(Alignment::Center) = self.col_alignments.get(self.col_index) {
                    out.push_str(" \\hfil");
                }
                self.col_index += 1;
            }
            TagEnd::Table => {
                if self.in_longtable {
                    out.push_str("\\noalign{\\smallskip\\hrule}\n}\n\\par\\medskip\n\n");
                    self.in_longtable = false;
                } else {
                    out.push_str("\\noalign{\\smallskip\\hrule}\n}\\par\\medskip\n\n");
                }
                if self.captions {
                    self.after_table = true;
                }
            }
            _ => {}
        }
    }
}

fn heading_cmd(level: HeadingLevel) -> &'static str {
    match level {
        HeadingLevel::H1 => "\\chap",
        HeadingLevel::H2 => "\\sec",
        HeadingLevel::H3 => "\\secc",
        _ => "\\seccc",
    }
}

fn heading_depth(level: HeadingLevel) -> u32 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        _ => 4,
    }
}

/// Returns the caption body if `text` starts with Pandoc-style caption prefix (`: `),
/// otherwise `None`.
fn strip_caption_prefix(text: &str) -> Option<&str> {
    text.strip_prefix(':').map(|rest| rest.trim())
}

/// Extracts and strips a YAML front matter block (`---\n…\n---\n`) from the start
/// of `markdown`. Returns `(Some(Metadata), rest)` when found, otherwise `(None, markdown)`.
fn extract_yaml_front_matter(markdown: &str) -> (Option<Metadata>, &str) {
    if !markdown.starts_with("---\n") {
        return (None, markdown);
    }
    let after_open = &markdown[4..]; // skip opening "---\n"
    if let Some(close_pos) = after_open.find("\n---\n") {
        let yaml = &after_open[..close_pos];
        let rest = &after_open[close_pos + 5..]; // skip "\n---\n"
        (Some(Metadata::from_yaml_str(yaml)), rest)
    } else if let Some(yaml) = after_open.strip_suffix("\n---") {
        (Some(Metadata::from_yaml_str(yaml)), "")
    } else {
        (None, markdown)
    }
}

/// Emits text containing `\x0F` span sentinels. Normal text is escaped + typo-processed;
/// sentinel segments (`\x0Fcmd:text\x0F`) are emitted as raw TeX.
fn emit_text_with_spans(t: &str, out: &mut String) {
    let parts: Vec<&str> = t.split('\x0F').collect();
    for (i, part) in parts.iter().enumerate() {
        if i % 2 == 0 {
            // Normal text segment
            if !part.is_empty() {
                let escaped = tex_escape(part);
                out.push_str(&typo::apply(&escaped));
            }
        } else {
            // Span sentinel: "cmd:text"
            if let Some((cmd, text)) = part.split_once(':') {
                let escaped_text = tex_escape(text);
                let processed_text = typo::apply(&escaped_text);
                match cmd {
                    "sc" => out.push_str(&format!("{{\\caps {processed_text}}}")),
                    "underline" => out.push_str(&format!("\\underbar{{{processed_text}}}")),
                    "mark" => out.push_str(&format!("\\highlight{{{processed_text}}}")),
                    _ => out.push_str(&processed_text),
                }
            }
        }
    }
}

/// Returns the column width TeX expression for column `i` of `n`.
/// If custom widths are provided, uses a proportional fraction of the available
/// space (`\hsize` minus inter-column tabskip glue).
fn col_width_expr(i: usize, n: usize, col_widths: &Option<Vec<f64>>) -> String {
    if let Some(widths) = col_widths
        && let Some(&w) = widths.get(i)
    {
        // Available space = \hsize - (n-1)*1em (tabskip gaps).
        // Column width = w * available = w*\hsize - w*(n-1)em.
        let gaps = n.saturating_sub(1);
        let tabskip_share = w * gaps as f64;
        let fmt = |v: f64| {
            let s = format!("{:.4}", v);
            let s = s.trim_end_matches('0');
            s.trim_end_matches('.').to_string()
        };
        let w_str = fmt(w);
        let t_str = fmt(tabskip_share);
        return format!("\\dimexpr {w_str}\\hsize-{t_str}em\\relax");
    }
    format!("\\dimexpr(\\hsize - {}em)/{}\\relax", n, n)
}

/// Builds the column spec for a regular `\table{...}` command.
fn build_table_spec(_alignments: &[Alignment], n: usize, col_widths: &Option<Vec<f64>>) -> String {
    (0..n)
        .map(|i| format!("p{{{}}}", col_width_expr(i, n, col_widths)))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Builds a `\halign` preamble spec for longtable columns.
/// Each column uses `\vtop{…}` for paragraph wrapping.
fn build_halign_spec(alignments: &[Alignment], n: usize, col_widths: &Option<Vec<f64>>) -> String {
    let mut parts = Vec::new();
    for (i, align) in alignments.iter().enumerate() {
        let width = col_width_expr(i, n, col_widths);
        let (pre, post) = match align {
            Alignment::Center => ("\\hfil ", " \\hfil"),
            Alignment::Right => ("\\hfill ", ""),
            _ => ("", ""),
        };
        let tabskip = if i + 1 < n {
            "\\tabskip=1em"
        } else {
            "\\tabskip=0pt plus1fil"
        };
        parts.push(format!(
            "\\vtop{{\\hsize={width}\\noindent\\strut{pre}#\\strut{post}}}{tabskip}"
        ));
    }
    // Pad if alignments is shorter than n
    while parts.len() < n {
        let width = col_width_expr(parts.len(), n, col_widths);
        let tabskip = if parts.len() + 1 < n {
            "\\tabskip=1em"
        } else {
            "\\tabskip=0pt plus1fil"
        };
        parts.push(format!(
            "\\vtop{{\\hsize={width}\\noindent\\strut#\\strut}}{tabskip}"
        ));
    }
    format!("\\tabskip=0pt{}", parts.join("&\n  "))
}

/// Checks if a line is a GFM table separator row (e.g. `|:---|---:|:---:|`).
fn is_separator_row(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|')
        && trimmed
            .chars()
            .all(|c| c == '|' || c == '-' || c == ':' || c == ' ')
        && trimmed.contains('-')
}

/// Derives proportional column widths from cell widths in a separator row.
/// Uses the total character count of each cell (between `|` delimiters) so that
/// alignment colons don't skew the result.
/// Returns `None` if all columns have the same width (use default equal distribution)
/// or if the widths differ by less than 20% from equal.
fn derive_col_widths_from_separator(separator: &str) -> Option<Vec<f64>> {
    let trimmed = separator.trim();
    // Split by '|', skip empty leading/trailing segments
    let cells: Vec<&str> = trimmed
        .split('|')
        .filter(|s| !s.trim().is_empty())
        .collect();
    if cells.len() < 2 {
        return None;
    }
    let char_counts: Vec<usize> = cells.iter().map(|cell| cell.len()).collect();
    let total: usize = char_counts.iter().sum();
    if total == 0 {
        return None;
    }
    let n = char_counts.len();
    let equal_share = 1.0 / n as f64;
    let widths: Vec<f64> = char_counts
        .iter()
        .map(|&c| c as f64 / total as f64)
        .collect();
    // Only use proportional widths if at least one column deviates >20% from equal
    let dominated_by_equal = widths
        .iter()
        .all(|&w| (w - equal_share).abs() / equal_share < 0.2);
    if dominated_by_equal {
        return None;
    }
    Some(widths)
}

/// Pre-processes table attribute lines from markdown.
/// Returns `(cleaned_markdown, table_attrs)` where `table_attrs[i]` contains
/// attributes for the i-th table.
fn preprocess_table_attrs(input: &str) -> (String, Vec<TableAttrs>) {
    let lines: Vec<&str> = input.lines().collect();
    let mut out_lines = Vec::with_capacity(lines.len());
    let mut attrs_list: Vec<TableAttrs> = Vec::new();
    let mut skip_next = false;
    let mut table_line_count = 0usize;

    for (i, &line) in lines.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        let trimmed = line.trim();
        // Detect attribute block `{...}` on a line by itself after a table
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            let inner = trimmed[1..trimmed.len() - 1].trim();
            if inner.split_whitespace().any(|a| a == ".longtable") {
                // Check if this follows a table (look backward for last non-empty line being a table row)
                let prev_non_empty = lines[..i].iter().rev().find(|l| !l.trim().is_empty());
                if let Some(prev) = prev_non_empty
                    && (prev.trim().starts_with('|') || prev.trim().ends_with('|'))
                {
                    if let Some(last) = attrs_list.last_mut() {
                        last.longtable = true;
                    }
                    continue;
                }
            }
        }
        // Track table starts and separator rows
        if trimmed.starts_with('|') {
            let prev_non_empty = out_lines
                .iter()
                .rev()
                .find(|l: &&String| !l.trim().is_empty());
            let prev_is_table = prev_non_empty
                .map(|l: &String| l.trim().starts_with('|'))
                .unwrap_or(false);
            if !prev_is_table {
                // New table starts
                attrs_list.push(TableAttrs::default());
                table_line_count = 1;
            } else {
                table_line_count += 1;
                // Second line of a table — the separator row
                if table_line_count == 2
                    && is_separator_row(trimmed)
                    && let Some(last) = attrs_list.last_mut()
                    && last.col_widths.is_none()
                {
                    last.col_widths = derive_col_widths_from_separator(trimmed);
                }
            }
        } else {
            table_line_count = 0;
        }
        out_lines.push(line.to_string());
    }

    // Reconstruct with original line endings
    let result = if input.ends_with('\n') {
        out_lines.join("\n") + "\n"
    } else {
        out_lines.join("\n")
    };
    (result, attrs_list)
}

/// Splits a fenced code block info string into `(language, attrs)`.
/// E.g. `python {.numberLines startFrom="5"}` → `("python", ".numberLines startFrom=\"5\"")`.
fn split_code_block_attrs(info: &str) -> (&str, &str) {
    let trimmed = info.trim();
    if let Some(brace_pos) = trimmed.find('{') {
        let lang = trimmed[..brace_pos].trim();
        let rest = &trimmed[brace_pos..];
        // Strip outer braces
        let attrs = rest
            .strip_prefix('{')
            .and_then(|s| s.strip_suffix('}'))
            .unwrap_or(rest)
            .trim();
        (lang, attrs)
    } else {
        (trimmed, "")
    }
}

/// Splits raw alt text into `(display_alt, attrs_str)`.
/// The `\x0E` sentinel is inserted by `preprocess_image_attrs` to encode Pandoc-style
/// image attributes `{...}` into the alt text before pulldown-cmark parsing.
fn split_image_alt(raw: &str) -> (&str, &str) {
    match raw.find('\x0E') {
        Some(pos) => (&raw[..pos], &raw[pos + '\x0E'.len_utf8()..]),
        None => (raw, ""),
    }
}

/// Pre-processes Pandoc-style span attributes `[text]{.class}` before pulldown-cmark parsing.
///
/// Converts `[text]{.smallcaps}` → `\x0Fsc:text\x0F`, etc.
/// The `\x0F` (ASCII SI) sentinels are detected in the text handler and emitted as raw TeX.
fn preprocess_span_attrs(input: &str) -> String {
    let mut result_lines: Vec<String> = Vec::new();
    let mut in_fenced = false;
    let mut fence_marker = String::new();

    for line in input.split('\n') {
        if in_fenced {
            result_lines.push(line.to_string());
            let trimmed = line.trim();
            if trimmed.starts_with(&fence_marker)
                && trimmed[fence_marker.len()..]
                    .chars()
                    .all(|c| c == '`' || c.is_whitespace())
            {
                in_fenced = false;
            }
            continue;
        }
        // Detect fenced code block opening
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            let bt_count = trimmed.chars().take_while(|&c| c == '`').count();
            fence_marker = "`".repeat(bt_count);
            in_fenced = true;
            result_lines.push(line.to_string());
            continue;
        }
        // Process spans on this line
        let chars: Vec<char> = line.chars().collect();
        let mut out = String::with_capacity(line.len());
        let mut i = 0;
        while i < chars.len() {
            // Skip inline code
            if chars[i] == '`' {
                let bt_start = i;
                while i < chars.len() && chars[i] == '`' {
                    i += 1;
                }
                let bt_count = i - bt_start;
                // emit opening backticks
                for _ in 0..bt_count {
                    out.push('`');
                }
                // find matching closing backticks
                loop {
                    if i >= chars.len() {
                        break;
                    }
                    let peek_start = i;
                    let mut peek_count = 0;
                    while i < chars.len() && chars[i] == '`' {
                        peek_count += 1;
                        i += 1;
                    }
                    if peek_count == bt_count {
                        for _ in 0..bt_count {
                            out.push('`');
                        }
                        break;
                    }
                    if peek_count > 0 {
                        let text: String = chars[peek_start..i].iter().collect();
                        out.push_str(&text);
                    } else {
                        out.push(chars[i]);
                        i += 1;
                    }
                }
                continue;
            }
            if chars[i] == '['
                && let Some((consumed, replacement)) = try_parse_span_with_attrs(&chars, i)
            {
                out.push_str(&replacement);
                i += consumed;
                continue;
            }
            out.push(chars[i]);
            i += 1;
        }
        result_lines.push(out);
    }
    result_lines.join("\n")
}

/// Attempts to parse `[text]{.class}` at position `start` (position of `[`).
/// Returns `(chars_consumed, replacement_text)` on success.
fn try_parse_span_with_attrs(chars: &[char], start: usize) -> Option<(usize, String)> {
    let mut i = start + 1; // skip '['

    // Collect text up to matching ']'
    let text_start = i;
    let mut depth = 1usize;
    while i < chars.len() {
        match chars[i] {
            '\\' => i += 1,
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        i += 1;
    }
    if chars.get(i) != Some(&']') {
        return None;
    }
    let text: String = chars[text_start..i].iter().collect();
    i += 1; // skip ']'

    // Must have '{' immediately after ']' (not '(' which is a link)
    if chars.get(i) != Some(&'{') {
        return None;
    }
    i += 1; // skip '{'

    // Collect attrs up to '}'
    let attrs_start = i;
    while i < chars.len() && chars[i] != '}' {
        i += 1;
    }
    if chars.get(i) != Some(&'}') {
        return None;
    }
    let attrs: String = chars[attrs_start..i].iter().collect();
    let attrs = attrs.trim();
    i += 1; // skip '}'

    if attrs.is_empty() {
        return None;
    }

    // Map known classes to TeX sentinel markers
    let tex_cmd = if attrs.split_whitespace().any(|a| a == ".smallcaps") {
        Some("sc")
    } else if attrs.split_whitespace().any(|a| a == ".underline") {
        Some("underline")
    } else if attrs.split_whitespace().any(|a| a == ".mark") {
        Some("mark")
    } else {
        None
    };

    tex_cmd.map(|cmd| (i - start, format!("\x0F{}:{}\x0F", cmd, text)))
}

/// Pre-processes Pandoc-style image attribute syntax before pulldown-cmark parsing.
///
/// Transforms `![alt](url){attrs}` → `![alt\x0Eattrs](url)`, encoding the attribute
/// block into the alt text via an `\x0E` (ASCII SO) sentinel. pulldown-cmark does not
/// parse image attributes, so they must be extracted at this stage.
fn preprocess_image_attrs(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '!'
            && chars.get(i + 1) == Some(&'[')
            && let Some((consumed, replacement)) = try_parse_image_with_attrs(&chars, i)
        {
            out.push_str(&replacement);
            i += consumed;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Attempts to parse `![alt](url){attrs}` at `start` (position of `!`).
/// Returns `(chars_consumed, replacement_text)` on success, `None` otherwise.
fn try_parse_image_with_attrs(chars: &[char], start: usize) -> Option<(usize, String)> {
    let mut i = start + 2; // skip "!["

    // Collect alt text up to matching ']'
    let alt_start = i;
    let mut depth = 1usize;
    while i < chars.len() {
        match chars[i] {
            '\\' => i += 1,
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        i += 1;
    }
    if chars.get(i) != Some(&']') {
        return None;
    }
    let alt: String = chars[alt_start..i].iter().collect();
    i += 1; // skip ']'

    // Must have '('
    if chars.get(i) != Some(&'(') {
        return None;
    }
    i += 1; // skip '('

    // Collect URL up to matching ')'
    let url_start = i;
    let mut depth = 1usize;
    while i < chars.len() {
        match chars[i] {
            '\\' => i += 1,
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        i += 1;
    }
    if chars.get(i) != Some(&')') {
        return None;
    }
    let url: String = chars[url_start..i].iter().collect();
    i += 1; // skip ')'

    // Check for '{' immediately after ')'
    if chars.get(i) != Some(&'{') {
        return None;
    }
    i += 1; // skip '{'

    // Collect attrs up to '}'
    let attrs_start = i;
    while i < chars.len() && chars[i] != '}' {
        i += 1;
    }
    if chars.get(i) != Some(&'}') {
        return None;
    }
    let attrs: String = chars[attrs_start..i].iter().collect();
    let attrs = attrs.trim().to_string();
    i += 1; // skip '}'

    if attrs.is_empty() {
        return None;
    }

    Some((i - start, format!("![{}\x0E{}]({})", alt, attrs, url)))
}

fn measure_image(path: &Path, dpi: u32) -> String {
    match imagesize::size(path) {
        Ok(dim) => {
            let width_cm = dim.width as f64 / dpi as f64 * 2.54;
            if width_cm > 15.0 {
                "\\hsize".to_owned()
            } else {
                format!("{:.2}cm", width_cm)
            }
        }
        Err(_) => {
            eprintln!(
                "Warning: cannot measure image '{}', using \\hsize",
                path.display()
            );
            "\\hsize".to_owned()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tschichold_b5_has_asymmetric_margins() {
        let (inner, outer, top, bottom) = tschichold_margins("b5");
        assert!(outer > inner, "outer margin must be larger than inner");
        assert!(bottom > top, "bottom margin must be larger than top");
        assert_eq!((inner, outer, top, bottom), (18, 36, 22, 44));
    }

    #[test]
    fn tschichold_known_papers() {
        assert_eq!(tschichold_margins("a5"), (15, 30, 18, 36));
        assert_eq!(tschichold_margins("a4"), (25, 50, 30, 60));
        assert_eq!(tschichold_margins("letter"), (22, 44, 26, 52));
    }

    #[test]
    fn tschichold_unknown_falls_back() {
        // Any unknown paper size returns a conservative fallback, not a panic.
        let (inner, outer, top, bottom) = tschichold_margins("junk");
        assert!(outer > inner);
        assert!(bottom > top);
    }
}
