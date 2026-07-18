//! Queued user input and pending-steer state for `ChatWidget`.
//!
//! This module keeps the mutable input queues together so `ChatWidget` can
//! apply UI/protocol effects around a focused reducer-style state bag.

use std::collections::VecDeque;

use super::PendingSteer;
use super::QueueTurnOutcome;
use super::QueuedMessageCondition;
use super::QueuedUserMessage;
use super::UserMessage;
use super::UserMessageHistoryRecord;
use super::user_message_preview_text;

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct QueuedPromptSnapshot {
    pub(super) condition: QueuedMessageCondition,
    pub(super) text: String,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct QueueManagerSnapshot {
    pub(super) paused: bool,
    pub(super) prompts: Vec<QueuedPromptSnapshot>,
}

pub(super) struct QueuedPromptEntry {
    pub(super) message: QueuedUserMessage,
    pub(super) history_record: UserMessageHistoryRecord,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct PendingInputPreview {
    pub(super) queued_messages: Vec<String>,
    pub(super) pending_steers: Vec<String>,
    pub(super) rejected_steers: Vec<String>,
}

#[derive(Debug, Default)]
pub(super) struct InputQueueState {
    /// User inputs queued while a turn is in progress.
    pub(super) queued_user_messages: VecDeque<QueuedUserMessage>,
    /// History records for queued user messages. Slash commands such as `/goal`
    /// can render history that differs from the text submitted to core, so this
    /// stays in lockstep with `queued_user_messages`, with missing entries
    /// treated as user-message text.
    pub(super) queued_user_message_history_records: VecDeque<UserMessageHistoryRecord>,
    /// A user turn has been submitted to core, but `TurnStarted` has not arrived yet.
    pub(super) user_turn_pending_start: bool,
    /// User messages that tried to steer a non-regular turn and must be retried first.
    pub(super) rejected_steers_queue: VecDeque<UserMessage>,
    /// History records for rejected steers. Slash commands such as `/goal` can
    /// render history that differs from the text submitted to core, so this stays
    /// in lockstep with `rejected_steers_queue`, with missing entries treated as
    /// user-message text.
    pub(super) rejected_steer_history_records: VecDeque<UserMessageHistoryRecord>,
    /// Steers already submitted to core but not yet committed into history.
    pub(super) pending_steers: VecDeque<PendingSteer>,
    /// When set, the next interrupt should resubmit all pending steers as one
    /// fresh user turn instead of restoring them into the composer.
    pub(super) submit_pending_steers_after_interrupt: bool,
    /// Durable user-controlled pause for queued prompt auto-send.
    pub(super) user_queue_paused: bool,
    /// Transient suppression used while settings/session state is settling.
    pub(super) suppress_queue_autosend: bool,
}

impl InputQueueState {
    pub(super) fn has_queued_follow_up_messages(&self) -> bool {
        !self.rejected_steers_queue.is_empty() || !self.queued_user_messages.is_empty()
    }

    pub(super) fn manager_snapshot(&self) -> QueueManagerSnapshot {
        let prompts = self
            .queued_user_messages
            .iter()
            .enumerate()
            .map(|(idx, message)| QueuedPromptSnapshot {
                condition: message.condition,
                text: user_message_preview_text(
                    message,
                    self.queued_user_message_history_records.get(idx),
                ),
            })
            .collect();

        QueueManagerSnapshot {
            paused: self.user_queue_paused,
            prompts,
        }
    }

    pub(super) fn set_user_queue_paused(&mut self, paused: bool) {
        self.user_queue_paused = paused;
    }

    pub(super) fn is_queue_autosend_blocked(&self) -> bool {
        self.user_queue_paused || self.suppress_queue_autosend
    }

    pub(super) fn set_autosend_suppressed(&mut self, suppressed: bool) {
        self.suppress_queue_autosend = suppressed;
    }

    pub(super) fn remove_queued_prompt(&mut self, index: usize) -> Option<QueuedPromptEntry> {
        let message = self.queued_user_messages.remove(index)?;
        let history_record = self
            .queued_user_message_history_records
            .remove(index)
            .unwrap_or(UserMessageHistoryRecord::UserMessageText);
        Some(QueuedPromptEntry {
            message,
            history_record,
        })
    }

    pub(super) fn delete_queued_prompt(&mut self, index: usize) -> Option<()> {
        self.remove_queued_prompt(index).map(|_| ())
    }

    pub(super) fn cycle_queued_prompt_condition(&mut self, index: usize) -> Option<()> {
        let message = self.queued_user_messages.get_mut(index)?;
        message.condition = message.condition.next();
        Some(())
    }

    pub(super) fn move_queued_prompt(
        &mut self,
        index: usize,
        direction: QueueMoveDirection,
    ) -> Option<()> {
        let len = self.queued_user_messages.len();
        let target = match direction {
            QueueMoveDirection::Up if index > 0 => index - 1,
            QueueMoveDirection::Down if index + 1 < len => index + 1,
            QueueMoveDirection::Up | QueueMoveDirection::Down => return None,
        };
        self.queued_user_messages.swap(index, target);
        self.queued_user_message_history_records.resize(
            self.queued_user_messages.len(),
            UserMessageHistoryRecord::UserMessageText,
        );
        self.queued_user_message_history_records.swap(index, target);
        Some(())
    }

    pub(super) fn send_queued_prompt_now(&mut self, index: usize) -> Option<QueuedPromptEntry> {
        self.remove_queued_prompt(index)
    }

    pub(super) fn pop_next_queued_user_message_for(
        &mut self,
        outcome: Option<QueueTurnOutcome>,
    ) -> Option<(QueuedUserMessage, UserMessageHistoryRecord)> {
        let index = self.next_eligible_queued_prompt_index(outcome)?;
        self.remove_queued_prompt(index)
            .map(|entry| (entry.message, entry.history_record))
    }

    fn next_eligible_queued_prompt_index(
        &mut self,
        outcome: Option<QueueTurnOutcome>,
    ) -> Option<usize> {
        if let Some(outcome) = outcome {
            let mut idx = 0;
            while idx < self.queued_user_messages.len() {
                let condition = self.queued_user_messages[idx].condition;
                if condition.is_eligible_after(outcome) {
                    idx += 1;
                } else {
                    let _ = self.remove_queued_prompt(idx);
                }
            }
        }

        self.queued_user_messages
            .iter()
            .position(|message| match outcome {
                Some(outcome) => message.condition.is_eligible_after(outcome),
                None => message.condition == QueuedMessageCondition::Always,
            })
    }

    pub(super) fn clear(&mut self) {
        self.queued_user_messages.clear();
        self.queued_user_message_history_records.clear();
        self.user_turn_pending_start = false;
        self.rejected_steers_queue.clear();
        self.rejected_steer_history_records.clear();
        self.pending_steers.clear();
        self.submit_pending_steers_after_interrupt = false;
        self.user_queue_paused = false;
        self.suppress_queue_autosend = false;
    }

    pub(super) fn preview(&self) -> PendingInputPreview {
        let queued_messages = self
            .queued_user_messages
            .iter()
            .enumerate()
            .map(|(idx, message)| {
                user_message_preview_text(
                    message,
                    self.queued_user_message_history_records.get(idx),
                )
            })
            .collect();
        let pending_steers = self
            .pending_steers
            .iter()
            .map(|steer| {
                user_message_preview_text(&steer.user_message, Some(&steer.history_record))
            })
            .collect();
        let rejected_steers = self
            .rejected_steers_queue
            .iter()
            .enumerate()
            .map(|(idx, message)| {
                user_message_preview_text(message, self.rejected_steer_history_records.get(idx))
            })
            .collect();

        PendingInputPreview {
            queued_messages,
            pending_steers,
            rejected_steers,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum QueueMoveDirection {
    Up,
    Down,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn preview_keeps_queue_categories_separate() {
        let mut state = InputQueueState::default();
        state
            .queued_user_messages
            .push_back(UserMessage::from("queued").into());
        state
            .rejected_steers_queue
            .push_back(UserMessage::from("rejected"));
        state.pending_steers.push_back(PendingSteer {
            user_message: UserMessage::from("pending"),
            history_record: UserMessageHistoryRecord::UserMessageText,
            compare_key: crate::chatwidget::user_messages::PendingSteerCompareKey {
                message: "pending".to_string(),
                image_count: 0,
            },
        });

        assert_eq!(
            state.preview(),
            PendingInputPreview {
                queued_messages: vec!["queued".to_string()],
                pending_steers: vec!["pending".to_string()],
                rejected_steers: vec!["rejected".to_string()],
            }
        );
    }

    #[test]
    fn queued_prompt_conditions_cycle_and_filter_by_outcome() {
        let mut state = InputQueueState::default();
        state
            .queued_user_messages
            .push_back(QueuedUserMessage::from(UserMessage::from("success")));
        state
            .queued_user_messages
            .push_back(QueuedUserMessage::from(UserMessage::from("failure")));
        state
            .queued_user_message_history_records
            .resize(2, UserMessageHistoryRecord::UserMessageText);

        state.cycle_queued_prompt_condition(/*index*/ 0);
        state.cycle_queued_prompt_condition(/*index*/ 1);
        state.cycle_queued_prompt_condition(/*index*/ 1);

        let (message, _) = state
            .pop_next_queued_user_message_for(Some(QueueTurnOutcome::Success))
            .expect("success prompt should be eligible");

        assert_eq!(message.text, "success");
        assert!(state.queued_user_messages.is_empty());
    }

    #[test]
    fn queued_prompt_conditions_keep_later_eligible_prompts() {
        let mut state = InputQueueState::default();
        state
            .queued_user_messages
            .push_back(QueuedUserMessage::from(UserMessage::from("success")));
        state
            .queued_user_messages
            .push_back(QueuedUserMessage::from(UserMessage::from("always")));
        state
            .queued_user_messages
            .push_back(QueuedUserMessage::from(UserMessage::from("failure")));
        state
            .queued_user_message_history_records
            .resize(3, UserMessageHistoryRecord::UserMessageText);

        state.cycle_queued_prompt_condition(/*index*/ 0);
        state.cycle_queued_prompt_condition(/*index*/ 2);
        state.cycle_queued_prompt_condition(/*index*/ 2);

        let (message, _) = state
            .pop_next_queued_user_message_for(Some(QueueTurnOutcome::Success))
            .expect("success prompt should be eligible");

        assert_eq!(message.text, "success");
        assert_eq!(state.preview().queued_messages, vec!["always".to_string()]);
    }

    #[test]
    fn queued_prompt_reorder_keeps_history_records_aligned() {
        let mut state = InputQueueState::default();
        state
            .queued_user_messages
            .push_back(QueuedUserMessage::from(UserMessage::from("first")));
        state
            .queued_user_messages
            .push_back(QueuedUserMessage::from(UserMessage::from("second")));
        state
            .queued_user_message_history_records
            .resize(2, UserMessageHistoryRecord::UserMessageText);

        state.move_queued_prompt(/*index*/ 1, QueueMoveDirection::Up);

        assert_eq!(
            state.preview().queued_messages,
            vec!["second".to_string(), "first".to_string()]
        );
    }

    #[test]
    fn user_queue_pause_survives_transient_autosend_suppression_changes() {
        let mut state = InputQueueState::default();
        state.set_user_queue_paused(/*paused*/ true);
        state.set_autosend_suppressed(/*suppressed*/ true);

        state.set_autosend_suppressed(/*suppressed*/ false);

        assert!(state.manager_snapshot().paused);
        assert!(state.is_queue_autosend_blocked());

        state.set_user_queue_paused(/*paused*/ false);

        assert!(!state.manager_snapshot().paused);
        assert!(!state.is_queue_autosend_blocked());
    }

    #[test]
    fn queue_manager_snapshot_includes_pause_and_condition_labels() {
        let mut state = InputQueueState::default();
        state.set_user_queue_paused(/*paused*/ true);
        state
            .queued_user_messages
            .push_back(QueuedUserMessage::from(UserMessage::from("queued")));
        state.cycle_queued_prompt_condition(/*index*/ 0);

        assert_eq!(
            state.manager_snapshot(),
            QueueManagerSnapshot {
                paused: true,
                prompts: vec![QueuedPromptSnapshot {
                    condition: QueuedMessageCondition::OnSuccess,
                    text: "queued".to_string(),
                }],
            }
        );
    }

    #[test]
    fn clear_resets_all_input_queues() {
        let mut state = InputQueueState::default();
        state
            .queued_user_messages
            .push_back(UserMessage::from("queued").into());
        state
            .rejected_steers_queue
            .push_back(UserMessage::from("rejected"));
        state.user_turn_pending_start = true;
        state.submit_pending_steers_after_interrupt = true;
        state.set_user_queue_paused(/*paused*/ true);
        state.set_autosend_suppressed(/*suppressed*/ true);

        state.clear();

        assert!(state.queued_user_messages.is_empty());
        assert!(state.queued_user_message_history_records.is_empty());
        assert!(!state.user_turn_pending_start);
        assert!(state.rejected_steers_queue.is_empty());
        assert!(state.rejected_steer_history_records.is_empty());
        assert!(state.pending_steers.is_empty());
        assert!(!state.submit_pending_steers_after_interrupt);
        assert!(!state.user_queue_paused);
        assert!(!state.suppress_queue_autosend);
    }
}
