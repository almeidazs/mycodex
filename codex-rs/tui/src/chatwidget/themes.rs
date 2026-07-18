use super::*;
use crate::app_event::AppEvent;
use crate::bottom_pane::SelectionItem;
use crate::bottom_pane::SelectionViewParams;
use crate::bottom_pane::popup_consts::standard_popup_hint_line;

impl ChatWidget {
    pub(super) fn open_ui_theme_manager(&mut self) {
        let codex_home = self.config.codex_home.clone();
        let registry = crate::theme::registry(&codex_home);
        let active_id = crate::theme::current().id.clone();
        let mut initial_idx = None;
        let entries = registry.sorted_entries();
        let ids = entries
            .iter()
            .map(|entry| entry.file.id.clone())
            .collect::<Vec<_>>();
        let items = entries
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let id = entry.file.id.clone();
                let is_current = id == active_id;
                if is_current {
                    initial_idx = Some(idx);
                }
                let status = if entry.valid { "" } else { "invalid · " };
                let location = entry
                    .path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| entry.file.id.clone());
                let description = Some(format!(
                    "{}{} · {} · {}",
                    status,
                    entry.source.label(),
                    appearance_label(&entry.file.appearance),
                    location
                ));
                let disabled_reason = (!entry.valid).then(|| {
                    entry
                        .error
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "theme is invalid".to_string())
                });
                SelectionItem {
                    name: entry.file.name.clone(),
                    description,
                    selected_description: entry.error.as_ref().map(ToString::to_string),
                    is_current,
                    is_disabled: !entry.valid,
                    disabled_reason,
                    dismiss_on_select: true,
                    search_value: Some(format!("{} {}", entry.file.id, entry.file.name)),
                    actions: vec![Box::new(move |tx| {
                        tx.send(AppEvent::UiThemeSelected { id: id.clone() });
                    })],
                    ..Default::default()
                }
            })
            .collect::<Vec<_>>();

        let preview_home = codex_home.clone();
        let on_selection_changed = Some(Box::new(
            move |idx: usize, tx: &crate::app_event_sender::AppEventSender| {
                if let Some(id) = ids.get(idx)
                    && crate::theme::preview(&preview_home, id).is_ok()
                {
                    tx.send(AppEvent::UiThemePreviewed);
                }
            },
        )
            as Box<dyn Fn(usize, &crate::app_event_sender::AppEventSender) + Send + Sync>);
        let preview_ids = entries
            .iter()
            .map(|entry| entry.file.id.clone())
            .collect::<Vec<_>>();
        let explicit_preview_home = codex_home.clone();
        let on_preview = Some(Box::new(
            move |idx: usize, tx: &crate::app_event_sender::AppEventSender| {
                if let Some(id) = preview_ids.get(idx) {
                    let _ = crate::theme::preview(&explicit_preview_home, id);
                    tx.send(AppEvent::UiThemePreviewRequested { id: id.clone() });
                }
            },
        )
            as Box<dyn Fn(usize, &crate::app_event_sender::AppEventSender) + Send + Sync>);
        let restore_home = codex_home;
        let restore_id = active_id;
        let on_cancel = Some(
            Box::new(move |tx: &crate::app_event_sender::AppEventSender| {
                let _ = crate::theme::preview(&restore_home, &restore_id);
                tx.send(AppEvent::UiThemePreviewed);
            }) as Box<dyn Fn(&crate::app_event_sender::AppEventSender) + Send + Sync>,
        );

        self.bottom_pane.show_selection_view(SelectionViewParams {
            title: Some("UI Themes".to_string()),
            subtitle: Some("Enter apply · p preview · Esc restore previous preview".to_string()),
            footer_hint: Some(standard_popup_hint_line()),
            items,
            is_searchable: true,
            search_placeholder: Some("Type to filter UI themes...".to_string()),
            initial_selected_idx: initial_idx,
            on_selection_changed,
            on_preview,
            on_cancel,
            ..Default::default()
        });
    }

    pub(super) fn handle_themes_command_args(&mut self, args: &str) {
        let mut parts = args.split_whitespace();
        match parts.next().unwrap_or("list") {
            "list" => self.add_ui_theme_list(),
            "current" => {
                let theme = crate::theme::current();
                self.add_info_message(
                    format!("Current UI theme: {} ({})", theme.name, theme.id),
                    Some(format!("Syntax theme: {}", theme.syntax_theme)),
                );
            }
            "use" => {
                let Some(id) = parts.next() else {
                    self.add_error_message("Usage: /themes use <id>".to_string());
                    return;
                };
                self.app_event_tx
                    .send(AppEvent::UiThemeSelected { id: id.to_string() });
            }
            "preview" => {
                let Some(id) = parts.next() else {
                    self.add_error_message("Usage: /themes preview <id>".to_string());
                    return;
                };
                match crate::theme::preview(&self.config.codex_home, id) {
                    Ok(result) => {
                        self.add_info_message(
                            format!("Previewing UI theme: {} ({})", result.theme.name, result.theme.id),
                            Some("Use `/themes use <id>` to persist it, or `/themes reset` to restore the fallback.".to_string()),
                        );
                        self.app_event_tx.send(AppEvent::UiThemePreviewed);
                    }
                    Err(err) => self.add_error_message(format!("Invalid UI theme: {err}")),
                }
            }
            "validate" => {
                let Some(id) = parts.next() else {
                    self.add_error_message("Usage: /themes validate <id>".to_string());
                    return;
                };
                match crate::theme::validate(&self.config.codex_home, id) {
                    Ok(result) => {
                        let warning_count = result.warnings.len();
                        self.add_info_message(
                            format!("Theme `{id}` is valid."),
                            Some(format!("{warning_count} warning(s).")),
                        );
                    }
                    Err(err) => self.add_error_message(format!("Theme `{id}` is invalid: {err}")),
                }
            }
            "create" => {
                let Some(id) = parts.next() else {
                    self.add_error_message("Usage: /themes create <id> [--extends parent]".to_string());
                    return;
                };
                let mut extends = None;
                while let Some(part) = parts.next() {
                    if part == "--extends" {
                        extends = parts.next();
                    }
                }
                match crate::theme::create(&self.config.codex_home, id, extends) {
                    Ok(path) => self.add_info_message(
                        format!("Created UI theme `{id}`."),
                        Some(path.display().to_string()),
                    ),
                    Err(err) => self.add_error_message(format!("Could not create theme `{id}`: {err}")),
                }
            }
            "reset" => {
                self.app_event_tx.send(AppEvent::UiThemeSelected {
                    id: "codex-dark".to_string(),
                });
            }
            _ => self.add_error_message(
                "Usage: /themes [list|current|use <id>|preview <id>|validate <id>|create <id> [--extends parent]|reset]".to_string(),
            ),
        }
    }

    fn add_ui_theme_list(&mut self) {
        let registry = crate::theme::registry(&self.config.codex_home);
        let active_id = crate::theme::current().id.clone();
        let mut lines = vec![
            format!("Current UI theme: {active_id}"),
            String::new(),
            "Themes".to_string(),
        ];
        for entry in registry.sorted_entries() {
            let marker = if entry.file.id == active_id { "*" } else { " " };
            let status = if entry.valid { "" } else { " invalid" };
            lines.push(format!(
                "{marker} {:<18} {:<9} {:<5}{}",
                entry.file.id,
                entry.source.label(),
                appearance_label(&entry.file.appearance),
                status
            ));
        }
        self.add_info_message(
            lines.join("\n"),
            Some("Use `/themes` for the interactive picker.".to_string()),
        );
    }
}

fn appearance_label(appearance: &crate::theme::schema::ThemeAppearance) -> &'static str {
    match appearance {
        crate::theme::schema::ThemeAppearance::Dark => "dark",
        crate::theme::schema::ThemeAppearance::Light => "light",
        crate::theme::schema::ThemeAppearance::Auto => "auto",
    }
}
