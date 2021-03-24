use crate::app::dashboard::COMMIT_INFO_OUTER_HEIGHT;
use git2::{Delta, Oid, Repository};
use once_cell::sync::OnceCell;
use similar::{ChangeTag, TextDiff};
use std::cmp;
use std::convert::TryFrom;

pub struct Diff<'a> {
    status: Delta,
    old_file_oid: Oid,
    new_file_oid: Oid,
    old_path: Option<String>,
    new_path: Option<String>,
    lines: OnceCell<Vec<DiffLine>>,
    repo: &'a Repository,
}

impl<'a> Diff<'a> {
    pub fn new<S: Into<String>>(
        status: Delta,
        old_file_oid: Oid,
        new_file_oid: Oid,
        old_path: Option<S>,
        new_path: Option<S>,
        repo: &'a Repository,
    ) -> Self {
        Self {
            status,
            old_file_oid,
            new_file_oid,
            old_path: old_path.map(|path| path.into()),
            new_path: new_path.map(|path| path.into()),
            lines: OnceCell::new(),
            repo,
        }
    }

    pub fn lines(&'a self) -> &'a Vec<DiffLine> {
        self.lines.get_or_init(|| self.calc_lines())
    }

    fn calc_lines(&self) -> Vec<DiffLine> {
        let old_file_text = self
            .repo
            .find_blob(self.old_file_oid)
            .map(|blob| blob.content().to_vec())
            .unwrap_or_default();
        let new_file_text = self
            .repo
            .find_blob(self.new_file_oid)
            .map(|blob| blob.content().to_vec())
            .unwrap_or_default();
        TextDiff::from_lines(&old_file_text, &new_file_text)
            .iter_all_changes()
            .map(|change| {
                DiffLine::new(
                    change.old_index(),
                    change.new_index(),
                    change.tag(),
                    change.to_string_lossy(),
                )
            })
            .collect::<Vec<_>>()
    }

    pub fn old_path(&self) -> Option<&str> {
        self.old_path.as_deref()
    }

    pub fn new_path(&self) -> Option<&str> {
        self.new_path.as_deref()
    }

    pub fn status(&self) -> Delta {
        self.status
    }

    pub fn max_line_number_len(&self) -> usize {
        self.lines()
            .iter()
            .filter_map(|change| {
                cmp::max(change.old_index, change.new_index).map(|x| {
                    // 0-indexed to 1-indexed
                    x + 1
                })
            })
            .fold(0, cmp::max)
            .to_string()
            .len()
    }

    pub fn allowed_min_index(&self, _terminal_height: usize) -> usize {
        0
    }

    pub fn allowed_max_index(&self, terminal_height: usize) -> usize {
        let terminal_height: isize = isize::try_from(terminal_height).unwrap();
        let commit_info_outer_height: isize = isize::try_from(COMMIT_INFO_OUTER_HEIGHT).unwrap();
        let diff_height: isize = isize::try_from(self.lines().len()).unwrap();

        // TODO:
        //   - option: beyond-last-line (default: false)
        //     - true:  max(0, diff_height - 1)
        //     - false: max(0, diff_height - (terminal_height - commit_info_outer_height))
        usize::try_from(cmp::max(
            0,
            diff_height - (terminal_height - commit_info_outer_height),
        ))
        .unwrap()
    }

    pub fn is_first_index(&self, index: usize, terminal_height: usize) -> bool {
        let lhs = index;
        let rhs = self.allowed_min_index(terminal_height);
        assert!(lhs >= rhs);
        lhs == rhs
    }

    pub fn is_last_index(&self, index: usize, terminal_height: usize) -> bool {
        let lhs = index;
        let rhs = self.allowed_max_index(terminal_height);
        assert!(lhs <= rhs);
        lhs == rhs
    }
}

#[derive(Debug)]
pub struct DiffLine {
    old_index: Option<usize>,
    new_index: Option<usize>,
    tag: ChangeTag,
    text: String,
}

impl DiffLine {
    fn new(
        old_index: Option<usize>,
        new_index: Option<usize>,
        tag: ChangeTag,
        text: impl Into<String>,
    ) -> Self {
        Self {
            old_index,
            new_index,
            tag,
            text: text.into(),
        }
    }

    pub fn old_line_number(&self) -> Option<usize> {
        self.old_index.map(|index| index + 1)
    }

    pub fn new_line_number(&self) -> Option<usize> {
        self.new_index.map(|index| index + 1)
    }

    pub fn sign(&self) -> String {
        match self.tag {
            ChangeTag::Delete => String::from("-"),
            ChangeTag::Insert => String::from("+"),
            ChangeTag::Equal => String::from(" "),
        }
    }

    pub fn text(&self) -> &str {
        self.text.as_str()
    }
}
