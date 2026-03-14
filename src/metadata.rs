use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::Error;

#[derive(Debug, Deserialize, Default)]
pub struct Metadata {
    #[serde(rename = "kniha")]
    pub book: Option<Book>,
    #[serde(rename = "sazba")]
    pub typesetting: Option<Typesetting>,
    #[serde(rename = "cesty")]
    pub paths: Option<Paths>,
    #[serde(rename = "styl")]
    pub style: Option<Style>,
}

#[derive(Debug, Deserialize)]
pub struct Book {
    #[serde(rename = "nazev")]
    pub title: Option<String>,
    #[serde(rename = "autor")]
    pub author: Option<String>,
    pub rok: Option<u32>,
    pub isbn: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Typesetting {
    #[serde(rename = "papir")]
    pub paper: Option<String>,
    pub font: Option<String>,
    #[serde(rename = "zakladni_vel")]
    pub base_size: Option<String>,
    #[serde(rename = "odstavec")]
    pub paragraph: Option<String>,
    #[serde(rename = "okraj_vlevo")]
    pub margin_left: Option<u32>,
    #[serde(rename = "okraj_vpravo")]
    pub margin_right: Option<u32>,
    #[serde(rename = "okraj_nahore")]
    pub margin_top: Option<u32>,
    #[serde(rename = "okraj_dole")]
    pub margin_bottom: Option<u32>,
    #[serde(rename = "zahlaví")]
    pub header: Option<String>,
    #[serde(rename = "zapati")]
    pub footer: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Paths {
    #[serde(rename = "obrazky")]
    pub images: Option<PathBuf>,
    pub hyphenation: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub struct Style {
    #[serde(rename = "styl")]
    pub name: Option<String>,
}

impl Metadata {
    pub fn load(path: &Path) -> Result<Self, Error> {
        let content = fs::read_to_string(path)?;
        let meta: Metadata = toml::from_str(&content)?;
        Ok(meta)
    }
}
