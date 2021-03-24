use super::history::{History, TurningPoint};

pub struct State<'a> {
    point: &'a TurningPoint<'a>,
    line_index: usize,
    is_latest_commit: bool,
    is_earliest_commit: bool,
}

impl<'a> State<'a> {
    pub fn new(
        point: &'a TurningPoint,
        line_index: usize,
        is_latest_commit: bool,
        is_earliest_commit: bool,
    ) -> Self {
        Self {
            point,
            line_index,
            is_latest_commit,
            is_earliest_commit,
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
}

impl<'a> From<&'a History<'a>> for State<'a> {
    fn from(history: &'a History) -> State<'a> {
        let point = history.latest().unwrap();
        let line_index = 0;
        let is_latest_commit = history.forward(point).is_none();
        let is_earliest_commit = history.backward(point).is_none();
        State::new(point, line_index, is_latest_commit, is_earliest_commit)
    }
}
