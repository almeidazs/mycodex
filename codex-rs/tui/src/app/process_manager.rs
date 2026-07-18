use super::*;
use crate::app_event::ProcessManagerAction;

const PROCESS_MANAGER_FOLLOW_INTERVAL: Duration = Duration::from_secs(/*secs*/ 1);

impl App {
    pub(super) async fn open_process_manager(
        &mut self,
        app_server: &mut AppServerSession,
        selected_index: Option<usize>,
        output_process_id: Option<String>,
        follow_output: bool,
    ) {
        let Some(thread_id) = self.current_displayed_thread_id() else {
            self.chat_widget.add_error_message(
                "Processes are unavailable before the session starts.".to_string(),
            );
            return;
        };
        match app_server.thread_background_terminals_list(thread_id).await {
            Ok(processes) => {
                let follow_target = output_process_id.clone().filter(|_| follow_output);
                self.chat_widget.open_process_manager(
                    processes,
                    selected_index,
                    output_process_id,
                    follow_output,
                );
                if let Some(process_id) = follow_target {
                    self.schedule_process_manager_follow_tick(process_id);
                }
            }
            Err(err) => {
                self.chat_widget
                    .add_error_message(format!("Failed to load processes: {err}"));
            }
        }
    }

    pub(super) async fn handle_process_manager_action(
        &mut self,
        app_server: &mut AppServerSession,
        action: ProcessManagerAction,
    ) {
        match action {
            ProcessManagerAction::ViewOutput { process_id } => {
                let selected_index = self.chat_widget.process_manager_selected_index();
                self.open_process_manager(app_server, selected_index, Some(process_id), false)
                    .await;
            }
            ProcessManagerAction::FollowLogs { process_id } => {
                let selected_index = self.chat_widget.process_manager_selected_index();
                self.open_process_manager(app_server, selected_index, Some(process_id), true)
                    .await;
            }
            ProcessManagerAction::FollowTick { process_id } => {
                if self.chat_widget.process_manager_follow_target().as_deref() != Some(&process_id)
                {
                    return;
                }
                let selected_index = self.chat_widget.process_manager_selected_index();
                self.open_process_manager(app_server, selected_index, Some(process_id), true)
                    .await;
            }
            ProcessManagerAction::Kill { process_id } => {
                let selected_index = self.chat_widget.process_manager_selected_index();
                self.terminate_process(app_server, process_id).await;
                self.open_process_manager(app_server, selected_index, None, false)
                    .await;
            }
            ProcessManagerAction::Restart {
                process_id,
                command,
                cwd,
            } => {
                let selected_index = self.chat_widget.process_manager_selected_index();
                self.terminate_process(app_server, process_id).await;
                self.restart_process(app_server, command, cwd).await;
                self.open_process_manager(app_server, selected_index, None, false)
                    .await;
            }
            ProcessManagerAction::OpenUrl { url } => {
                self.open_url_in_browser(url);
            }
            ProcessManagerAction::CopyCommand { command } => {
                self.chat_widget.copy_process_command_to_clipboard(&command);
            }
        }
    }

    async fn terminate_process(&mut self, app_server: &mut AppServerSession, process_id: String) {
        let Some(thread_id) = self.current_displayed_thread_id() else {
            self.chat_widget
                .add_error_message("Cannot stop a process before the session starts.".to_string());
            return;
        };
        match app_server
            .thread_background_terminal_terminate(thread_id, process_id)
            .await
        {
            Ok(true) => self
                .chat_widget
                .add_info_message("Stopped process.".to_string(), None),
            Ok(false) => self
                .chat_widget
                .add_error_message("Process was already gone.".to_string()),
            Err(err) => self
                .chat_widget
                .add_error_message(format!("Failed to stop process: {err}")),
        }
    }

    async fn restart_process(
        &mut self,
        app_server: &mut AppServerSession,
        command: String,
        cwd: String,
    ) {
        let Some(thread_id) = self.current_displayed_thread_id() else {
            self.chat_widget.add_error_message(
                "Cannot restart a process before the session starts.".to_string(),
            );
            return;
        };
        let restart_command = restart_command_for_cwd(&command, &cwd);
        match app_server
            .thread_shell_command(thread_id, restart_command)
            .await
        {
            Ok(()) => self
                .chat_widget
                .add_info_message(format!("Restarted process: {command}"), None),
            Err(err) => self
                .chat_widget
                .add_error_message(format!("Failed to restart process: {err}")),
        }
    }

    fn schedule_process_manager_follow_tick(&self, process_id: String) {
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            tokio::time::sleep(PROCESS_MANAGER_FOLLOW_INTERVAL).await;
            tx.send(AppEvent::ProcessManagerAction(
                ProcessManagerAction::FollowTick { process_id },
            ));
        });
    }
}

fn restart_command_for_cwd(command: &str, cwd: &str) -> String {
    if cfg!(windows) {
        format!("cd /d \"{}\" && {command}", cwd.replace('"', "\\\""))
    } else {
        format!("cd {} && {command}", shell_quote(cwd))
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
