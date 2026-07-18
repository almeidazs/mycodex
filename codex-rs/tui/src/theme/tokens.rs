use ratatui::style::Style;
use ratatui::style::Stylize;

use super::schema::ThemeAppearance;

#[derive(Clone, Debug)]
pub(crate) struct ResolvedTheme {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) appearance: ThemeAppearance,
    pub(crate) syntax_theme: String,
    pub(crate) text: TextTokens,
    pub(crate) accent: AccentTokens,
    pub(crate) surface: SurfaceTokens,
    pub(crate) border: BorderTokens,
    pub(crate) status: StatusTokens,
    pub(crate) diff: DiffTokens,
    pub(crate) git: GitTokens,
    pub(crate) markdown: MarkdownTokens,
    pub(crate) components: ComponentTokens,
}

#[derive(Clone, Debug)]
pub(crate) struct TextTokens {
    pub(crate) primary: Style,
    pub(crate) secondary: Style,
    pub(crate) muted: Style,
    pub(crate) disabled: Style,
    pub(crate) link: Style,
}
#[derive(Clone, Debug)]
pub(crate) struct AccentTokens {
    pub(crate) primary: Style,
    pub(crate) secondary: Style,
    pub(crate) muted: Style,
}
#[derive(Clone, Debug)]
pub(crate) struct SurfaceTokens {
    pub(crate) base: Style,
    pub(crate) raised: Style,
    pub(crate) overlay: Style,
    pub(crate) selected: Style,
    pub(crate) input: Style,
}
#[derive(Clone, Debug)]
pub(crate) struct BorderTokens {
    pub(crate) subtle: Style,
    pub(crate) default: Style,
    pub(crate) strong: Style,
    pub(crate) focus: Style,
    pub(crate) error: Style,
}
#[derive(Clone, Debug)]
pub(crate) struct StatusTokens {
    pub(crate) running: Style,
    pub(crate) success: Style,
    pub(crate) warning: Style,
    pub(crate) error: Style,
    pub(crate) info: Style,
    pub(crate) cancelled: Style,
    pub(crate) paused: Style,
}
#[derive(Clone, Debug)]
pub(crate) struct DiffTokens {
    pub(crate) added: Style,
    pub(crate) removed: Style,
    pub(crate) modified: Style,
    pub(crate) context: Style,
    pub(crate) line_number: Style,
    pub(crate) hunk_header: Style,
}
#[derive(Clone, Debug)]
pub(crate) struct GitTokens {
    pub(crate) added: Style,
    pub(crate) modified: Style,
    pub(crate) deleted: Style,
    pub(crate) renamed: Style,
    pub(crate) untracked: Style,
    pub(crate) conflicted: Style,
}
#[derive(Clone, Debug)]
pub(crate) struct MarkdownTokens {
    pub(crate) heading: Style,
    pub(crate) bold: Style,
    pub(crate) italic: Style,
    pub(crate) quote: Style,
    pub(crate) list_marker: Style,
    pub(crate) inline_code: Style,
    pub(crate) link: Style,
}
#[derive(Clone, Debug)]
pub(crate) struct ComponentTokens {
    pub(crate) composer: ComposerTokens,
    pub(crate) tool_call: ToolCallTokens,
    pub(crate) user_message: MessageTokens,
    pub(crate) assistant_message: MessageTokens,
    pub(crate) picker: PickerTokens,
    pub(crate) modal: ModalTokens,
}
#[derive(Clone, Debug)]
pub(crate) struct ComposerTokens {
    pub(crate) background: Style,
    pub(crate) border: Style,
    pub(crate) border_focused: Style,
    pub(crate) prompt: Style,
    pub(crate) placeholder: Style,
}
#[derive(Clone, Debug)]
pub(crate) struct ToolCallTokens {
    pub(crate) running: Style,
    pub(crate) success: Style,
    pub(crate) error: Style,
    pub(crate) description: Style,
    pub(crate) metadata: Style,
}
#[derive(Clone, Debug)]
pub(crate) struct MessageTokens {
    pub(crate) label: Style,
    pub(crate) body: Style,
}
#[derive(Clone, Debug)]
pub(crate) struct PickerTokens {
    pub(crate) selected: Style,
    pub(crate) selected_text: Style,
    pub(crate) metadata: Style,
}
#[derive(Clone, Debug)]
pub(crate) struct ModalTokens {
    pub(crate) border: Style,
    pub(crate) border_focused: Style,
    pub(crate) title: Style,
}

impl ResolvedTheme {
    pub(crate) fn fallback() -> Self {
        let text_primary = Style::new();
        let text_secondary = Style::new().dim();
        let text_muted = Style::new().dim();
        let accent = Style::new().cyan().bold();
        let success = Style::new().green();
        let warning = Style::new().yellow();
        let error = Style::new().red();
        let info = Style::new().cyan();
        Self {
            id: "codex-dark".to_string(),
            name: "Codex Dark".to_string(),
            appearance: ThemeAppearance::Dark,
            syntax_theme: "default-dark".to_string(),
            text: TextTokens {
                primary: text_primary,
                secondary: text_secondary,
                muted: text_muted,
                disabled: Style::new().dim(),
                link: Style::new().cyan().underlined(),
            },
            accent: AccentTokens {
                primary: accent,
                secondary: Style::new().cyan(),
                muted: Style::new().cyan().dim(),
            },
            surface: SurfaceTokens {
                base: Style::new(),
                raised: Style::new(),
                overlay: Style::new(),
                selected: Style::new().reversed(),
                input: Style::new(),
            },
            border: BorderTokens {
                subtle: Style::new().dim(),
                default: Style::new(),
                strong: Style::new().bold(),
                focus: accent,
                error,
            },
            status: StatusTokens {
                running: accent,
                success,
                warning,
                error,
                info,
                cancelled: text_muted,
                paused: warning,
            },
            diff: DiffTokens {
                added: success,
                removed: error,
                modified: warning,
                context: text_secondary,
                line_number: text_muted,
                hunk_header: info,
            },
            git: GitTokens {
                added: success,
                modified: warning,
                deleted: error,
                renamed: info,
                untracked: text_muted,
                conflicted: error,
            },
            markdown: MarkdownTokens {
                heading: text_primary.bold(),
                bold: text_primary.bold(),
                italic: text_primary.italic(),
                quote: text_secondary,
                list_marker: accent,
                inline_code: warning,
                link: Style::new().cyan().underlined(),
            },
            components: ComponentTokens {
                composer: ComposerTokens {
                    background: Style::new(),
                    border: Style::new().dim(),
                    border_focused: accent,
                    prompt: accent,
                    placeholder: text_muted,
                },
                tool_call: ToolCallTokens {
                    running: accent,
                    success,
                    error,
                    description: text_secondary,
                    metadata: text_muted,
                },
                user_message: MessageTokens {
                    label: accent,
                    body: text_primary,
                },
                assistant_message: MessageTokens {
                    label: text_secondary,
                    body: text_primary,
                },
                picker: PickerTokens {
                    selected: Style::new().reversed(),
                    selected_text: Style::new().bold(),
                    metadata: text_muted,
                },
                modal: ModalTokens {
                    border: Style::new().dim(),
                    border_focused: accent,
                    title: Style::new().bold(),
                },
            },
        }
    }
}
