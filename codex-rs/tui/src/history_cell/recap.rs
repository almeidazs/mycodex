use ratatui::prelude::*;
use ratatui::style::Stylize;

use super::HistoryCell;
use crate::session_recap::SessionRecapSnapshot;

#[derive(Debug)]
pub(crate) struct SessionRecapCell {
    snapshot: SessionRecapSnapshot,
}

impl SessionRecapCell {
    pub(crate) fn new(snapshot: SessionRecapSnapshot) -> Self {
        Self { snapshot }
    }
}

impl HistoryCell for SessionRecapCell {
    fn display_lines(&self, _width: u16) -> Vec<Line<'static>> {
        let mut lines = vec![Line::from(self.snapshot.title).bold()];
        push_section(&mut lines, "Goal", &self.snapshot.goal);
        push_section(&mut lines, "Completed", &self.snapshot.completed);
        push_section(&mut lines, "Current work", &self.snapshot.current_work);
        push_section(&mut lines, "Problems", &self.snapshot.problems);
        push_section(&mut lines, "Decisions", &self.snapshot.decisions);
        push_section(&mut lines, "Changed files", &self.snapshot.changed_files);
        push_section(&mut lines, "Next steps", &self.snapshot.next_steps);
        lines
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        self.display_lines(u16::MAX)
    }
}

pub(crate) fn new_session_recap(snapshot: SessionRecapSnapshot) -> SessionRecapCell {
    SessionRecapCell::new(snapshot)
}

fn push_section(lines: &mut Vec<Line<'static>>, title: &'static str, items: &[String]) {
    lines.push(Line::from(""));
    lines.push(Line::from(title).dim());
    if items.is_empty() {
        lines.push(Line::from(vec!["  - ".dim(), "None yet".dim()]));
        return;
    }
    for item in items {
        lines.push(Line::from(vec!["  - ".dim(), item.clone().into()]));
    }
}
