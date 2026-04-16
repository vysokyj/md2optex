//! Metadata schema aligned with the mdf `metadata.toml` specification
//! (`../../mdf/docs/metadata-toml-spec.md`).
//!
//! Layout:
//!   * flat top-level fields for document identity (`title`, `author`, `lang`,
//!     `style`) and bibliographic metadata (`year`, `isbn`, `copyright`,
//!     `subtitle`, `translator`, `publisher`, `edition`).
//!   * `[chapters]` — explicit list of chapter files.
//!   * `[options]` — engine options (kebab-case keys).
//!   * `[page]` — page size, margins, Tschichold canon.
//!   * `[paths]` — project-relative paths (images, styles, hyphenation).
//!
//! Fields not listed in the mdf spec but supported by md2optex (TeX-specific
//! running headers, the book-style half-title, …) live inside `[options]` and
//! are annotated below as *md2optex extensions*. Other tools consuming the
//! same TOML should ignore them.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::Error;

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Metadata {
    // Document identity
    pub title: Option<String>,
    pub author: Option<String>,
    pub lang: Option<String>,
    pub style: Option<String>,

    // Bibliographic (optional)
    pub year: Option<u32>,
    pub isbn: Option<String>,
    pub copyright: Option<String>,
    pub subtitle: Option<String>,
    pub translator: Option<String>,
    pub publisher: Option<String>,
    pub edition: Option<String>,

    pub chapters: Option<Chapters>,
    pub options: Option<Options>,
    pub page: Option<Page>,
    pub paths: Option<Paths>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Chapters {
    pub files: Option<Vec<PathBuf>>,
}

/// Controls whether and where to place the table of contents.
/// In TOML: `toc = true/false`, `toc = "off"`, `toc = "front"`, or `toc = "back"`.
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum TocValue {
    Bool(bool),
    Position(String),
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct Options {
    pub toc: Option<TocValue>,
    pub toc_depth: Option<u32>,
    pub toc_title: Option<String>,
    pub drop_cap: Option<bool>,
    pub font: Option<String>,
    pub heading_font: Option<String>,
    pub mono_font: Option<String>,
    pub widows: Option<u32>,
    pub orphans: Option<u32>,

    // md2optex extensions (not part of the mdf spec; mdf handles these via CSS).
    /// Base font size, e.g. `"11pt"`. Drives OpTeX `\typosize`.
    pub base_size: Option<String>,
    /// `"indent"` (default) or `"noindent"` — suppresses paragraph indentation.
    pub paragraph: Option<String>,
    /// OpTeX `\headline` template: `"left & center & right"`.
    pub header: Option<String>,
    /// OpTeX `\footline` template: `"left & center & right"`.
    pub footer: Option<String>,
    /// Book-style half-title page. Defaults to `true` when `style = "book"`.
    pub half_title: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct Page {
    /// `"A4"` | `"A5"` | `"B5"` | `"Letter"` (case-insensitive); explicit
    /// dimensions like `"210mm 297mm"` are accepted but currently map to the
    /// closest OpTeX paper name.
    pub size: Option<String>,
    /// CSS margin shorthand: `"25mm"`, `"30mm 25mm"`, `"30mm 20mm 40mm"`,
    /// or `"30mm 20mm 40mm 10mm"`.
    pub margin: Option<String>,
    pub margin_top: Option<String>,
    pub margin_bottom: Option<String>,
    pub margin_left: Option<String>,
    pub margin_right: Option<String>,
    /// `"tschichold"` derives asymmetric margins from the paper size.
    pub canon: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Paths {
    pub images: Option<PathBuf>,
    /// Declared for mdf-spec compatibility; md2optex currently resolves style
    /// overrides via the lookup chain in `styles.rs` instead.
    #[allow(dead_code)]
    pub styles: Option<PathBuf>,
    pub hyphenation: Option<PathBuf>,
}

impl Metadata {
    pub fn load(path: &Path) -> Result<Self, Error> {
        let content = fs::read_to_string(path)?;
        let meta: Metadata = toml::from_str(&content)?;
        Ok(meta)
    }

    /// Parses a flat YAML front matter block (`key: value` pairs) into
    /// Metadata. Covers the fields typically set inline in a single-file
    /// document: identity, bibliographic, `drop-cap`, `half-title`, `canon`.
    pub fn from_yaml_str(yaml: &str) -> Self {
        let mut meta = Metadata::default();
        let mut opts = Options::default();
        let mut page = Page::default();
        let mut opts_set = false;
        let mut page_set = false;

        for raw_line in yaml.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, val)) = line.split_once(':') else {
                continue;
            };
            let key = key.trim();
            let val = val.trim().trim_matches('"').trim_matches('\'');
            match key {
                "title" => meta.title = Some(val.to_string()),
                "author" => meta.author = Some(val.to_string()),
                "lang" => meta.lang = Some(val.to_string()),
                "style" => meta.style = Some(val.to_string()),
                "year" | "date" => {
                    meta.year = val.split('-').next().and_then(|y| y.trim().parse().ok());
                }
                "isbn" => meta.isbn = Some(val.to_string()),
                "copyright" => meta.copyright = Some(val.to_string()),
                "subtitle" => meta.subtitle = Some(val.to_string()),
                "translator" => meta.translator = Some(val.to_string()),
                "publisher" => meta.publisher = Some(val.to_string()),
                "edition" => meta.edition = Some(val.to_string()),
                "drop-cap" | "drop_cap" => {
                    opts.drop_cap = parse_bool(val);
                    opts_set = true;
                }
                "half-title" | "half_title" => {
                    opts.half_title = parse_bool(val);
                    opts_set = true;
                }
                "canon" => {
                    page.canon = Some(val.to_string());
                    page_set = true;
                }
                _ => {}
            }
        }

        if opts_set {
            meta.options = Some(opts);
        }
        if page_set {
            meta.page = Some(page);
        }
        meta
    }
}

fn parse_bool(val: &str) -> Option<bool> {
    match val.to_ascii_lowercase().as_str() {
        "true" | "yes" | "1" | "on" => Some(true),
        "false" | "no" | "0" | "off" => Some(false),
        _ => None,
    }
}

/// Normalises a page-size string to the lowercase paper name used by OpTeX
/// (`a4`, `a5`, `b5`, `letter`). Unknown sizes fall back to `a4` and trigger
/// no warning here — callers decide.
pub fn normalize_paper(size: &str) -> String {
    let lc = size.trim().to_ascii_lowercase();
    match lc.as_str() {
        "a4" | "a5" | "b5" | "letter" => lc,
        _ => "a4".to_string(),
    }
}

/// Parses a single CSS length into millimetres. Supports `mm`, `cm`, `in`,
/// `pt`. Bare numbers are interpreted as millimetres for convenience.
pub fn parse_length_mm(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let idx = s.find(|c: char| c.is_alphabetic()).unwrap_or(s.len());
    let (num, unit) = (s[..idx].trim(), s[idx..].trim().to_ascii_lowercase());
    let v: f64 = num.parse().ok()?;
    match unit.as_str() {
        "mm" | "" => Some(v),
        "cm" => Some(v * 10.0),
        "in" => Some(v * 25.4),
        "pt" => Some(v * 25.4 / 72.0),
        _ => None,
    }
}

/// Parses a CSS margin shorthand into `(top, right, bottom, left)` in mm.
/// Accepts 1–4 whitespace-separated lengths.
pub fn parse_margin_shorthand(s: &str) -> Option<(f64, f64, f64, f64)> {
    let parts: Vec<f64> = s.split_whitespace().filter_map(parse_length_mm).collect();
    match parts.len() {
        1 => Some((parts[0], parts[0], parts[0], parts[0])),
        2 => Some((parts[0], parts[1], parts[0], parts[1])),
        3 => Some((parts[0], parts[1], parts[2], parts[1])),
        4 => Some((parts[0], parts[1], parts[2], parts[3])),
        _ => None,
    }
}
