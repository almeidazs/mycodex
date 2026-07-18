use std::time::Instant;

use super::*;
use crate::session_recap::SessionRecapEventKind;
use crate::session_recap::SessionRecapRuntimeState;

impl ChatWidget {
    pub(crate) fn add_session_recap_output(&mut self) {
        let snapshot = self
            .session_recap
            .manual_snapshot(self.session_recap_runtime_state());
        self.add_to_history(history_cell::new_session_recap(snapshot));
        self.request_redraw();
    }

    pub(crate) fn maybe_add_away_session_recap(&mut self, now: Instant) {
        if !self.no_modal_or_popup_active() {
            return;
        }
        let runtime = self.session_recap_runtime_state();
        if let Some(snapshot) = self.session_recap.maybe_auto_snapshot(now, runtime) {
            self.add_to_history(history_cell::new_session_recap(snapshot));
            self.request_redraw();
        }
    }

    pub(super) fn recap_completed(&mut self, text: impl Into<String>) {
        self.session_recap
            .record_event(SessionRecapEventKind::Completed, text);
    }

    pub(super) fn recap_problem(&mut self, text: impl Into<String>) {
        self.session_recap
            .record_event(SessionRecapEventKind::Problem, text);
    }

    pub(super) fn recap_decision(&mut self, text: impl Into<String>) {
        self.session_recap
            .record_event(SessionRecapEventKind::Decision, text);
    }

    pub(super) fn recap_changed_file(&mut self, path: impl Into<String>) {
        self.session_recap.record_changed_file(path);
    }

    fn session_recap_runtime_state(&self) -> SessionRecapRuntimeState {
        let mut current_work = Vec::new();
        if self.bottom_pane.is_task_running() {
            current_work.push("Agent turn is currently running".to_string());
        }
        current_work.extend(
            self.running_commands
                .values()
                .map(|command| format!("Running {}", command.command.join(" "))),
        );
        current_work.extend(
            self.unified_exec_processes
                .iter()
                .map(|process| format!("Background process {}", process.command_display)),
        );

        let queue_snapshot = self.input_queue.manager_snapshot();
        let next_steps = queue_snapshot
            .prompts
            .into_iter()
            .map(|prompt| {
                let condition = prompt.condition.label();
                if condition == "always" {
                    prompt.text
                } else {
                    format!("{condition}: {}", prompt.text)
                }
            })
            .collect();

        SessionRecapRuntimeState {
            goal: self
                .current_goal_status
                .as_ref()
                .map(|status| status.recap_summary()),
            current_work,
            next_steps,
        }
    }
}
