use std::collections::HashMap;
use std::path::{Path, PathBuf};

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, HeadingLevel, Alignment};

use crate::error::Error;
use crate::metadata::{Metadata, TocValue};
use crate::styles;
use crate::typo;

#[derive(Debug, PartialEq, Clone, Copy)]
enum TocPlacement { Front, Back }

/// Resolves TOC placement from metadata + style default.
/// Style "book" defaults to Back; all others default to Front.
fn resolve_toc(toc: Option<&TocValue>, style_name: Option<&str>) -> Option<TocPlacement> {
    let style_default = match style_name {
        Some("book") => TocPlacement::Back,
        _            => TocPlacement::Front,
    };
    match toc {
        None | Some(TocValue::Bool(false)) => None,
        Some(TocValue::Bool(true))         => Some(style_default),
        Some(TocValue::Position(s)) if s == "back"  => Some(TocPlacement::Back),
        Some(TocValue::Position(_))        => Some(TocPlacement::Front),
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
) -> Result<String, Error> {
    let style_name = style.or_else(|| {
        metadata
            .and_then(|m| m.style.as_ref())
            .and_then(|s| s.name.as_deref())
    });
    let toc = metadata
        .and_then(|m| m.book.as_ref())
        .and_then(|b| b.toc.as_ref());
    let toc_placement = resolve_toc(toc, style_name);

    // nonum: suppress heading numbers (book style convention)
    let nonum = style_name == Some("book");
    // toc_depth: max heading level included in TOC (book default = 1, others = no limit)
    let toc_depth = metadata
        .and_then(|m| m.typesetting.as_ref())
        .and_then(|t| t.toc_depth)
        .unwrap_or(if nonum { 1 } else { u32::MAX });

    let is_book = style_name == Some("book");

    let mut out = String::new();
    out.push_str(&build_preamble(metadata, hyphenation, style, toc_placement, is_book)?);
    let images_dir = metadata
        .and_then(|m| m.paths.as_ref())
        .and_then(|p| p.images.as_deref())
        .and_then(|rel| base_dir.map(|b| b.join(rel)));
    let captions = style_name == Some("academic");
    out.push_str(&render_body_impl(markdown, dpi, base_dir, images_dir.as_deref(), nonum, toc_depth, captions));
    if toc_placement == Some(TocPlacement::Back) {
        out.push_str(&toc_block(TocPlacement::Back));
    }
    if is_book {
        if let Some(meta) = metadata {
            out.push_str(&back_colophon_block(meta));
        }
    }
    out.push_str("\n\\bye\n");
    Ok(out)
}

/// Renders only the document body (no preamble, no `\bye`).
/// Used by integration tests; uses neutral defaults (no nonum, unlimited TOC depth).
pub fn render_body(markdown: &str, dpi: u32, base_dir: Option<&Path>, images_dir: Option<&Path>) -> String {
    render_body_impl(markdown, dpi, base_dir, images_dir, false, u32::MAX, false)
}

/// Like `render_body` but with captions enabled (academic style convention).
pub fn render_body_captions(markdown: &str, dpi: u32, base_dir: Option<&Path>, images_dir: Option<&Path>) -> String {
    render_body_impl(markdown, dpi, base_dir, images_dir, false, u32::MAX, true)
}

fn render_body_impl(markdown: &str, dpi: u32, base_dir: Option<&Path>, images_dir: Option<&Path>, nonum: bool, toc_depth: u32, captions: bool) -> String {
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES;

    let footnotes = collect_footnotes(markdown, opts);
    let parser = Parser::new_ext(markdown, opts);
    let mut ctx = Context::new(dpi, base_dir, images_dir, nonum, toc_depth, captions, footnotes);
    let mut out = String::new();

    for event in parser {
        ctx.handle_event(event, &mut out);
    }
    out
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
            _ if current_label.is_some() => {
                match &event {
                    Event::Start(_) => depth += 1,
                    Event::End(_)   => { if depth > 0 { depth -= 1; } }
                    Event::Text(t)  => {
                        let escaped = tex_escape(t);
                        let processed = typo::apply(&escaped);
                        current_body.push_str(&processed);
                    }
                    Event::Code(t)     => current_body.push_str(&format!("{{\\tt {}}}", tex_escape(t))),
                    Event::SoftBreak   => current_body.push(' '),
                    Event::HardBreak   => current_body.push(' '),
                    _ => {}
                }
            }
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
    let left   = parts.first().map(|s| s.trim()).unwrap_or("");
    let center = parts.get(1).map(|s| s.trim()).unwrap_or("");
    let right  = parts.get(2).map(|s| s.trim()).unwrap_or("");
    let subst = |s: &str| {
        s.replace("{author}", "\\theauthor")
         .replace("{title}",  "\\thetitle")
         .replace("{folio}",  "\\folio")
    };
    format!("{}\\hfil {}\\hfil {}", subst(left), subst(center), subst(right))
}

/// Generates the back colophon (tiráž) for book style — placed at the very end
/// of the document, before `\bye`. The content is pushed to the bottom of the page.
/// Only emitted when at least one of copyright/year/isbn is present in metadata.
fn back_colophon_block(metadata: &Metadata) -> String {
    let book = match &metadata.book {
        Some(b) => b,
        None => return String::new(),
    };
    let has_content = book.copyright.is_some() || book.year.is_some() || book.isbn.is_some();
    if !has_content {
        return String::new();
    }

    let mut s = String::new();
    s.push_str("\n\\vfil\\supereject\n");
    s.push_str("\\bgroup\\footline={}\\headline={}\n");
    s.push_str("\\null\\vfil\n");

    if book.title.is_some() || book.author.is_some() {
        if book.title.is_some() {
            s.push_str("\\noindent {\\bf\\thetitle}\\par\n");
        }
        if book.author.is_some() {
            s.push_str("\\noindent {\\it\\theauthor}\\par\n");
        }
        s.push_str("\\smallskip\n");
    }

    if let Some(cr) = &book.copyright {
        s.push_str(&format!("\\noindent {cr}\\par\n"));
    } else if let (Some(year), Some(author)) = (&book.year, &book.author) {
        s.push_str(&format!("\\noindent \\char169 \\ {year} {author}\\par\n"));
    } else if let Some(year) = &book.year {
        s.push_str(&format!("\\noindent \\char169 \\ {year}\\par\n"));
    }

    if let Some(isbn) = &book.isbn {
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
) -> Result<String, Error> {
    let mut s = String::new();

    // OpTeX is a LuaTeX format — no \input optex needed, it is pre-loaded by the engine.
    s.push_str("\\fontfam[LM]\n"); // Latin Modern Unicode — required for Czech characters
    s.push_str("\\uselanguage{czech}\n");
    // \begcitation/\endcitation are not part of OpTeX; define them here.
    s.push_str("\\def\\begcitation{\\par\\medskip\\leftskip=2em\\rightskip=2em\\noindent}\n");
    s.push_str("\\def\\endcitation{\\par\\leftskip=0em\\rightskip=0em\\medskip}\n");
    // \maketitle is not built into OpTeX; define it here.
    // vertical fill, title (via \tit), author in italics, vertical fill, page break.
    s.push_str("\\def\\maketitle{\\bgroup\\footline={}\\headline={}\\vglue0pt plus1fill\\centerline{{\\typosize[18/22]\\bf\\thetitle}}\\medskip\\centerline{{\\it\\theauthor}}\\vglue0pt plus2fill\\eject\\egroup}\n");
    // \strike is not built into OpTeX; draw a mid-height rule over the text.
    s.push_str("\\def\\strike#1{\\leavevmode\\setbox0=\\hbox{#1}\\hbox{\\copy0\\kern-\\wd0\\vrule height0.55em depth-0.45em width\\wd0}}\n");

    // Resolve and inject style: CLI --style takes priority over metadata [styl].
    let style_name = style.or_else(|| {
        metadata
            .and_then(|m| m.style.as_ref())
            .and_then(|st| st.name.as_deref())
    });
    if let Some(name) = style_name {
        match styles::resolve(name, None) {
            Some(content) => s.push_str(&content),
            None => eprintln!("md2optex: warning: style '{name}' not found, using defaults"),
        }
    }

    // Metadata overrides: applied after the style so they take precedence.
    if let Some(meta) = metadata
        && let Some(ts) = &meta.typesetting
    {
        if let Some(font) = &ts.font {
            s.push_str(&format!("\\fontfam[{}]\n", font));
        }
        if let Some(size) = &ts.base_size {
            // e.g. "11pt" → \typosize[11/13]
            let pt: u32 = size.trim_end_matches("pt").parse().unwrap_or(10);
            let leading = pt * 13 / 10;
            s.push_str(&format!("\\typosize[{pt}/{leading}]\n"));
        }
        // Emit \margins whenever paper size or any margin is specified in metadata.
        // This lets `papir = "b5"` work without requiring explicit margin values.
        let has_paper = ts.paper.is_some();
        let has_margins = ts.margin_left.is_some()
            || ts.margin_right.is_some()
            || ts.margin_top.is_some()
            || ts.margin_bottom.is_some();
        if has_paper || has_margins {
            let paper = ts.paper.as_deref().unwrap_or("a4");
            let left = ts.margin_left.unwrap_or(25);
            let right = ts.margin_right.unwrap_or(25);
            let top = ts.margin_top.unwrap_or(30);
            let bottom = ts.margin_bottom.unwrap_or(30);
            s.push_str(&format!(
                "\\margins/1 {paper} ({left},{right},{top},{bottom})mm\n"
            ));
        }
        if let Some(header) = &ts.header {
            s.push_str(&format!("\\headline={{{}}}\n", running_line(header)));
        }
        if let Some(footer) = &ts.footer {
            s.push_str(&format!("\\footline={{{}}}\n", running_line(footer)));
        }
        if ts.paragraph.as_deref() == Some("noindent") {
            s.push_str("\\parindent=0pt\n");
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

    if let Some(meta) = metadata
        && let Some(book) = &meta.book
    {
        if let Some(title) = &book.title {
            s.push_str(&format!("\\gdef\\thetitle{{{title}}}\n"));
        }
        if let Some(author) = &book.author {
            s.push_str(&format!("\\gdef\\theauthor{{{author}}}\n"));
        }
        if book.title.is_some() || book.author.is_some() {
            s.push_str("\\maketitle\n");
            // Verso of title page (page 2):
            // - Book style: colophon is deferred to the end (tiráž); emit blank verso.
            // - Other styles: emit copyright/ISBN here if available, otherwise blank.
            let has_colophon = !is_book
                && (book.copyright.is_some() || book.year.is_some() || book.isbn.is_some());
            if has_colophon {
                s.push_str("\\bgroup\\footline={}\\headline={}\n");
                s.push_str("\\null\\vfil\n");
                if let Some(cr) = &book.copyright {
                    s.push_str(&format!("\\noindent {cr}\\par\n"));
                } else if let (Some(year), Some(author)) = (&book.year, &book.author) {
                    s.push_str(&format!("\\noindent \\char169 \\ {year} {author}\\par\n"));
                }
                if let Some(isbn) = &book.isbn {
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
        if book.title.is_some() || book.author.is_some() {
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
            '&'  => out.push_str(r"\&"),
            '%'  => out.push_str(r"\%"),
            '$'  => out.push_str(r"\$"),
            '#'  => out.push_str(r"\#"),
            '_'  => out.push_str(r"\_"),
            '{'  => out.push_str(r"\{"),
            '}'  => out.push_str(r"\}"),
            '~'  => out.push_str(r"\char126 "),
            '^'  => out.push_str(r"\char94 "),
            '\\' => out.push_str(r"\char92 "),
            c    => out.push(c),
        }
    }
    out
}

struct Context {
    dpi: u32,
    base_dir: Option<PathBuf>,
    images_dir: Option<PathBuf>,
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
    /// Set after a table ends (captions mode only); cleared by the next paragraph start.
    after_table: bool,
    /// True while buffering a potential table-caption paragraph.
    caption_para: bool,
    /// Byte offset in `out` where the current caption paragraph started.
    caption_start: usize,
    /// Raw text collected in the current caption paragraph (for prefix detection).
    caption_text: String,
}

impl Context {
    fn new(dpi: u32, base_dir: Option<&Path>, images_dir: Option<&Path>, nonum: bool, toc_depth: u32, captions: bool, footnotes: HashMap<String, String>) -> Self {
        Self {
            dpi,
            base_dir: base_dir.map(|p| p.to_path_buf()),
            images_dir: images_dir.map(|p| p.to_path_buf()),
            nonum,
            toc_depth,
            captions,
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
            after_table: false,
            caption_para: false,
            caption_start: 0,
            caption_text: String::new(),
        }
    }

    /// Resolves an image path: tries images_dir first, then base_dir.
    /// Returns an absolute path when possible, otherwise the path as-is.
    fn resolve_image_path(&self, path: &str) -> PathBuf {
        let p = Path::new(path);
        if p.is_absolute() {
            return p.to_path_buf();
        }
        // Prefer images_dir / path when it exists.
        if let Some(img_dir) = &self.images_dir {
            let candidate = img_dir.join(p);
            if candidate.exists() {
                return std::fs::canonicalize(&candidate).unwrap_or(candidate);
            }
        }
        if let Some(base) = &self.base_dir {
            let joined = base.join(p);
            return std::fs::canonicalize(&joined).unwrap_or(joined);
        }
        p.to_path_buf()
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
            Event::End(tag)   => self.end_tag(tag, out),
            Event::Text(t)    => {
                if self.in_image {
                    self.image_alt.push_str(&t);
                } else if self.in_code_block {
                    out.push_str(&t);
                } else {
                    if self.caption_para {
                        self.caption_text.push_str(&t);
                    }
                    let escaped = tex_escape(&t);
                    let processed = typo::apply(&escaped);
                    out.push_str(&processed);
                }
            }
            Event::Code(t) => {
                out.push_str(&format!("{{\\tt {}}}", tex_escape(&t)));
            }
            Event::FootnoteReference(label) => {
                let body = self.footnotes.get(label.as_ref()).cloned()
                    .unwrap_or_else(|| format!("?{label}"));
                out.push_str(&format!("\\fnote{{{body}}}"));
            }
            Event::TaskListMarker(checked) => {
                out.push_str(if checked { "[{\\tt x}]\\ " } else { "[\\ ]\\ " });
            }
            Event::SoftBreak => out.push('\n'),
            Event::HardBreak => out.push_str("\\hfil\\break\n"),
            Event::Rule      => out.push_str("\\noindent\\hrule\n\n"),
            Event::Html(_) | Event::InlineHtml(_) => {} // discarded
            _ => {}
        }
    }

    fn start_tag(&mut self, tag: Tag, out: &mut String) {
        match tag {
            Tag::Heading { level, .. } => {
                let cmd = heading_cmd(level);
                let depth = heading_depth(level);
                out.push('\n');
                if self.nonum { out.push_str("\\nonum "); }
                if depth > self.toc_depth { out.push_str("\\notoc "); }
                out.push_str(&format!("{cmd} "));
            }
            Tag::Paragraph => {
                if self.captions && self.after_table {
                    self.caption_para = true;
                    self.caption_start = out.len();
                    self.caption_text.clear();
                }
                self.after_table = false;
            }
            Tag::Strong => out.push_str("{\\bf "),
            Tag::Emphasis => out.push_str("{\\it "),
            Tag::Strikethrough => out.push_str("\\strike{"),
            Tag::CodeBlock(_) => {
                self.in_code_block = true;
                out.push_str("\\begtt\n");
            }
            Tag::List(None)    => {
                self.list_depth += 1;
                out.push_str("\\begitems\n");
            }
            Tag::List(Some(_)) => {
                self.list_depth += 1;
                out.push_str("\\begitems \\style n\n");
            }
            Tag::Item => out.push_str("* "),
            Tag::BlockQuote(_) => out.push_str("\\begcitation\n"),
            Tag::Link { dest_url, title: _, id: _, .. } => {
                out.push_str(&format!("\\ulink[{}]{{", dest_url));
            }
            Tag::Image { dest_url, .. } => {
                self.in_image = true;
                self.image_alt.clear();
                let resolved = self.resolve_image_path(&dest_url);
                let width = measure_image(&resolved, self.dpi);
                out.push_str(&format!("\\picw={width} \\inspic {}\n", resolved.display()));
            }
            Tag::Table(alignments) => {
                self.col_alignments = alignments;
                self.col_index = 0;
                self.row_count = 0;
                let spec: String = self.col_alignments.iter().map(alignment_char).collect();
                // \par\medskip ensures vertical spacing before the table.
                // \noindent\hfil...\hfil centres narrow tables; for wide tables the glue
                // simply compresses to zero so there is no overflow.
                // \noalign{\hrule\smallskip} adds the top rule (three-line / booktabs style).
                out.push_str(&format!("\\par\\medskip\\noindent\\hfil\n\\table{{{spec}}}{{\\noalign{{\\hrule\\smallskip}}\n"));
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
            }
            Tag::FootnoteDefinition(_) => {
                self.in_footnote_def = true;
            }
            _ => {}
        }
    }

    fn end_tag(&mut self, tag: TagEnd, out: &mut String) {
        match tag {
            TagEnd::Heading(_) => out.push('\n'),
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
            TagEnd::Strong | TagEnd::Emphasis | TagEnd::Strikethrough => out.push('}'),
            TagEnd::CodeBlock => {
                self.in_code_block = false;
                out.push_str("\\endtt\n\n");
            }
            TagEnd::List(_) => {
                self.list_depth -= 1;
                out.push_str("\\enditems\n\n");
            }
            TagEnd::Item => out.push('\n'),
            TagEnd::BlockQuote(_) => out.push_str("\\endcitation\n\n"),
            TagEnd::Link => out.push('}'),
            TagEnd::Image => {
                self.in_image = false;
                if self.captions && !self.image_alt.is_empty() {
                    let alt = std::mem::take(&mut self.image_alt);
                    out.push_str(&format!("\\caption/f {alt}\n"));
                } else {
                    self.image_alt.clear();
                }
            }
            TagEnd::TableHead => {
                self.in_table_head = false;
                out.push_str(" \\crli\n"); // horizontal rule below the header row
            }
            TagEnd::TableRow => {
                out.push_str(" \\cr\n");
            }
            TagEnd::TableCell => {
                self.col_index += 1;
            }
            TagEnd::Table => {
                // \noalign{\smallskip\hrule} adds the bottom rule; closing \hfil\par\medskip
                // finishes the centred paragraph and adds trailing vertical space.
                out.push_str("\\noalign{\\smallskip\\hrule}\n}\\hfil\\par\\medskip\n\n");
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
        _                => "\\seccc",
    }
}

fn heading_depth(level: HeadingLevel) -> u32 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        _                => 4,
    }
}

fn alignment_char(a: &Alignment) -> char {
    match a {
        Alignment::Left   => 'l',
        Alignment::Center => 'c',
        Alignment::Right  => 'r',
        Alignment::None   => 'l',
    }
}

/// Returns the caption body if `text` starts with Pandoc-style caption prefix (`: `),
/// otherwise `None`.
fn strip_caption_prefix(text: &str) -> Option<&str> {
    text.strip_prefix(':').map(|rest| rest.trim())
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
            eprintln!("Warning: cannot measure image '{}', using \\hsize", path.display());
            "\\hsize".to_owned()
        }
    }
}
