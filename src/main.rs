mod error;
mod metadata;
mod renderer;
mod styles;
mod typo;

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use clap::Parser;

use error::Error;
use metadata::Metadata;

#[derive(Parser)]
#[command(version, about = "Markdown to OpTeX converter")]
struct Args {
    /// Input Markdown file or book directory (default: stdin)
    input: Option<PathBuf>,

    /// Output TeX file (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Hyphenation dictionary file (one word per line: syl-la-ble)
    #[arg(long)]
    hyphenation_dict: Option<PathBuf>,

    /// Image resolution in DPI used to compute physical dimensions
    #[arg(long, default_value_t = 96)]
    dpi: u32,

    /// Built-in or custom style name (minimal | kniha | odborny | manual, or a path)
    #[arg(long)]
    style: Option<String>,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let args = Args::parse();

    let (markdown, metadata) = load_input(&args)?;
    let hyphenation = load_hyphenation(&args, metadata.as_ref())?;
    let tex = renderer::render(
        &markdown,
        metadata.as_ref(),
        &hyphenation,
        args.dpi,
        args.style.as_deref(),
    )?;

    match &args.output {
        Some(path) => fs::write(path, tex)?,
        None => io::stdout().write_all(tex.as_bytes())?,
    }

    Ok(())
}

fn load_input(args: &Args) -> Result<(String, Option<Metadata>), Error> {
    match &args.input {
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Ok((buf, None))
        }
        Some(path) if path.is_dir() => {
            let meta_path = path.join("metadata.toml");
            let metadata = if meta_path.exists() {
                Some(Metadata::load(&meta_path)?)
            } else {
                None
            };
            let markdown = load_chapters(path)?;
            Ok((markdown, metadata))
        }
        Some(path) => {
            let markdown = fs::read_to_string(path)?;
            Ok((markdown, None))
        }
    }
}

fn load_chapters(dir: &Path) -> Result<String, Error> {
    // Accept both "chapters" (preferred) and "kapitoly" (legacy Czech name)
    let chapters_dir = ["chapters", "kapitoly"]
        .iter()
        .map(|name| dir.join(name))
        .find(|p| p.exists())
        .ok_or_else(|| Error::MissingChaptersDir(dir.join("chapters")))?;


    let mut files: Vec<PathBuf> = fs::read_dir(&chapters_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .collect();

    files.sort();

    let mut out = String::new();
    for f in files {
        out.push_str(&fs::read_to_string(&f)?);
        out.push('\n');
    }
    Ok(out)
}

fn load_hyphenation(args: &Args, metadata: Option<&Metadata>) -> Result<Vec<String>, Error> {
    let path = args
        .hyphenation_dict
        .clone()
        .or_else(|| metadata.and_then(|m| m.paths.as_ref()?.hyphenation.clone()));

    let Some(path) = path else {
        return Ok(vec![]);
    };

    let content = fs::read_to_string(&path)
        .map_err(|e| Error::HyphenationDict(path.clone(), e))?;

    let words = content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_owned)
        .collect();

    Ok(words)
}
