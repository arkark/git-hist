use crate::app::diff::{Diff, DiffLine};
use git2::{Commit, Delta, Oid, Repository};
use std::slice::Iter;

pub struct TurningPoint<'a> {
    commit_oid: Oid,
    diff: Diff<'a>,
    index_of_history: Option<usize>,
}

impl<'a> TurningPoint<'a> {
    pub fn new(commit_oid: Oid, diff: Diff<'a>) -> Self {
        Self {
            commit_oid,
            diff,
            index_of_history: None,
        }
    }

    pub fn get_commit<'repo>(&self, repo: &'repo Repository) -> Commit<'repo> {
        repo.find_commit(self.commit_oid).unwrap()
    }

    pub fn old_path(&self) -> Option<&str> {
        self.diff.old_path()
    }

    pub fn new_path(&self) -> Option<&str> {
        self.diff.new_path()
    }

    pub fn diff_status(&self) -> Delta {
        self.diff.status()
    }

    pub fn max_line_number_len(&self) -> usize {
        self.diff.max_line_number_len()
    }

    pub fn iter_diff_lines(&self) -> Iter<'_, DiffLine> {
        self.diff.lines().iter()
    }
}

pub struct History<'a> {
    points: Vec<TurningPoint<'a>>,
}

impl<'a> History<'a> {
    pub fn new<I: Iterator<Item = TurningPoint<'a>>>(points: I) -> Self {
        History {
            points: points
                .enumerate()
                .map(|(i, mut p)| {
                    p.index_of_history = Some(i);
                    p
                })
                .collect::<Vec<_>>(),
        }
    }

    pub fn latest(&self) -> Option<&TurningPoint> {
        self.points.first()
    }

    pub fn backward(&self, point: &TurningPoint) -> Option<&TurningPoint> {
        point
            .index_of_history
            .and_then(|i| i.checked_add(1))
            .and_then(|i| self.points.get(i))
    }

    pub fn forward(&self, point: &TurningPoint) -> Option<&TurningPoint> {
        point
            .index_of_history
            .and_then(|i| i.checked_sub(1))
            .and_then(|i| self.points.get(i))
    }
}