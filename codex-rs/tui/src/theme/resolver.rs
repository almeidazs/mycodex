use std::collections::BTreeMap;
use std::collections::HashMap;

use ratatui::style::Modifier;
use ratatui::style::Style;

use super::color::parse_color;
use super::schema::ComponentOverrides;
use super::schema::ThemeFile;
use super::schema::ThemeValue;
use super::tokens::ResolvedTheme;
use crate::terminal_palette::stdout_color_level;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ThemeDiagnostic {
    pub(crate) path: String,
    pub(crate) message: String,
}

impl std::fmt::Display for ThemeDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.path.is_empty() {
            f.write_str(&self.message)
        } else {
            write!(f, "{}: {}", self.path, self.message)
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedThemeResult {
    pub(crate) theme: ResolvedTheme,
    pub(crate) warnings: Vec<ThemeDiagnostic>,
}

#[derive(Clone, Debug, Default)]
struct MergedTheme {
    metadata: Option<ThemeFile>,
    palette: BTreeMap<String, ThemeValue>,
    tokens: BTreeMap<String, ThemeValue>,
    components: ComponentOverrides,
}

pub(crate) fn resolve_theme(
    id: &str,
    themes: &HashMap<String, ThemeFile>,
) -> Result<ResolvedThemeResult, ThemeDiagnostic> {
    let mut visiting = Vec::new();
    let merged = merge_theme(id, themes, &mut visiting)?;
    let metadata = merged.metadata.ok_or_else(|| ThemeDiagnostic {
        path: "id".to_string(),
        message: format!("theme `{id}` is missing metadata"),
    })?;
    let mut theme = ResolvedTheme::fallback();
    theme.id = metadata.id;
    theme.name = metadata.name;
    theme.appearance = metadata.appearance;

    let mut values = BTreeMap::new();
    for (key, value) in &merged.palette {
        values.insert(format!("palette.{key}"), value.clone());
    }
    for (key, value) in &merged.tokens {
        if key != "syntax.theme" {
            values.insert(key.clone(), value.clone());
        }
    }

    let mut ctx = ResolveContext {
        values,
        resolved: BTreeMap::new(),
        resolving: Vec::new(),
    };
    for (key, value) in &merged.tokens {
        if key == "syntax.theme" {
            if let ThemeValue::String(name) = value {
                theme.syntax_theme = name.clone();
            }
            continue;
        }
        let style = ctx.resolve_style(value, &format!("tokens.{key}"))?;
        apply_token(&mut theme, key, style)?;
        ctx.resolved.insert(key.clone(), style);
    }
    apply_component_tokens(&mut theme, &merged.components, &mut ctx)?;

    Ok(ResolvedThemeResult {
        theme,
        warnings: Vec::new(),
    })
}

fn merge_theme(
    id: &str,
    themes: &HashMap<String, ThemeFile>,
    visiting: &mut Vec<String>,
) -> Result<MergedTheme, ThemeDiagnostic> {
    if visiting.iter().any(|entry| entry == id) {
        let mut cycle = visiting.join(" -> ");
        if !cycle.is_empty() {
            cycle.push_str(" -> ");
        }
        cycle.push_str(id);
        return Err(ThemeDiagnostic {
            path: "extends".to_string(),
            message: format!("theme inheritance cycle detected: {cycle}"),
        });
    }
    let Some(file) = themes.get(id) else {
        return Err(ThemeDiagnostic {
            path: "extends".to_string(),
            message: format!("unknown parent theme `{id}`"),
        });
    };
    if file.schema_version != 1 {
        return Err(ThemeDiagnostic {
            path: "schemaVersion".to_string(),
            message: format!("unsupported schema version {}", file.schema_version),
        });
    }
    if file.id.trim().is_empty() {
        return Err(ThemeDiagnostic {
            path: "id".to_string(),
            message: "theme id cannot be empty".to_string(),
        });
    }

    visiting.push(id.to_string());
    let mut merged = if let Some(parent) = file.extends.as_deref() {
        merge_theme(parent, themes, visiting)?
    } else {
        MergedTheme::default()
    };
    visiting.pop();

    merged.metadata = Some(file.clone());
    merged.palette.extend(file.palette.clone());
    merged.tokens.extend(file.tokens.clone());
    merge_component_maps(&mut merged.components, &file.components);
    Ok(merged)
}

fn merge_component_maps(target: &mut ComponentOverrides, source: &ComponentOverrides) {
    target.composer.extend(source.composer.clone());
    target.tool_call.extend(source.tool_call.clone());
    target.user_message.extend(source.user_message.clone());
    target
        .assistant_message
        .extend(source.assistant_message.clone());
    target.picker.extend(source.picker.clone());
    target.modal.extend(source.modal.clone());
}

struct ResolveContext {
    values: BTreeMap<String, ThemeValue>,
    resolved: BTreeMap<String, Style>,
    resolving: Vec<String>,
}

impl ResolveContext {
    fn resolve_style(&mut self, value: &ThemeValue, path: &str) -> Result<Style, ThemeDiagnostic> {
        match value {
            ThemeValue::String(value) => self.resolve_string_style(value, path),
            ThemeValue::Style(value) => {
                let mut style = Style::new();
                if let Some(fg) = value.foreground.as_deref() {
                    style = style
                        .fg_style(self.resolve_string_style(fg, &format!("{path}.foreground"))?);
                }
                if let Some(bg) = value.background.as_deref() {
                    let bg_style = self.resolve_string_style(bg, &format!("{path}.background"))?;
                    style.bg = bg_style.fg;
                }
                for modifier in &value.modifiers {
                    style = style.add_modifier(parse_modifier(modifier).map_err(|message| {
                        ThemeDiagnostic {
                            path: format!("{path}.modifiers"),
                            message,
                        }
                    })?);
                }
                Ok(style)
            }
        }
    }

    fn resolve_string_style(&mut self, value: &str, path: &str) -> Result<Style, ThemeDiagnostic> {
        if let Some(reference) = value.strip_prefix('$') {
            return self.resolve_reference(reference, path);
        }
        if let Some(color) = parse_color(value)
            .map_err(|message| ThemeDiagnostic {
                path: path.to_string(),
                message,
            })?
            .to_ratatui(stdout_color_level())
        {
            Ok(Style::new().fg(color))
        } else {
            Ok(Style::new())
        }
    }

    fn resolve_reference(&mut self, reference: &str, path: &str) -> Result<Style, ThemeDiagnostic> {
        if let Some(style) = self.resolved.get(reference).copied() {
            return Ok(style);
        }
        if self.resolving.iter().any(|entry| entry == reference) {
            let mut cycle = self.resolving.join(" -> ");
            if !cycle.is_empty() {
                cycle.push_str(" -> ");
            }
            cycle.push_str(reference);
            return Err(ThemeDiagnostic {
                path: path.to_string(),
                message: format!("theme reference cycle detected: {cycle}"),
            });
        }
        let Some(value) = self.values.get(reference).cloned() else {
            return Err(ThemeDiagnostic {
                path: path.to_string(),
                message: format!("unknown reference `${reference}`"),
            });
        };
        self.resolving.push(reference.to_string());
        let style = self.resolve_style(&value, &format!("${reference}"));
        self.resolving.pop();
        let style = style?;
        self.resolved.insert(reference.to_string(), style);
        Ok(style)
    }
}

trait FgStyleExt {
    fn fg_style(self, other: Style) -> Self;
}

impl FgStyleExt for Style {
    fn fg_style(mut self, other: Style) -> Self {
        self.fg = other.fg;
        self.add_modifier |= other.add_modifier;
        self.sub_modifier |= other.sub_modifier;
        self
    }
}

fn parse_modifier(value: &str) -> Result<Modifier, String> {
    match value {
        "bold" => Ok(Modifier::BOLD),
        "dim" => Ok(Modifier::DIM),
        "italic" => Ok(Modifier::ITALIC),
        "underline" => Ok(Modifier::UNDERLINED),
        "reversed" => Ok(Modifier::REVERSED),
        "crossedOut" | "crossed-out" => Ok(Modifier::CROSSED_OUT),
        other => Err(format!("unknown modifier `{other}`")),
    }
}

fn apply_token(theme: &mut ResolvedTheme, key: &str, style: Style) -> Result<(), ThemeDiagnostic> {
    match key {
        "text.primary" => theme.text.primary = style,
        "text.secondary" => theme.text.secondary = style,
        "text.muted" => theme.text.muted = style,
        "text.disabled" => theme.text.disabled = style,
        "text.link" => theme.text.link = style,
        "accent.primary" => theme.accent.primary = style,
        "accent.secondary" => theme.accent.secondary = style,
        "accent.muted" => theme.accent.muted = style,
        "surface.base" => theme.surface.base = style,
        "surface.raised" => theme.surface.raised = style,
        "surface.overlay" => theme.surface.overlay = style,
        "surface.selected" => theme.surface.selected = style,
        "surface.input" => theme.surface.input = style,
        "border.subtle" => theme.border.subtle = style,
        "border.default" => theme.border.default = style,
        "border.strong" => theme.border.strong = style,
        "border.focus" => theme.border.focus = style,
        "border.error" => theme.border.error = style,
        "status.running" => theme.status.running = style,
        "status.success" => theme.status.success = style,
        "status.warning" => theme.status.warning = style,
        "status.error" => theme.status.error = style,
        "status.info" => theme.status.info = style,
        "status.cancelled" => theme.status.cancelled = style,
        "status.paused" => theme.status.paused = style,
        "diff.added" => theme.diff.added = style,
        "diff.removed" => theme.diff.removed = style,
        "diff.modified" => theme.diff.modified = style,
        "diff.context" => theme.diff.context = style,
        "diff.lineNumber" => theme.diff.line_number = style,
        "diff.hunkHeader" => theme.diff.hunk_header = style,
        "git.added" => theme.git.added = style,
        "git.modified" => theme.git.modified = style,
        "git.deleted" => theme.git.deleted = style,
        "git.renamed" => theme.git.renamed = style,
        "git.untracked" => theme.git.untracked = style,
        "git.conflicted" => theme.git.conflicted = style,
        "markdown.heading" => theme.markdown.heading = style,
        "markdown.bold" => theme.markdown.bold = style,
        "markdown.italic" => theme.markdown.italic = style,
        "markdown.quote" => theme.markdown.quote = style,
        "markdown.listMarker" => theme.markdown.list_marker = style,
        "markdown.inlineCode" => theme.markdown.inline_code = style,
        "markdown.link" => theme.markdown.link = style,
        unknown => {
            return Err(ThemeDiagnostic {
                path: format!("tokens.{unknown}"),
                message: "unknown semantic token".to_string(),
            });
        }
    }
    Ok(())
}

fn apply_component_tokens(
    theme: &mut ResolvedTheme,
    components: &ComponentOverrides,
    ctx: &mut ResolveContext,
) -> Result<(), ThemeDiagnostic> {
    for (key, value) in &components.composer {
        let style = ctx.resolve_style(value, &format!("components.composer.{key}"))?;
        match key.as_str() {
            "background" => theme.components.composer.background = style,
            "border" => theme.components.composer.border = style,
            "borderFocused" => theme.components.composer.border_focused = style,
            "prompt" => theme.components.composer.prompt = style,
            "placeholder" => theme.components.composer.placeholder = style,
            unknown => {
                return Err(ThemeDiagnostic {
                    path: format!("components.composer.{unknown}"),
                    message: "unknown component token".to_string(),
                });
            }
        }
    }
    for (key, value) in &components.tool_call {
        let style = ctx.resolve_style(value, &format!("components.toolCall.{key}"))?;
        match key.as_str() {
            "running" => theme.components.tool_call.running = style,
            "success" => theme.components.tool_call.success = style,
            "error" => theme.components.tool_call.error = style,
            "description" => theme.components.tool_call.description = style,
            "metadata" => theme.components.tool_call.metadata = style,
            unknown => {
                return Err(ThemeDiagnostic {
                    path: format!("components.toolCall.{unknown}"),
                    message: "unknown component token".to_string(),
                });
            }
        }
    }
    for (key, value) in &components.user_message {
        let style = ctx.resolve_style(value, &format!("components.userMessage.{key}"))?;
        match key.as_str() {
            "label" => theme.components.user_message.label = style,
            "body" => theme.components.user_message.body = style,
            unknown => {
                return Err(ThemeDiagnostic {
                    path: format!("components.userMessage.{unknown}"),
                    message: "unknown component token".to_string(),
                });
            }
        }
    }
    for (key, value) in &components.assistant_message {
        let style = ctx.resolve_style(value, &format!("components.assistantMessage.{key}"))?;
        match key.as_str() {
            "label" => theme.components.assistant_message.label = style,
            "body" => theme.components.assistant_message.body = style,
            unknown => {
                return Err(ThemeDiagnostic {
                    path: format!("components.assistantMessage.{unknown}"),
                    message: "unknown component token".to_string(),
                });
            }
        }
    }
    for (key, value) in &components.picker {
        let style = ctx.resolve_style(value, &format!("components.picker.{key}"))?;
        match key.as_str() {
            "selected" => theme.components.picker.selected = style,
            "selectedText" => theme.components.picker.selected_text = style,
            "metadata" => theme.components.picker.metadata = style,
            unknown => {
                return Err(ThemeDiagnostic {
                    path: format!("components.picker.{unknown}"),
                    message: "unknown component token".to_string(),
                });
            }
        }
    }
    for (key, value) in &components.modal {
        let style = ctx.resolve_style(value, &format!("components.modal.{key}"))?;
        match key.as_str() {
            "border" => theme.components.modal.border = style,
            "borderFocused" => theme.components.modal.border_focused = style,
            "title" => theme.components.modal.title = style,
            unknown => {
                return Err(ThemeDiagnostic {
                    path: format!("components.modal.{unknown}"),
                    message: "unknown component token".to_string(),
                });
            }
        }
    }
    Ok(())
}
