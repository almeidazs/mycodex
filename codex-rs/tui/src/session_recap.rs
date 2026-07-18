use std::collections::HashSet;
use std::collections::VecDeque;
use std::time::Duration;
use std::time::Instant;

const MAX_SECTION_ITEMS: usize = 6;
const MAX_STORED_EVENTS: usize = 120;
const MAX_ITEM_CHARS: usize = 180;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SessionRecapSnapshot {
    pub(crate) title: &'static str,
    pub(crate) goal: Vec<String>,
    pub(crate) completed: Vec<String>,
    pub(crate) current_work: Vec<String>,
    pub(crate) problems: Vec<String>,
    pub(crate) decisions: Vec<String>,
    pub(crate) changed_files: Vec<String>,
    pub(crate) next_steps: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct SessionRecapRuntimeState {
    pub(crate) goal: Option<String>,
    pub(crate) current_work: Vec<String>,
    pub(crate) next_steps: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SessionRecapEventKind {
    Completed,
    Problem,
    Decision,
}

#[derive(Clone, Debug)]
struct SessionRecapEvent {
    kind: SessionRecapEventKind,
    text: String,
    observed_at: Instant,
}

#[derive(Debug)]
pub(crate) struct SessionRecapState {
    events: VecDeque<SessionRecapEvent>,
    changed_files: VecDeque<(String, Instant)>,
    changed_file_set: HashSet<String>,
    last_user_activity_at: Instant,
    last_auto_recap_event_count: usize,
    event_count: usize,
    idle_threshold: Duration,
}

impl SessionRecapState {
    pub(crate) fn new(now: Instant) -> Self {
        Self {
            events: VecDeque::new(),
            changed_files: VecDeque::new(),
            changed_file_set: HashSet::new(),
            last_user_activity_at: now,
            last_auto_recap_event_count: 0,
            event_count: 0,
            idle_threshold: Duration::from_secs(5 * 60),
        }
    }

    pub(crate) fn record_event(&mut self, kind: SessionRecapEventKind, text: impl Into<String>) {
        let text = normalize_item(text.into());
        if text.is_empty() {
            return;
        }
        self.event_count = self.event_count.saturating_add(1);
        self.events.push_back(SessionRecapEvent {
            kind,
            text,
            observed_at: Instant::now(),
        });
        while self.events.len() > MAX_STORED_EVENTS {
            self.events.pop_front();
        }
    }

    pub(crate) fn record_changed_file(&mut self, path: impl Into<String>) {
        let path = normalize_item(path.into());
        if path.is_empty() || !self.changed_file_set.insert(path.clone()) {
            return;
        }
        self.event_count = self.event_count.saturating_add(1);
        self.changed_files.push_back((path.clone(), Instant::now()));
        while self.changed_files.len() > MAX_STORED_EVENTS {
            if let Some((removed, _)) = self.changed_files.pop_front() {
                self.changed_file_set.remove(&removed);
            }
        }
    }

    pub(crate) fn manual_snapshot(
        &self,
        runtime: SessionRecapRuntimeState,
    ) -> SessionRecapSnapshot {
        self.snapshot("Session Recap", runtime, /*since*/ None)
    }

    pub(crate) fn maybe_auto_snapshot(
        &mut self,
        now: Instant,
        runtime: SessionRecapRuntimeState,
    ) -> Option<SessionRecapSnapshot> {
        let away_for = now.saturating_duration_since(self.last_user_activity_at);
        let had_new_events = self.event_count > self.last_auto_recap_event_count;
        let had_events_while_away = self
            .events
            .iter()
            .any(|event| event.observed_at > self.last_user_activity_at)
            || self
                .changed_files
                .iter()
                .any(|(_, observed_at)| *observed_at > self.last_user_activity_at);
        self.last_user_activity_at = now;
        if away_for < self.idle_threshold || !had_new_events || !had_events_while_away {
            return None;
        }
        self.last_auto_recap_event_count = self.event_count;
        Some(self.snapshot("While you were away", runtime, Some(now - away_for)))
    }

    fn snapshot(
        &self,
        title: &'static str,
        runtime: SessionRecapRuntimeState,
        since: Option<Instant>,
    ) -> SessionRecapSnapshot {
        SessionRecapSnapshot {
            title,
            goal: runtime.goal.into_iter().collect(),
            completed: self.recent_events(SessionRecapEventKind::Completed, since),
            current_work: limit_items(runtime.current_work),
            problems: self.recent_events(SessionRecapEventKind::Problem, since),
            decisions: self.recent_events(SessionRecapEventKind::Decision, since),
            changed_files: limit_items(
                self.changed_files
                    .iter()
                    .rev()
                    .filter(|(_, observed_at)| since.is_none_or(|since| *observed_at >= since))
                    .map(|(path, _)| path.clone())
                    .collect(),
            ),
            next_steps: limit_items(runtime.next_steps),
        }
    }

    fn recent_events(&self, kind: SessionRecapEventKind, since: Option<Instant>) -> Vec<String> {
        limit_items(
            self.events
                .iter()
                .rev()
                .filter(|event| event.kind == kind)
                .filter(|event| since.is_none_or(|since| event.observed_at >= since))
                .map(|event| event.text.clone())
                .collect(),
        )
    }
}

fn limit_items(items: Vec<String>) -> Vec<String> {
    items
        .into_iter()
        .filter(|item| !item.trim().is_empty())
        .take(MAX_SECTION_ITEMS)
        .collect()
}

fn normalize_item(mut text: String) -> String {
    text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() > MAX_ITEM_CHARS {
        let truncated = text.chars().take(MAX_ITEM_CHARS).collect::<String>();
        format!("{truncated}…")
    } else {
        text
    }
}
