use super::schema::ThemeFile;

pub(crate) const BUILTIN_THEME_JSON: &[&str] = &[
    CODEX_DARK,
    CODEX_LIGHT,
    CLAUDE_DARK,
    CLAUDE_LIGHT,
    MINIMAL_DARK,
    MINIMAL_LIGHT,
    HIGH_CONTRAST,
    ANSI,
];

pub(crate) fn load_builtins() -> Vec<ThemeFile> {
    BUILTIN_THEME_JSON
        .iter()
        .filter_map(|json| serde_json::from_str::<ThemeFile>(json).ok())
        .collect()
}

const CODEX_DARK: &str = r##"{
  "$schema": "https://example.com/codex-theme-v1.schema.json",
  "schemaVersion": 1,
  "id": "codex-dark",
  "name": "Codex Dark",
  "description": "Default Codex semantic dark theme.",
  "appearance": "dark",
  "palette": {
    "foreground": "#e6e6e6",
    "muted": "#8a8f98",
    "accent": "cyan",
    "green": "green",
    "yellow": "yellow",
    "red": "red",
    "blue": "blue"
  },
  "tokens": {
    "text.primary": "$palette.foreground",
    "text.secondary": "#b8bcc4",
    "text.muted": "$palette.muted",
    "text.disabled": "dark-gray",
    "text.link": "cyan",
    "accent.primary": "$palette.accent",
    "accent.secondary": "light-cyan",
    "accent.muted": "cyan",
    "surface.base": "terminal.background",
    "surface.raised": "transparent",
    "surface.overlay": "transparent",
    "surface.selected": "transparent",
    "surface.input": "transparent",
    "border.subtle": "dark-gray",
    "border.default": "gray",
    "border.strong": "white",
    "border.focus": "$palette.accent",
    "border.error": "$palette.red",
    "status.running": "$palette.accent",
    "status.success": "$palette.green",
    "status.warning": "$palette.yellow",
    "status.error": "$palette.red",
    "status.info": "$palette.blue",
    "status.cancelled": "$palette.muted",
    "status.paused": "$palette.yellow",
    "diff.added": "$palette.green",
    "diff.removed": "$palette.red",
    "diff.modified": "$palette.yellow",
    "diff.context": "$text.secondary",
    "diff.lineNumber": "$text.muted",
    "diff.hunkHeader": "$palette.blue",
    "git.added": "$palette.green",
    "git.modified": "$palette.yellow",
    "git.deleted": "$palette.red",
    "git.renamed": "$palette.blue",
    "git.untracked": "$text.muted",
    "git.conflicted": "$palette.red",
    "markdown.heading": {"foreground":"$text.primary", "modifiers":["bold"]},
    "markdown.bold": {"foreground":"$text.primary", "modifiers":["bold"]},
    "markdown.italic": {"foreground":"$text.primary", "modifiers":["italic"]},
    "markdown.quote": "$text.secondary",
    "markdown.listMarker": "$accent.primary",
    "markdown.inlineCode": "$status.warning",
    "markdown.link": {"foreground":"$text.link", "modifiers":["underline"]},
    "syntax.theme": "default-dark"
  },
  "components": {
    "composer": {"background":"$surface.input", "border":"$border.subtle", "borderFocused":"$border.focus", "prompt":"$accent.primary", "placeholder":"$text.muted"},
    "toolCall": {"running":"$status.running", "success":"$status.success", "error":"$status.error", "description":"$text.secondary", "metadata":"$text.muted"},
    "userMessage": {"label":"$accent.primary", "body":"$text.primary"},
    "assistantMessage": {"label":"$text.secondary", "body":"$text.primary"},
    "picker": {"selected":"transparent", "selectedText":{"foreground":"$text.primary", "modifiers":["bold"]}, "metadata":"$text.muted"},
    "modal": {"border":"$border.subtle", "borderFocused":"$border.focus", "title":{"foreground":"$text.primary", "modifiers":["bold"]}}
  }
}"##;

const CODEX_LIGHT: &str = r##"{"schemaVersion":1,"id":"codex-light","name":"Codex Light","appearance":"light","extends":"codex-dark","palette":{"foreground":"#1f2328","muted":"#6e7781","accent":"blue","green":"green","yellow":"yellow","red":"red","blue":"blue"},"tokens":{"syntax.theme":"default-light","text.primary":"$palette.foreground","text.secondary":"#4b5563","text.muted":"$palette.muted"}}"##;
const CLAUDE_DARK: &str = r##"{"schemaVersion":1,"id":"claude-dark","name":"Claude Dark","appearance":"dark","extends":"codex-dark","palette":{"accent":"#d97757","foreground":"#e8e6e3","muted":"#8b8985","green":"#79b98a","yellow":"#d6a85f","red":"#d96c75","blue":"#72a6c9"},"tokens":{"accent.primary":"$palette.accent","syntax.theme":"catppuccin-mocha"}}"##;
const CLAUDE_LIGHT: &str = r##"{"schemaVersion":1,"id":"claude-light","name":"Claude Light","appearance":"light","extends":"codex-light","palette":{"accent":"#b85f3b","foreground":"#2b2926","muted":"#716e68","green":"#3f8f57","yellow":"#9a6a2f","red":"#b84d5a","blue":"#3b779a"},"tokens":{"accent.primary":"$palette.accent","syntax.theme":"catppuccin-latte"}}"##;
const MINIMAL_DARK: &str = r##"{"schemaVersion":1,"id":"minimal-dark","name":"Minimal Dark","appearance":"dark","extends":"codex-dark","tokens":{"accent.primary":"gray","status.success":"gray","status.warning":"gray","status.error":"gray","syntax.theme":"ansi"}}"##;
const MINIMAL_LIGHT: &str = r##"{"schemaVersion":1,"id":"minimal-light","name":"Minimal Light","appearance":"light","extends":"codex-light","tokens":{"accent.primary":"dark-gray","status.success":"dark-gray","status.warning":"dark-gray","status.error":"dark-gray","syntax.theme":"ansi"}}"##;
const HIGH_CONTRAST: &str = r##"{"schemaVersion":1,"id":"high-contrast","name":"High Contrast","appearance":"dark","extends":"codex-dark","tokens":{"text.primary":"white","text.secondary":"white","text.muted":"gray","accent.primary":"yellow","status.success":"light-green","status.warning":"yellow","status.error":"light-red","status.info":"light-blue","border.focus":"yellow","syntax.theme":"ansi"}}"##;
const ANSI: &str = r##"{"schemaVersion":1,"id":"ansi","name":"ANSI","appearance":"auto","extends":"codex-dark","tokens":{"text.primary":"terminal.foreground","text.secondary":"gray","text.muted":"dark-gray","accent.primary":"cyan","status.success":"green","status.warning":"yellow","status.error":"red","status.info":"blue","syntax.theme":"ansi"}}"##;
