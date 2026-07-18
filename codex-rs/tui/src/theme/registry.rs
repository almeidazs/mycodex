use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use super::builtins::load_builtins;
use super::resolver::ThemeDiagnostic;
use super::resolver::resolve_theme;
use super::schema::ThemeFile;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ThemeSource {
    BuiltIn,
    User,
    Project,
}

impl ThemeSource {
    pub(crate) fn label(self) -> &'static str {
        match self {
            ThemeSource::BuiltIn => "built-in",
            ThemeSource::User => "user",
            ThemeSource::Project => "project",
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RegisteredTheme {
    pub(crate) file: ThemeFile,
    pub(crate) source: ThemeSource,
    pub(crate) path: Option<PathBuf>,
    pub(crate) valid: bool,
    pub(crate) error: Option<ThemeDiagnostic>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ThemeRegistry {
    pub(crate) themes: HashMap<String, RegisteredTheme>,
    pub(crate) duplicate_errors: Vec<ThemeDiagnostic>,
}

impl ThemeRegistry {
    pub(crate) fn load(
        codex_home: &Path,
        project_root: Option<&Path>,
        trusted_project: bool,
    ) -> Self {
        let mut registry = ThemeRegistry::default();
        for file in load_builtins() {
            registry.insert(file, ThemeSource::BuiltIn, None);
        }
        registry.load_dir(&codex_home.join("themes"), ThemeSource::User);
        if trusted_project && let Some(project_root) = project_root {
            registry.load_dir(&project_root.join(".codex/themes"), ThemeSource::Project);
        }
        let files = registry
            .themes
            .iter()
            .map(|(id, entry)| (id.clone(), entry.file.clone()))
            .collect::<HashMap<_, _>>();
        for (id, entry) in &mut registry.themes {
            if entry.valid {
                if let Err(err) = resolve_theme(id, &files) {
                    entry.valid = false;
                    entry.error = Some(err);
                }
            }
        }
        registry
    }

    pub(crate) fn files(&self) -> HashMap<String, ThemeFile> {
        self.themes
            .iter()
            .filter(|(_, entry)| entry.valid)
            .map(|(id, entry)| (id.clone(), entry.file.clone()))
            .collect()
    }

    pub(crate) fn sorted_entries(&self) -> Vec<&RegisteredTheme> {
        let mut entries = self.themes.values().collect::<Vec<_>>();
        entries.sort_by_key(|entry| {
            (
                source_rank(entry.source),
                entry.file.name.to_ascii_lowercase(),
            )
        });
        entries
    }

    pub(crate) fn get(&self, id: &str) -> Option<&RegisteredTheme> {
        self.themes.get(id)
    }

    fn load_dir(&mut self, dir: &Path, source: ThemeSource) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            match fs::read_to_string(&path).and_then(|raw| {
                serde_json::from_str::<ThemeFile>(&raw)
                    .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
            }) {
                Ok(file) => self.insert(file, source, Some(path)),
                Err(err) => {
                    let id = path
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .unwrap_or("invalid-theme")
                        .to_string();
                    self.themes.insert(
                        id.clone(),
                        RegisteredTheme {
                            file: ThemeFile::invalid_placeholder(id),
                            source,
                            path: Some(path),
                            valid: false,
                            error: Some(ThemeDiagnostic {
                                path: String::new(),
                                message: err.to_string(),
                            }),
                        },
                    );
                }
            }
        }
    }

    fn insert(&mut self, file: ThemeFile, source: ThemeSource, path: Option<PathBuf>) {
        let id = file.id.clone();
        if self.themes.contains_key(&id) {
            self.duplicate_errors.push(ThemeDiagnostic {
                path: "id".to_string(),
                message: format!("duplicate theme id `{id}`"),
            });
            return;
        }
        self.themes.insert(
            id,
            RegisteredTheme {
                file,
                source,
                path,
                valid: true,
                error: None,
            },
        );
    }
}

fn source_rank(source: ThemeSource) -> u8 {
    match source {
        ThemeSource::BuiltIn => 0,
        ThemeSource::User => 1,
        ThemeSource::Project => 2,
    }
}
