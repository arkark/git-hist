use std::cmp;

use crate::app::history::{History, TurningPoint};
use crate::app::terminal::Terminal;

pub struct State<'a> {
    point: &'a TurningPoint<'a>,
    line_index: usize,
    max_line_number_len: usize,
    is_latest_commit: bool,
    is_earliest_commit: bool,
    terminal_height: usize,
}

impl<'a> State<'a> {
    pub fn new(history: &'a History<'a>, terminal: &Terminal) -> Self {
        let point = history.latest().unwrap();
        let line_index = 0;
        let max_line_number_len = point.max_line_number_len();
        let is_latest_commit = history.forward(point).is_none();
        let is_earliest_commit = history.backward(point).is_none();
        let terminal_height = terminal.height();
        Self {
            point,
            line_index,
            max_line_number_len,
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

    pub fn max_line_number_len(&self) -> usize {
        self.max_line_number_len
    }

    pub fn is_latest_commit(&self) -> bool {
        self.is_latest_commit
    }

    pub fn is_earliest_commit(&self) -> bool {
        self.is_earliest_commit
    }

    pub fn can_move_up(&self) -> bool {
        self.point
            .can_move_up(self.line_index, self.terminal_height)
    }

    pub fn can_move_down(&self) -> bool {
        self.point
            .can_move_down(self.line_index, self.terminal_height)
    }

    pub fn backward_commit(mut self, history: &'a History) -> Self {
        if let Some(next_point) = history.backward(self.point) {
            let index_pair = self.point.nearest_old_index_pair(self.line_index);
            let line_index = next_point
                .find_index_from_new_index(index_pair.partial_index())
                .map(|index| index + index_pair.relative_index())
                .unwrap_or(0);
            let max_line_number_len =
                cmp::max(self.max_line_number_len, next_point.max_line_number_len());

            self.point = next_point;
            self.line_index = line_index;
            self.max_line_number_len = max_line_number_len;
        }
        self
    }

    pub fn forward_commit(mut self, history: &'a History) -> Self {
        if let Some(next_point) = history.forward(self.point) {
            let index_pair = self.point.nearest_new_index_pair(self.line_index);
            let line_index = next_point
                .find_index_from_old_index(index_pair.partial_index())
                .map(|index| index + index_pair.relative_index())
                .unwrap_or(0);
            let max_line_number_len =
                cmp::max(self.max_line_number_len, next_point.max_line_number_len());

            self.point = next_point;
            self.line_index = line_index;
            self.max_line_number_len = max_line_number_len;
        }
        self
    }

    pub fn decrement_line_index(mut self) -> Self {
        if self.can_move_up() {
            self.line_index -= 1;
        }
        self
    }

    pub fn increment_line_index(mut self) -> Self {
        if self.can_move_down() {
            self.line_index += 1;
        }
        self
    }

    pub fn update_terminal_height(mut self, terminal_height: usize) -> Self {
        self.terminal_height = terminal_height;
        self
    }
}
