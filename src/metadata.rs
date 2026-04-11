use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::Error;

#[derive(Debug, Deserialize, Default)]
pub struct Metadata {
    pub book: Option<Book>,
    #[serde(rename = "typesetting")]
    pub typesetting: Option<Typesetting>,
    pub paths: Option<Paths>,
    pub style: Option<Style>,
}

/// Controls whether and where to place the table of contents.
/// In TOML: `toc = true/false` or `toc = "front"/"back"`.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TocValue {
    Bool(bool),
    Position(String),
}

#[derive(Debug, Deserialize)]
pub struct Book {
    pub title: Option<String>,
    pub author: Option<String>,
    pub year: Option<u32>,
    pub isbn: Option<String>,
    pub toc: Option<TocValue>,
    pub copyright: Option<String>,
    /// Insert a half-title page (název pouze, verso prázdné) before the full
    /// title page. Defaults to `true` for the `book` style, `false` otherwise.
    pub half_title: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct Typesetting {
    pub paper: Option<String>,
    pub font: Option<String>,
    pub base_size: Option<String>,
    pub paragraph: Option<String>,
    pub margin_left: Option<u32>,
    pub margin_right: Option<u32>,
    pub margin_top: Option<u32>,
    pub margin_bottom: Option<u32>,
    pub header: Option<String>,
    pub footer: Option<String>,
    /// Maximum heading depth included in the table of contents (1 = chapters only).
    pub toc_depth: Option<u32>,
    /// Enable drop-cap (enlarged initial) on the first paragraph of each
    /// chapter. Defaults to `true` for `book`, `false` otherwise.
    pub drop_cap: Option<bool>,
    /// Page canon: when set to `"tschichold"`, margins are derived from the
    /// paper size as asymmetric proportions (inner smaller than outer, top
    /// smaller than bottom). Overrides `margin_*` fields.
    pub canon: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Paths {
    pub images: Option<PathBuf>,
    pub hyphenation: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub struct Style {
    pub name: Option<String>,
}

impl Metadata {
    pub fn load(path: &Path) -> Result<Self, Error> {
        let content = fs::read_to_string(path)?;
        let meta: Metadata = toml::from_str(&content)?;
        Ok(meta)
    }

    /// Parses a flat YAML front matter block (`key: value` pairs) into Metadata.
    /// Supports: title, author, date/year, isbn, style.
    pub fn from_yaml_str(yaml: &str) -> Self {
        let mut title = None;
        let mut author = None;
        let mut year: Option<u32> = None;
        let mut isbn = None;
        let mut style_name = None;
        let mut drop_cap: Option<bool> = None;
        let mut canon: Option<String> = None;
        let mut half_title: Option<bool> = None;

        for raw_line in yaml.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, val)) = line.split_once(':') {
                let key = key.trim();
                let val = val.trim().trim_matches('"').trim_matches('\'');
                match key {
                    "title" => title = Some(val.to_string()),
                    "author" => author = Some(val.to_string()),
                    "year" | "date" => {
                        year = val.split('-').next().and_then(|y| y.trim().parse().ok())
                    }
                    "isbn" => isbn = Some(val.to_string()),
                    "style" => style_name = Some(val.to_string()),
                    "drop_cap" => drop_cap = parse_bool(val),
                    "canon" => canon = Some(val.to_string()),
                    "half_title" => half_title = parse_bool(val),
                    _ => {}
                }
            }
        }

        let has_book = title.is_some()
            || author.is_some()
            || year.is_some()
            || isbn.is_some()
            || half_title.is_some();
        let typesetting = if drop_cap.is_some() || canon.is_some() {
            Some(Typesetting {
                paper: None,
                font: None,
                base_size: None,
                paragraph: None,
                margin_left: None,
                margin_right: None,
                margin_top: None,
                margin_bottom: None,
                header: None,
                footer: None,
                toc_depth: None,
                drop_cap,
                canon,
            })
        } else {
            None
        };
        Metadata {
            book: if has_book {
                Some(Book {
                    title,
                    author,
                    year,
                    isbn,
                    toc: None,
                    copyright: None,
                    half_title,
                })
            } else {
                None
            },
            typesetting,
            paths: None,
            style: style_name.map(|name| Style { name: Some(name) }),
        }
    }
}

fn parse_bool(val: &str) -> Option<bool> {
    match val.to_ascii_lowercase().as_str() {
        "true" | "yes" | "1" | "on" => Some(true),
        "false" | "no" | "0" | "off" => Some(false),
        _ => None,
    }
}
