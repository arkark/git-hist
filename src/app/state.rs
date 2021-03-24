use crate::app::history::{History, TurningPoint};
use crate::app::terminal::Terminal;

pub struct State<'a> {
    point: &'a TurningPoint<'a>,
    line_index: usize,
    is_latest_commit: bool,
    is_earliest_commit: bool,
    terminal_height: usize,
}

impl<'a> State<'a> {
    pub fn new(history: &'a History<'a>, terminal: &Terminal) -> Self {
        let point = history.latest().unwrap();
        let line_index = 0;
        let is_latest_commit = history.forward(point).is_none();
        let is_earliest_commit = history.backward(point).is_none();
        let terminal_height = terminal.height();
        Self {
            point,
            line_index,
            is_latest_commit,
            is_earliest_commit,
            terminal_height,
        }
    }

    pub fn point(&self) -> &TurningPoint {
        self.point
    }

    pub fn line_index(&self) -> usize {
        self.line_index
    }

    pub fn is_latest_commit(&self) -> bool {
        self.is_latest_commit
    }

    pub fn is_earliest_commit(&self) -> bool {
        self.is_earliest_commit
    }

    pub fn is_first_line_index(&self) -> bool {
        self.point
            .is_first_index(self.line_index, self.terminal_height)
    }

    pub fn is_last_line_index(&self) -> bool {
        self.point
            .is_last_index(self.line_index, self.terminal_height)
    }

    pub fn backward_commit(mut self, history: &'a History) -> Self {
        if let Some(point) = history.backward(self.point) {
            self.point = point;
            self.line_index = 0; // TODO
        }
        self
    }

    pub fn forward_commit(mut self, history: &'a History) -> Self {
        if let Some(point) = history.forward(self.point) {
            self.point = point;
            self.line_index = 0; // TODO
        }
        self
    }

    pub fn decrement_line_index(mut self) -> Self {
        if !self.is_first_line_index() {
            self.line_index -= 1;
        }
        self
    }

    pub fn increment_line_index(mut self) -> Self {
        if !self.is_last_line_index() {
            self.line_index += 1;
        }
        self
    }

    pub fn update_terminal_height(mut self, terminal_height: usize) -> Self {
        self.terminal_height = terminal_height;
        self.line_index = self.line_index.clamp(
            self.point.allowed_min_index(terminal_height),
            self.point.allowed_max_index(terminal_height),
        );
        self
    }
}
