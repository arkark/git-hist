use git2::{Blob, Commit, Delta, Oid, Repository};

#[derive(Debug)]
pub struct TurningPoint {
    commit_oid: Oid,
    old_file_oid: Oid,
    new_file_oid: Oid,
    old_path: Option<String>,
    new_path: Option<String>,
    diff_status: Delta,
    index_of_history: Option<usize>,
}

impl TurningPoint {
    pub fn new<S: Into<String>>(
        commit_oid: Oid,
        old_file_oid: Oid,
        new_file_oid: Oid,
        old_path: Option<S>,
        new_path: Option<S>,
        diff_status: Delta,
    ) -> Self {
        Self {
            commit_oid,
            old_file_oid,
            new_file_oid,
            old_path: old_path.map(|path| path.into()),
            new_path: new_path.map(|path| path.into()),
            diff_status,
            index_of_history: None,
        }
    }

    pub fn get_commit<'repo>(&self, repo: &'repo Repository) -> Commit<'repo> {
        repo.find_commit(self.commit_oid).unwrap()
    }

    pub fn get_old_blob<'repo>(&self, repo: &'repo Repository) -> Option<Blob<'repo>> {
        repo.find_blob(self.old_file_oid).ok()
    }

    pub fn get_new_blob<'repo>(&self, repo: &'repo Repository) -> Option<Blob<'repo>> {
        repo.find_blob(self.new_file_oid).ok()
    }

    pub fn old_path(&self) -> Option<&String> {
        self.old_path.as_ref()
    }

    pub fn new_path(&self) -> Option<&String> {
        self.new_path.as_ref()
    }

    pub fn diff_status(&self) -> Delta {
        self.diff_status
    }
}

#[derive(Debug)]
pub struct History {
    points: Vec<TurningPoint>,
}

impl History {
    pub fn new<I: Iterator<Item = TurningPoint>>(points: I) -> Self {
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
