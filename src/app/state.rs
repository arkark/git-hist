use crate::app::dashboard::Dashboard;
use crate::app::history::{History, TurningPoint};
use crate::app::terminal::Terminal;
use std::cmp;
use std::convert::TryFrom;

pub struct State<'a> {
    point: &'a TurningPoint<'a>,
    line_index: usize,
    max_line_number_len: usize,
    terminal_height: usize,
}

impl<'a> State<'a> {
    pub fn new(
        point: &'a TurningPoint<'a>,
        line_index: usize,
        max_line_number_len: usize,
        terminal_height: usize,
    ) -> Self {
        Self {
            point,
            line_index,
            max_line_number_len,
            terminal_height,
        }
    }

    pub fn first(history: &'a History<'a>, terminal: &Terminal) -> Self {
        let point = history.latest().unwrap();
        let line_index = 0;
        let max_line_number_len = point.diff().max_line_number_len();
        let terminal_height = terminal.height();
        Self::new(point, line_index, max_line_number_len, terminal_height)
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

    pub fn can_move_up(&self) -> bool {
        self.point
            .diff()
            .can_move_up(self.line_index, self.terminal_height)
    }

    pub fn can_move_down(&self) -> bool {
        self.point
            .diff()
            .can_move_down(self.line_index, self.terminal_height)
    }

    pub fn backward_commit(self, history: &'a History) -> Self {
        if let Some(next_point) = history.backward(self.point) {
            let index_pair = self.point.diff().nearest_old_index_pair(self.line_index);
            let line_index = next_point
                .diff()
                .find_index_from_new_index(index_pair.partial_index())
                .map(|index| {
                    usize::try_from(cmp::max(
                        0,
                        isize::try_from(index).unwrap()
                            - isize::try_from(index_pair.relative_index()).unwrap(),
                    ))
                    .unwrap()
                })
                .unwrap_or(0);
            let max_line_number_len = cmp::max(
                self.max_line_number_len,
                next_point.diff().max_line_number_len(),
            );

            State::new(
                next_point,
                line_index,
                max_line_number_len,
                self.terminal_height,
            )
        } else {
            self
        }
    }

    pub fn forward_commit(self, history: &'a History) -> Self {
        if let Some(next_point) = history.forward(self.point) {
            let index_pair = self.point.diff().nearest_new_index_pair(self.line_index);
            let line_index = next_point
                .diff()
                .find_index_from_old_index(index_pair.partial_index())
                .map(|index| {
                    usize::try_from(cmp::max(
                        0,
                        isize::try_from(index).unwrap()
                            - isize::try_from(index_pair.relative_index()).unwrap(),
                    ))
                    .unwrap()
                })
                .unwrap_or(0);
            let max_line_number_len = cmp::max(
                self.max_line_number_len,
                next_point.diff().max_line_number_len(),
            );

            State::new(
                next_point,
                line_index,
                max_line_number_len,
                self.terminal_height,
            )
        } else {
            self
        }
    }

    pub fn scroll_line_up(self) -> Self {
        if self.can_move_up() {
            let line_index = self.line_index - 1;
            State::new(
                self.point,
                line_index,
                self.max_line_number_len,
                self.terminal_height,
            )
        } else {
            self
        }
    }

    pub fn scroll_line_down(self) -> Self {
        if self.can_move_down() {
            let line_index = self.line_index + 1;
            State::new(
                self.point,
                line_index,
                self.max_line_number_len,
                self.terminal_height,
            )
        } else {
            self
        }
    }

    pub fn scroll_page_up(self) -> Self {
        let diff_height = Dashboard::diff_height(self.terminal_height);

        let line_index = usize::try_from(cmp::min(
            isize::try_from(self.line_index).unwrap(),
            cmp::max(
                isize::try_from(self.line_index).unwrap() - isize::try_from(diff_height).unwrap(),
                isize::try_from(self.point.diff().allowed_min_index(self.terminal_height)).unwrap(),
            ),
        ))
        .unwrap();

        State::new(
            self.point,
            line_index,
            self.max_line_number_len,
            self.terminal_height,
        )
    }

    pub fn scroll_page_down(self) -> Self {
        let diff_height = Dashboard::diff_height(self.terminal_height);

        let line_index = cmp::max(
            self.line_index,
            cmp::min(
                self.line_index + diff_height,
                self.point.diff().allowed_max_index(self.terminal_height),
            ),
        );

        State::new(
            self.point,
            line_index,
            self.max_line_number_len,
            self.terminal_height,
        )
    }

    pub fn scroll_to_top(self) -> Self {
        let line_index = cmp::min(
            self.line_index,
            self.point.diff().allowed_min_index(self.terminal_height),
        );

        State::new(
            self.point,
            line_index,
            self.max_line_number_len,
            self.terminal_height,
        )
    }

    pub fn scroll_to_bottom(self) -> Self {
        let line_index = cmp::max(
            self.line_index,
            self.point.diff().allowed_max_index(self.terminal_height),
        );

        State::new(
            self.point,
            line_index,
            self.max_line_number_len,
            self.terminal_height,
        )
    }

    pub fn terminal_height(&self) -> usize {
        self.terminal_height
    }

    pub fn update_terminal_height(self, terminal_height: usize) -> Self {
        State::new(
            self.point,
            self.line_index,
            self.max_line_number_len,
            terminal_height,
        )
    }
}
