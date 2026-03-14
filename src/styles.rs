use std::fs;
use std::path::Path;

const MINIMAL: &str = include_str!("styles/minimal.tex");
const BOOK: &str = include_str!("styles/book.tex");
const ACADEMIC: &str = include_str!("styles/academic.tex");
const MANUAL: &str = include_str!("styles/manual.tex");

#[allow(dead_code)]
pub const BUILTIN_NAMES: &[&str] = &["minimal", "book", "academic", "manual"];

/// Returns the TeX content for `name`, or `None` if the style is not found.
///
/// Lookup order:
/// 1. `<base_dir>/styles/<name>.tex`  — project-local override
/// 2. `~/.config/md2optex/styles/<name>.tex`  — user-level override
/// 3. Built-in styles embedded in the binary
pub fn resolve(name: &str, base_dir: Option<&Path>) -> Option<String> {
    // 1. Project-local styles directory
    if let Some(dir) = base_dir {
        let local = dir.join("styles").join(format!("{name}.tex"));
        if local.exists()
            && let Ok(content) = fs::read_to_string(&local)
        {
            return Some(content);
        }
    }

    // 2. User config directory (~/.config/md2optex/styles/)
    if let Some(config) = user_config_dir() {
        let user = config.join("md2optex").join("styles").join(format!("{name}.tex"));
        if user.exists()
            && let Ok(content) = fs::read_to_string(&user)
        {
            return Some(content);
        }
    }

    // 3. Built-in
    builtin(name).map(str::to_owned)
}

/// Returns the content of a built-in style by name, or `None`.
pub fn builtin(name: &str) -> Option<&'static str> {
    match name {
        "minimal" => Some(MINIMAL),
        "book" => Some(BOOK),
        "academic" => Some(ACADEMIC),
        "manual" => Some(MANUAL),
        _ => None,
    }
}

fn user_config_dir() -> Option<std::path::PathBuf> {
    // Honour $XDG_CONFIG_HOME if set, otherwise fall back to ~/.config
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return Some(std::path::PathBuf::from(xdg));
    }
    std::env::var("HOME")
        .ok()
        .map(|h| std::path::PathBuf::from(h).join(".config"))
}
