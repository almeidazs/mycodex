use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use super::*;
use codex_app_server_protocol::ThreadBackgroundTerminal;
use codex_app_server_protocol::ThreadBackgroundTerminalStatus;

use crate::bottom_pane::PROCESS_MANAGER_VIEW_ID;
use crate::bottom_pane::ProcessManagerItem;
use crate::bottom_pane::ProcessManagerStatus;
use crate::bottom_pane::ProcessManagerView;
use crate::bottom_pane::ProcessManagerViewParams;

impl ChatWidget {
    pub(crate) fn open_process_manager(
        &mut self,
        processes: Vec<ThreadBackgroundTerminal>,
        selected_index: Option<usize>,
        output_process_id: Option<String>,
        follow_output: bool,
    ) {
        self.bottom_pane
            .dismiss_active_view_if_id(PROCESS_MANAGER_VIEW_ID);
        let items = processes.into_iter().map(process_manager_item).collect();
        let params = ProcessManagerViewParams {
            items,
            selected_index,
            output_process_id,
            follow_output,
        };
        self.bottom_pane.show_view(Box::new(ProcessManagerView::new(
            params,
            self.app_event_tx.clone(),
        )));
        self.request_redraw();
    }

    pub(crate) fn process_manager_selected_index(&self) -> Option<usize> {
        self.bottom_pane
            .selected_index_for_active_view(PROCESS_MANAGER_VIEW_ID)
    }

    pub(crate) fn process_manager_follow_target(&self) -> Option<String> {
        self.bottom_pane.active_process_manager_follow_target()
    }

    pub(crate) fn copy_process_command_to_clipboard(&mut self, command: &str) {
        match crate::clipboard_copy::copy_to_clipboard(command) {
            Ok(lease) => {
                self.clipboard_lease = lease;
                self.add_info_message("Copied process command to clipboard.".to_string(), None);
            }
            Err(err) => {
                self.add_error_message(format!("Failed to copy process command: {err}"));
            }
        }
    }
}

fn process_manager_item(process: ThreadBackgroundTerminal) -> ProcessManagerItem {
    let url = process.detected_urls.first().cloned();
    ProcessManagerItem {
        process_id: process.process_id,
        name: process_name(&process.command),
        command: process.command,
        cwd: process.cwd.as_path().display().to_string(),
        elapsed: elapsed_label(process.started_at, process.ended_at),
        url,
        status: match process.status {
            ThreadBackgroundTerminalStatus::Running => ProcessManagerStatus::Running,
            ThreadBackgroundTerminalStatus::Exited => ProcessManagerStatus::Exited {
                exit_code: process.exit_code,
            },
        },
        recent_output: process.recent_output,
    }
}

fn process_name(command: &str) -> String {
    let lower = command.to_ascii_lowercase();
    if lower.contains("migrate") {
        return "migration".to_string();
    }
    if lower.contains("test") && lower.contains("watch") {
        return "tests-watch".to_string();
    }
    if lower.contains("dev") || lower.contains("serve") || lower.contains("start") {
        return "dev-server".to_string();
    }
    command
        .split_whitespace()
        .next()
        .map(|token| token.trim_matches(|ch| ch == '\'' || ch == '"').to_string())
        .filter(|token| !token.is_empty())
        .unwrap_or_else(|| "process".to_string())
}

fn elapsed_label(started_at: i64, ended_at: Option<i64>) -> String {
    let end = ended_at.unwrap_or_else(current_unix_seconds);
    let elapsed = end.saturating_sub(started_at).max(0);
    let minutes = elapsed / 60;
    let seconds = elapsed % 60;
    if minutes >= 60 {
        let hours = minutes / 60;
        let minutes = minutes % 60;
        format!("{hours:02}:{minutes:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

fn current_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}
