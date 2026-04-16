mod error;
mod metadata;
mod renderer;
mod styles;
mod typo;

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

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
    let base_dir: Option<PathBuf> = args.input.as_ref().and_then(|p| {
        let dir: &Path = if p.is_dir() {
            p.as_path()
        } else {
            match p.parent() {
                Some(parent) if !parent.as_os_str().is_empty() => parent,
                // Input is a bare filename like "doc.md" — parent is "".
                // Interpret as current working directory.
                _ => Path::new("."),
            }
        };
        fs::canonicalize(dir).ok()
    });

    // Determine output mode based on `-o` file extension.
    let output_mode = match &args.output {
        None => OutputMode::Stdout,
        Some(path) => {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_lowercase());
            if ext.as_deref() == Some("pdf") {
                OutputMode::Pdf(path.clone())
            } else {
                OutputMode::Tex(path.clone())
            }
        }
    };

    match output_mode {
        OutputMode::Stdout => {
            let tex = renderer::render(
                &markdown,
                metadata.as_ref(),
                &hyphenation,
                args.dpi,
                args.style.as_deref(),
                base_dir.as_deref(),
                None,
            )?;
            io::stdout().write_all(tex.as_bytes())?;
        }
        OutputMode::Tex(path) => {
            let output_dir = resolve_output_dir(&path)?;
            let tex = renderer::render(
                &markdown,
                metadata.as_ref(),
                &hyphenation,
                args.dpi,
                args.style.as_deref(),
                base_dir.as_deref(),
                Some(&output_dir),
            )?;
            fs::write(&path, tex)?;
        }
        OutputMode::Pdf(path) => {
            let tmp = tempfile::Builder::new().prefix("md2optex-").tempdir()?;
            let tmp_dir = fs::canonicalize(tmp.path())?;
            let tex = renderer::render(
                &markdown,
                metadata.as_ref(),
                &hyphenation,
                args.dpi,
                args.style.as_deref(),
                base_dir.as_deref(),
                Some(&tmp_dir),
            )?;
            let tex_path = tmp_dir.join("doc.tex");
            fs::write(&tex_path, tex)?;
            run_optex(&tmp_dir, "doc.tex")?;
            // Ensure parent of output PDF exists.
            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
            {
                fs::create_dir_all(parent)?;
            }
            fs::copy(tmp_dir.join("doc.pdf"), &path)?;
        }
    }

    Ok(())
}

enum OutputMode {
    Stdout,
    Tex(PathBuf),
    Pdf(PathBuf),
}

/// Resolves the directory where the output TeX file will live. Creates the
/// parent directory if missing so that `canonicalize` succeeds.
fn resolve_output_dir(output: &Path) -> Result<PathBuf, Error> {
    let parent = output.parent().filter(|p| !p.as_os_str().is_empty());
    let dir: PathBuf = match parent {
        Some(p) => p.to_path_buf(),
        None => PathBuf::from("."),
    };
    fs::create_dir_all(&dir)?;
    Ok(fs::canonicalize(&dir)?)
}

/// Runs `optex <tex_name>` in `work_dir`. Runs twice so the TOC / cross-refs
/// resolve on the second pass.
fn run_optex(work_dir: &Path, tex_name: &str) -> Result<(), Error> {
    for _ in 0..2 {
        let status = Command::new("optex")
            .arg(tex_name)
            .current_dir(work_dir)
            .status()
            .map_err(|e| {
                if e.kind() == io::ErrorKind::NotFound {
                    Error::OptexNotFound
                } else {
                    Error::Io(e)
                }
            })?;
        if !status.success() {
            return Err(Error::OptexFailed(status.code().unwrap_or(-1)));
        }
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
            let markdown = load_chapters(path, metadata.as_ref())?;
            Ok((markdown, metadata))
        }
        Some(path) => {
            let markdown = fs::read_to_string(path)?;
            Ok((markdown, None))
        }
    }
}

fn load_chapters(dir: &Path, metadata: Option<&Metadata>) -> Result<String, Error> {
    // Explicit list from `[chapters] files = [...]` takes priority.
    if let Some(files) = metadata
        .and_then(|m| m.chapters.as_ref())
        .and_then(|c| c.files.as_ref())
        && !files.is_empty()
    {
        let mut out = String::new();
        for rel in files {
            let path = if rel.is_absolute() {
                rel.clone()
            } else {
                dir.join(rel)
            };
            out.push_str(&fs::read_to_string(&path)?);
            out.push('\n');
        }
        return Ok(out);
    }

    // Fallback: read `chapters/*.md` (or legacy `kapitoly/*.md`) alphabetically.
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

    let content = fs::read_to_string(&path).map_err(|e| Error::HyphenationDict(path.clone(), e))?;

    let words = content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_owned)
        .collect();

    Ok(words)
}
