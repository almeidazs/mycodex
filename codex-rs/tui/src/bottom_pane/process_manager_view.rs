use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::style::Styled;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use unicode_width::UnicodeWidthStr;

use super::CancellationEvent;
use super::bottom_pane_view::BottomPaneView;
use super::bottom_pane_view::ViewCompletion;
use super::selection_popup_common::render_menu_surface;
use crate::app_event::AppEvent;
use crate::app_event::ProcessManagerAction;
use crate::app_event_sender::AppEventSender;
use crate::key_hint;
use crate::render::renderable::Renderable;

pub(crate) const PROCESS_MANAGER_VIEW_ID: &str = "process-manager";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProcessManagerStatus {
    Running,
    Exited { exit_code: Option<i32> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessManagerItem {
    pub(crate) process_id: String,
    pub(crate) name: String,
    pub(crate) command: String,
    pub(crate) cwd: String,
    pub(crate) elapsed: String,
    pub(crate) url: Option<String>,
    pub(crate) status: ProcessManagerStatus,
    pub(crate) recent_output: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessManagerViewParams {
    pub(crate) items: Vec<ProcessManagerItem>,
    pub(crate) selected_index: Option<usize>,
    pub(crate) output_process_id: Option<String>,
    pub(crate) follow_output: bool,
}

pub(crate) struct ProcessManagerView {
    app_event_tx: AppEventSender,
    params: ProcessManagerViewParams,
    selected_index: usize,
    complete: Option<ViewCompletion>,
}

impl ProcessManagerView {
    pub(crate) fn new(params: ProcessManagerViewParams, app_event_tx: AppEventSender) -> Self {
        let selected_index = params
            .selected_index
            .or_else(|| {
                params.output_process_id.as_ref().and_then(|process_id| {
                    params
                        .items
                        .iter()
                        .position(|item| &item.process_id == process_id)
                })
            })
            .unwrap_or(0)
            .min(params.items.len().saturating_sub(1));
        Self {
            app_event_tx,
            params,
            selected_index,
            complete: None,
        }
    }

    fn selected_item(&self) -> Option<&ProcessManagerItem> {
        self.params.items.get(self.selected_index)
    }

    fn selected_item_index(&self) -> Option<usize> {
        (!self.params.items.is_empty()).then_some(self.selected_index)
    }

    fn output_item(&self) -> Option<&ProcessManagerItem> {
        let process_id = self.params.output_process_id.as_ref()?;
        self.params
            .items
            .iter()
            .find(|item| &item.process_id == process_id)
    }

    fn send_action(&self, action: ProcessManagerAction) {
        self.app_event_tx
            .send(AppEvent::ProcessManagerAction(action));
    }

    fn move_selection_up(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    fn move_selection_down(&mut self) {
        if self.selected_index + 1 < self.params.items.len() {
            self.selected_index += 1;
        }
    }

    fn truncated_text(text: &str, width: usize) -> String {
        if UnicodeWidthStr::width(text) <= width {
            return text.to_string();
        }
        if width <= 1 {
            return "…".to_string();
        }
        let mut result = String::new();
        let mut used = 0;
        for ch in text.chars() {
            let ch_width = UnicodeWidthStr::width(ch.to_string().as_str());
            if used + ch_width + 1 > width {
                break;
            }
            result.push(ch);
            used += ch_width;
        }
        result.push('…');
        result
    }

    fn status_icon(status: &ProcessManagerStatus) -> Span<'static> {
        match status {
            ProcessManagerStatus::Running => "●".green(),
            ProcessManagerStatus::Exited { .. } => "✗".red(),
        }
    }

    fn runtime_label(item: &ProcessManagerItem) -> String {
        match &item.status {
            ProcessManagerStatus::Running => item.elapsed.clone(),
            ProcessManagerStatus::Exited { exit_code } => exit_code
                .map(|code| format!("exited {code}"))
                .unwrap_or_else(|| "exited".to_string()),
        }
    }

    fn list_lines(&self, width: u16) -> Vec<Line<'static>> {
        let mut lines = vec![Line::from("Processes".bold()), Line::from("")];
        if self.params.items.is_empty() {
            lines.push(Line::from("No managed background processes.".dim()));
        } else {
            let command_width = width.saturating_sub(39).max(12) as usize;
            for (idx, item) in self.params.items.iter().enumerate() {
                let mut spans = vec![
                    Self::status_icon(&item.status),
                    " ".into(),
                    format!("{:<15}", Self::truncated_text(&item.name, 15)).into(),
                    " ".into(),
                    format!("{:<18}", Self::truncated_text(&item.command, command_width)).dim(),
                    " ".into(),
                    format!("{:>8}", Self::runtime_label(item)).dim(),
                ];
                if let Some(url) = &item.url {
                    spans.push("   ".into());
                    spans.push(Self::truncated_text(url, 24).cyan().underlined());
                }
                if idx == self.selected_index {
                    spans.insert(0, "› ".cyan().bold());
                    spans = spans
                        .into_iter()
                        .map(|span| {
                            let style = span.style.patch(Style::new().bold());
                            span.set_style(style)
                        })
                        .collect();
                } else {
                    spans.insert(0, "  ".into());
                }
                lines.push(Line::from(spans));
            }
        }
        lines.push(Line::from(""));
        lines.extend(self.hint_lines());
        lines
    }

    fn output_lines(&self, width: u16) -> Vec<Line<'static>> {
        let Some(item) = self.output_item() else {
            return self.list_lines(width);
        };
        let title = if self.params.follow_output {
            format!("Following logs · {}", item.name)
        } else {
            format!("Process output · {}", item.name)
        };
        let mut lines = vec![Line::from(title.bold())];
        lines.push(Line::from(item.command.clone().dim()));
        if let Some(url) = &item.url {
            lines.push(Line::from(url.clone().cyan().underlined()));
        }
        lines.push(Line::from(""));
        if item.recent_output.trim().is_empty() {
            lines.push(Line::from("No output captured yet.".dim()));
        } else {
            let max_lines = 18usize;
            let output_lines = item
                .recent_output
                .lines()
                .rev()
                .take(max_lines)
                .collect::<Vec<_>>()
                .into_iter()
                .rev();
            let output_width = width.max(8) as usize;
            for line in output_lines {
                lines.push(Line::from(Self::truncated_text(line, output_width).dim()));
            }
        }
        lines.push(Line::from(""));
        lines.extend(self.hint_lines());
        lines
    }

    fn hint_lines(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(vec![
                key_hint::plain(KeyCode::Enter).into(),
                " view output  ".dim(),
                key_hint::plain(KeyCode::Char('k')).into(),
                " kill  ".dim(),
                key_hint::plain(KeyCode::Char('r')).into(),
                " restart  ".dim(),
                key_hint::plain(KeyCode::Char('o')).into(),
                " open URL".dim(),
            ]),
            Line::from(vec![
                key_hint::plain(KeyCode::Char('f')).into(),
                " follow logs  ".dim(),
                key_hint::plain(KeyCode::Char('l')).into(),
                " logs  ".dim(),
                key_hint::plain(KeyCode::Char('c')).into(),
                " copy command  ".dim(),
                key_hint::plain(KeyCode::Esc).into(),
                " close".dim(),
            ]),
        ]
    }

    fn lines(&self, width: u16) -> Vec<Line<'static>> {
        if self.params.output_process_id.is_some() {
            self.output_lines(width)
        } else {
            self.list_lines(width)
        }
    }

    fn selected_process_action(
        &self,
        build: impl FnOnce(&ProcessManagerItem) -> ProcessManagerAction,
    ) {
        if let Some(item) = self.selected_item() {
            self.send_action(build(item));
        }
    }
}

impl Renderable for ProcessManagerView {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }
        let content_area = render_menu_surface(area, buf);
        Paragraph::new(self.lines(content_area.width)).render(content_area, buf);
    }

    fn desired_height(&self, width: u16) -> u16 {
        self.lines(width).len() as u16 + 2
    }
}

impl BottomPaneView for ProcessManagerView {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event {
            KeyEvent {
                code: KeyCode::Esc, ..
            } => self.complete = Some(ViewCompletion::Cancelled),
            KeyEvent {
                code: KeyCode::Up, ..
            } => self.move_selection_up(),
            KeyEvent {
                code: KeyCode::Down,
                ..
            } => self.move_selection_down(),
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => self.selected_process_action(|item| ProcessManagerAction::ViewOutput {
                process_id: item.process_id.clone(),
            }),
            KeyEvent {
                code: KeyCode::Char('k'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.selected_process_action(|item| ProcessManagerAction::Kill {
                    process_id: item.process_id.clone(),
                });
            }
            KeyEvent {
                code: KeyCode::Char('r'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.selected_process_action(|item| ProcessManagerAction::Restart {
                    process_id: item.process_id.clone(),
                    command: item.command.clone(),
                    cwd: item.cwd.clone(),
                });
            }
            KeyEvent {
                code: KeyCode::Char('o'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                if let Some(url) = self.selected_item().and_then(|item| item.url.clone()) {
                    self.send_action(ProcessManagerAction::OpenUrl { url });
                }
            }
            KeyEvent {
                code: KeyCode::Char('f') | KeyCode::Char('l'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.selected_process_action(|item| ProcessManagerAction::FollowLogs {
                    process_id: item.process_id.clone(),
                });
            }
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.selected_process_action(|item| ProcessManagerAction::CopyCommand {
                    command: item.command.clone(),
                });
            }
            _ => {}
        }
    }

    fn is_complete(&self) -> bool {
        self.complete.is_some()
    }

    fn completion(&self) -> Option<ViewCompletion> {
        self.complete
    }

    fn view_id(&self) -> Option<&'static str> {
        Some(PROCESS_MANAGER_VIEW_ID)
    }

    fn selected_index(&self) -> Option<usize> {
        self.selected_item_index()
    }

    fn process_manager_follow_target(&self) -> Option<&str> {
        self.params
            .follow_output
            .then_some(self.params.output_process_id.as_deref())
            .flatten()
    }

    fn on_ctrl_c(&mut self) -> CancellationEvent {
        self.complete = Some(ViewCompletion::Cancelled);
        CancellationEvent::Handled
    }

    fn prefer_esc_to_handle_key_event(&self) -> bool {
        true
    }
}
