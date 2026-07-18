use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::RwLock;

use super::registry::ThemeRegistry;
use super::resolver::ResolvedThemeResult;
use super::resolver::ThemeDiagnostic;
use super::resolver::resolve_theme;
use super::schema::ThemeFile;
use super::tokens::ResolvedTheme;

const FALLBACK_THEME_ID: &str = "codex-dark";

static ACTIVE_THEME: OnceLock<RwLock<Arc<ResolvedTheme>>> = OnceLock::new();

pub(crate) fn current() -> Arc<ResolvedTheme> {
    ACTIVE_THEME
        .get_or_init(|| RwLock::new(Arc::new(ResolvedTheme::fallback())))
        .read()
        .expect("theme lock poisoned")
        .clone()
}

pub(crate) fn initialize(
    codex_home: &Path,
    selected_id: Option<&str>,
) -> Result<(), ThemeDiagnostic> {
    let id = selected_id.unwrap_or(FALLBACK_THEME_ID);
    let resolved =
        load_resolved(codex_home, id).or_else(|_| load_resolved(codex_home, FALLBACK_THEME_ID))?;
    set_current(resolved.theme);
    Ok(())
}

pub(crate) fn apply(codex_home: &Path, id: &str) -> Result<ResolvedThemeResult, ThemeDiagnostic> {
    let resolved = load_resolved(codex_home, id)?;
    set_current(resolved.theme.clone());
    Ok(resolved)
}

pub(crate) fn preview(codex_home: &Path, id: &str) -> Result<ResolvedThemeResult, ThemeDiagnostic> {
    apply(codex_home, id)
}

pub(crate) fn validate(
    codex_home: &Path,
    id: &str,
) -> Result<ResolvedThemeResult, ThemeDiagnostic> {
    load_resolved(codex_home, id)
}

pub(crate) fn registry(codex_home: &Path) -> ThemeRegistry {
    ThemeRegistry::load(
        codex_home, /*project_root*/ None, /*trusted_project*/ false,
    )
}

pub(crate) fn create(
    codex_home: &Path,
    id: &str,
    extends: Option<&str>,
) -> Result<PathBuf, String> {
    if !valid_id(id) {
        return Err("theme id must contain only letters, numbers, '-', and '_'".to_string());
    }
    let registry = registry(codex_home);
    if registry.get(id).is_some() {
        return Err(format!("theme `{id}` already exists"));
    }
    let dir = codex_home.join("themes");
    fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    let path = dir.join(format!("{id}.json"));
    let parent = extends.unwrap_or("claude-dark");
    let file = serde_json::json!({
        "$schema": "https://example.com/codex-theme-v1.schema.json",
        "schemaVersion": 1,
        "id": id,
        "name": id_to_name(id),
        "description": "Custom Codex UI theme.",
        "author": "",
        "version": "1.0.0",
        "appearance": "dark",
        "extends": parent,
        "palette": {},
        "tokens": {},
        "components": {}
    });
    let pretty = serde_json::to_string_pretty(&file).map_err(|err| err.to_string())?;
    fs::write(&path, format!("{pretty}\n")).map_err(|err| err.to_string())?;
    Ok(path)
}

fn load_resolved(codex_home: &Path, id: &str) -> Result<ResolvedThemeResult, ThemeDiagnostic> {
    let registry = registry(codex_home);
    let files = registry.files();
    if !files.contains_key(id) {
        return Err(ThemeDiagnostic {
            path: "id".to_string(),
            message: format!("unknown theme `{id}`"),
        });
    }
    resolve_theme(id, &files)
}

fn set_current(theme: ResolvedTheme) {
    let lock = ACTIVE_THEME.get_or_init(|| RwLock::new(Arc::new(ResolvedTheme::fallback())));
    *lock.write().expect("theme lock poisoned") = Arc::new(theme);
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}

fn id_to_name(id: &str) -> String {
    id.split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[allow(dead_code)]
fn _assert_theme_file_is_send_sync(_: ThemeFile) {}
