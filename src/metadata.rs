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

        for raw_line in yaml.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, val)) = line.split_once(':') {
                let key = key.trim();
                let val = val.trim().trim_matches('"').trim_matches('\'');
                match key {
                    "title"         => title      = Some(val.to_string()),
                    "author"        => author     = Some(val.to_string()),
                    "year" | "date" => year       = val.split('-').next()
                                            .and_then(|y| y.trim().parse().ok()),
                    "isbn"          => isbn       = Some(val.to_string()),
                    "style"         => style_name = Some(val.to_string()),
                    _               => {}
                }
            }
        }

        let has_book = title.is_some() || author.is_some()
            || year.is_some() || isbn.is_some();
        Metadata {
            book: if has_book {
                Some(Book { title, author, year, isbn, toc: None, copyright: None })
            } else {
                None
            },
            typesetting: None,
            paths: None,
            style: style_name.map(|name| Style { name: Some(name) }),
        }
    }
}
