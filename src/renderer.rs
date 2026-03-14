use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, HeadingLevel, Alignment};

use crate::error::Error;
use crate::metadata::Metadata;
use crate::typo;

pub fn render(
    markdown: &str,
    metadata: Option<&Metadata>,
    hyphenation: &[String],
    dpi: u32,
) -> Result<String, Error> {
    let mut out = String::new();

    // Preamble
    out.push_str(&build_preamble(metadata, hyphenation)?);

    // Document body
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS;

    let parser = Parser::new_ext(markdown, opts);
    let mut ctx = Context::new(dpi);

    for event in parser {
        ctx.handle_event(event, &mut out);
    }

    out.push_str("\n\\bye\n");
    Ok(out)
}

fn build_preamble(metadata: Option<&Metadata>, hyphenation: &[String]) -> Result<String, Error> {
    let mut s = String::new();

    // OpTeX is a LuaTeX format — no \input optex needed, it is pre-loaded by the engine.
    s.push_str("\\fontfam[LM]\n"); // Latin Modern Unicode — required for Czech characters
    s.push_str("\\uselanguage{czech}\n");
    // \begcitation/\endcitation are not part of OpTeX; define them here.
    s.push_str("\\def\\begcitation{\\par\\medskip\\leftskip=2em\\rightskip=2em\\noindent}\n");
    s.push_str("\\def\\endcitation{\\par\\leftskip=0em\\rightskip=0em\\medskip}\n");

    if let Some(meta) = metadata {
        if let Some(ts) = &meta.typesetting {
            if let Some(font) = &ts.font {
                s.push_str(&format!("\\fontfam[{}]\n", font));
            }
            if let Some(size) = &ts.base_size {
                // e.g. "11pt" → \typosize[11/13]
                let pt: u32 = size.trim_end_matches("pt").parse().unwrap_or(10);
                let leading = pt * 13 / 10;
                s.push_str(&format!("\\typosize[{pt}/{leading}]\n"));
            }
            if let Some(left) = ts.margin_left {
                s.push_str(&format!(
                    "\\margins/1 a4 ({left}mm,{},{},{}mm)\n",
                    ts.margin_right.unwrap_or(25),
                    ts.margin_top.unwrap_or(30),
                    ts.margin_bottom.unwrap_or(30),
                ));
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
        if let Some(book) = &meta.book {
            if let Some(title) = &book.title {
                s.push_str(&format!("\\tit {title}\n"));
            }
            if let Some(author) = &book.author {
                s.push_str(&format!("\\author {author}\n"));
            }
            if book.title.is_some() || book.author.is_some() {
                s.push_str("\\maketitle\n");
            }
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
    list_depth: u32,
    in_code_block: bool,
    in_image: bool,
    in_table_head: bool,
    col_alignments: Vec<Alignment>,
    col_index: usize,
    row_count: usize,
}

impl Context {
    fn new(dpi: u32) -> Self {
        Self {
            dpi,
            list_depth: 0,
            in_code_block: false,
            in_image: false,
            in_table_head: false,
            col_alignments: vec![],
            col_index: 0,
            row_count: 0,
        }
    }

    fn handle_event(&mut self, event: Event, out: &mut String) {
        match event {
            Event::Start(tag) => self.start_tag(tag, out),
            Event::End(tag)   => self.end_tag(tag, out),
            Event::Text(t)    => {
                if self.in_image {
                    // alt text is discarded — OpTeX does not use it
                } else if self.in_code_block {
                    out.push_str(&t);
                } else {
                    let escaped = tex_escape(&t);
                    let processed = typo::apply(&escaped);
                    out.push_str(&processed);
                }
            }
            Event::Code(t) => {
                out.push_str(&format!("{{\\tt {}}}", tex_escape(&t)));
            }
            Event::SoftBreak => out.push('\n'),
            Event::HardBreak => out.push_str("\\\\\n"),
            Event::Rule      => out.push_str("\\noindent\\hrule\n\n"),
            Event::Html(_) | Event::InlineHtml(_) => {} // discarded
            _ => {}
        }
    }

    fn start_tag(&mut self, tag: Tag, out: &mut String) {
        match tag {
            Tag::Heading { level, .. } => {
                let cmd = heading_cmd(level);
                out.push_str(&format!("\n{cmd} "));
            }
            Tag::Paragraph => {}
            Tag::Strong => out.push_str("{\\bf "),
            Tag::Emphasis => out.push_str("{\\it "),
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
                let width = measure_image(&dest_url, self.dpi);
                out.push_str(&format!("\\picw={width} \\inspic {dest_url}\n"));
            }
            Tag::Table(alignments) => {
                self.col_alignments = alignments;
                self.col_index = 0;
                self.row_count = 0;
                let spec: String = self.col_alignments.iter().map(alignment_char).collect();
                out.push_str(&format!("\\table{{{spec}}}{{\n"));
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
            _ => {}
        }
    }

    fn end_tag(&mut self, tag: TagEnd, out: &mut String) {
        match tag {
            TagEnd::Heading(_) => out.push('\n'),
            TagEnd::Paragraph => out.push_str("\n\n"),
            TagEnd::Strong | TagEnd::Emphasis => out.push('}'),
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
                out.push_str("}\n\n");
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

fn alignment_char(a: &Alignment) -> char {
    match a {
        Alignment::Left   => 'l',
        Alignment::Center => 'c',
        Alignment::Right  => 'r',
        Alignment::None   => 'l',
    }
}

fn measure_image(path: &str, dpi: u32) -> String {
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
            eprintln!("Warning: cannot measure image '{path}', using \\hsize");
            "\\hsize".to_owned()
        }
    }
}
