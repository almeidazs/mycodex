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
use crate::app_event::QueueManagerAction;
use crate::app_event::QueueManagerMoveDirection;
use crate::app_event_sender::AppEventSender;
use crate::key_hint;
use crate::render::renderable::Renderable;

pub(crate) const QUEUE_MANAGER_VIEW_ID: &str = "queue-manager";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct QueueManagerItem {
    pub(crate) condition_label: String,
    pub(crate) text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct QueueManagerViewParams {
    pub(crate) paused: bool,
    pub(crate) items: Vec<QueueManagerItem>,
    pub(crate) selected_index: Option<usize>,
}

pub(crate) struct QueueManagerView {
    app_event_tx: AppEventSender,
    params: QueueManagerViewParams,
    selected_index: usize,
    complete: Option<ViewCompletion>,
}

impl QueueManagerView {
    pub(crate) fn new(params: QueueManagerViewParams, app_event_tx: AppEventSender) -> Self {
        let selected_index = params
            .selected_index
            .unwrap_or(0)
            .min(params.items.len().saturating_sub(1));
        Self {
            app_event_tx,
            params,
            selected_index,
            complete: None,
        }
    }

    fn selected_item_index(&self) -> Option<usize> {
        (!self.params.items.is_empty()).then_some(self.selected_index)
    }

    fn send_action(&self, action: QueueManagerAction) {
        self.app_event_tx.send(AppEvent::QueueManagerAction(action));
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

    fn lines(&self, width: u16) -> Vec<Line<'static>> {
        let mut lines = vec![Line::from("Queued prompts".bold())];
        if self.params.paused {
            lines.push(Line::from(
                "Paused — queued prompts will not auto-send".dim(),
            ));
        }
        lines.push(Line::from(""));

        if self.params.items.is_empty() {
            lines.push(Line::from("No queued follow-up prompts.".dim()));
        } else {
            let text_width = width.saturating_sub(20).max(8) as usize;
            for (idx, item) in self.params.items.iter().enumerate() {
                let prefix = format!("{}.", idx + 1);
                let condition = format!("{:<10}", item.condition_label);
                let text = Self::truncated_text(item.text.trim(), text_width);
                let mut spans: Vec<Span<'static>> = vec![
                    format!("{prefix:>3} ").into(),
                    condition.dim(),
                    " ".into(),
                    text.into(),
                ];
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
        lines.push(Line::from(vec![
            key_hint::plain(KeyCode::Char('e')).into(),
            " edit  ".dim(),
            key_hint::plain(KeyCode::Char('d')).into(),
            " delete  ".dim(),
            key_hint::plain(KeyCode::Char('p')).into(),
            " pause  ".dim(),
            key_hint::plain(KeyCode::Char('c')).into(),
            " condition".dim(),
        ]));
        lines.push(Line::from(vec![
            key_hint::plain(KeyCode::Enter).into(),
            " send now  ".dim(),
            key_hint::alt(KeyCode::Up).into(),
            " / ".dim(),
            key_hint::alt(KeyCode::Down).into(),
            " reorder  ".dim(),
            key_hint::plain(KeyCode::Esc).into(),
            " close".dim(),
        ]));
        lines
    }
}

impl Renderable for QueueManagerView {
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

impl BottomPaneView for QueueManagerView {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event {
            KeyEvent {
                code: KeyCode::Esc, ..
            } => self.complete = Some(ViewCompletion::Cancelled),
            KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::ALT,
                ..
            } => {
                if let Some(index) = self.selected_item_index() {
                    self.send_action(QueueManagerAction::Move {
                        index,
                        direction: QueueManagerMoveDirection::Up,
                    });
                }
            }
            KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::ALT,
                ..
            } => {
                if let Some(index) = self.selected_item_index() {
                    self.send_action(QueueManagerAction::Move {
                        index,
                        direction: QueueManagerMoveDirection::Down,
                    });
                }
            }
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
            } => {
                if let Some(index) = self.selected_item_index() {
                    self.send_action(QueueManagerAction::SendNow { index });
                }
            }
            KeyEvent {
                code: KeyCode::Char('e'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                if let Some(index) = self.selected_item_index() {
                    self.send_action(QueueManagerAction::Edit { index });
                }
            }
            KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                if let Some(index) = self.selected_item_index() {
                    self.send_action(QueueManagerAction::Delete { index });
                }
            }
            KeyEvent {
                code: KeyCode::Char('p'),
                modifiers: KeyModifiers::NONE,
                ..
            } => self.send_action(QueueManagerAction::TogglePause),
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                if let Some(index) = self.selected_item_index() {
                    self.send_action(QueueManagerAction::CycleCondition { index });
                }
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
        Some(QUEUE_MANAGER_VIEW_ID)
    }

    fn selected_index(&self) -> Option<usize> {
        self.selected_item_index()
    }

    fn on_ctrl_c(&mut self) -> CancellationEvent {
        self.complete = Some(ViewCompletion::Cancelled);
        CancellationEvent::Handled
    }

    fn prefer_esc_to_handle_key_event(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;
    use ratatui::buffer::Buffer;

    #[test]
    fn renders_queue_manager_items_and_hints() {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let tx = crate::app_event_sender::AppEventSender::new(tx);
        let view = QueueManagerView::new(
            QueueManagerViewParams {
                paused: true,
                items: vec![
                    QueueManagerItem {
                        condition_label: "always".to_string(),
                        text: "Depois rode os testes completos".to_string(),
                    },
                    QueueManagerItem {
                        condition_label: "on_failure".to_string(),
                        text: "Pare e explique os erros".to_string(),
                    },
                ],
                selected_index: Some(1),
            },
            tx,
        );
        let mut buffer = Buffer::empty(Rect::new(0, 0, 72, view.desired_height(/*width*/ 72)));
        view.render(buffer.area, &mut buffer);

        assert_snapshot!("queue_manager_view", format!("{buffer:?}"));
    }
}
