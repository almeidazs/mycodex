use super::input_queue::QueuedPromptEntry;
use super::*;
use crate::app_event::QueueManagerAction;
use crate::app_event::QueueManagerMoveDirection;
use crate::bottom_pane::QUEUE_MANAGER_VIEW_ID;
use crate::bottom_pane::QueueManagerItem;
use crate::bottom_pane::QueueManagerView;
use crate::bottom_pane::QueueManagerViewParams;

impl ChatWidget {
    pub(crate) fn open_queue_manager(&mut self) {
        let params = self.queue_manager_view_params(/*selected_index*/ None);
        self.bottom_pane.show_view(Box::new(QueueManagerView::new(
            params,
            self.app_event_tx.clone(),
        )));
        self.request_redraw();
    }

    pub(crate) fn handle_queue_manager_action(&mut self, action: QueueManagerAction) {
        match action {
            QueueManagerAction::SendNow { index } => {
                self.recap_decision(format!("Sent queued prompt {} now", index + 1));
                self.dismiss_queue_manager_view();
                self.send_queued_prompt_now(index);
            }
            QueueManagerAction::Edit { index } => {
                self.recap_decision(format!("Edited queued prompt {}", index + 1));
                self.dismiss_queue_manager_view();
                self.edit_queued_prompt(index);
            }
            QueueManagerAction::Delete { index } => {
                if self.input_queue.delete_queued_prompt(index).is_some() {
                    self.recap_decision(format!("Deleted queued prompt {}", index + 1));
                    self.refresh_pending_input_preview();
                    self.refresh_queue_manager_view(Some(index));
                }
            }
            QueueManagerAction::TogglePause => {
                let paused = !self.input_queue.user_queue_paused;
                self.recap_decision(if paused {
                    "Paused queued prompts".to_string()
                } else {
                    "Resumed queued prompts".to_string()
                });
                self.input_queue.set_user_queue_paused(paused);
                if !paused {
                    self.maybe_send_next_queued_input();
                }
                self.refresh_pending_input_preview();
                self.refresh_queue_manager_view(self.queue_manager_selected_index());
            }
            QueueManagerAction::CycleCondition { index } => {
                if self
                    .input_queue
                    .cycle_queued_prompt_condition(index)
                    .is_some()
                {
                    self.recap_decision(format!(
                        "Changed condition for queued prompt {}",
                        index + 1
                    ));
                    self.refresh_pending_input_preview();
                    self.refresh_queue_manager_view(Some(index));
                }
            }
            QueueManagerAction::Move { index, direction } => {
                let direction = match direction {
                    QueueManagerMoveDirection::Up => input_queue::QueueMoveDirection::Up,
                    QueueManagerMoveDirection::Down => input_queue::QueueMoveDirection::Down,
                };
                if self
                    .input_queue
                    .move_queued_prompt(index, direction)
                    .is_some()
                {
                    self.recap_decision(format!("Reordered queued prompt {}", index + 1));
                    let selected = match direction {
                        input_queue::QueueMoveDirection::Up => index.saturating_sub(1),
                        input_queue::QueueMoveDirection::Down => (index + 1).min(
                            self.input_queue
                                .queued_user_messages
                                .len()
                                .saturating_sub(1),
                        ),
                    };
                    self.refresh_pending_input_preview();
                    self.refresh_queue_manager_view(Some(selected));
                }
            }
        }
        self.request_redraw();
    }

    fn queue_manager_view_params(&self, selected_index: Option<usize>) -> QueueManagerViewParams {
        let snapshot = self.input_queue.manager_snapshot();
        let items = snapshot
            .prompts
            .into_iter()
            .map(|prompt| QueueManagerItem {
                condition_label: prompt.condition.label().to_string(),
                text: prompt.text,
            })
            .collect();
        QueueManagerViewParams {
            paused: snapshot.paused,
            items,
            selected_index,
        }
    }

    fn queue_manager_selected_index(&self) -> Option<usize> {
        self.bottom_pane
            .selected_index_for_active_view(QUEUE_MANAGER_VIEW_ID)
    }

    fn refresh_queue_manager_view(&mut self, selected_index: Option<usize>) {
        if !self
            .bottom_pane
            .dismiss_active_view_if_id(QUEUE_MANAGER_VIEW_ID)
        {
            return;
        }
        self.bottom_pane.show_view(Box::new(QueueManagerView::new(
            self.queue_manager_view_params(selected_index),
            self.app_event_tx.clone(),
        )));
    }

    fn dismiss_queue_manager_view(&mut self) {
        self.bottom_pane
            .dismiss_active_view_if_id(QUEUE_MANAGER_VIEW_ID);
    }

    fn edit_queued_prompt(&mut self, index: usize) {
        let Some(entry) = self.input_queue.remove_queued_prompt(index) else {
            return;
        };
        let QueuedPromptEntry {
            message,
            history_record,
        } = entry;
        let QueuedUserMessage {
            user_message,
            pending_pastes,
            ..
        } = message;
        let composer = Self::composer_state_from_user_message(
            user_message_for_restore(user_message, &history_record),
            pending_pastes,
        );
        self.restore_composer_state(composer);
        self.refresh_pending_input_preview();
    }

    fn send_queued_prompt_now(&mut self, index: usize) {
        if self.is_user_turn_pending_or_running() || self.blocks_direct_input {
            self.refresh_pending_input_preview();
            return;
        }
        let Some(result) = self.input_queue.send_queued_prompt_now(index) else {
            return;
        };

        let QueuedPromptEntry {
            message,
            history_record,
        } = result;
        self.submit_queued_prompt_entry(message, history_record);
        self.refresh_pending_input_preview();
    }

    fn submit_queued_prompt_entry(
        &mut self,
        queued_message: QueuedUserMessage,
        history_record: UserMessageHistoryRecord,
    ) {
        match queued_message.action {
            QueuedInputAction::Plain => {
                self.submit_user_message_with_history_record(
                    queued_message.into_user_message(),
                    history_record,
                );
            }
            QueuedInputAction::ParseSlash => {
                self.submit_queued_slash_prompt(queued_message);
            }
            QueuedInputAction::RunShell => {
                self.submit_queued_shell_prompt(queued_message.into_user_message());
            }
        }
    }
}
