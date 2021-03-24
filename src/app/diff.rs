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
            .enumerate()
            .map(|(index, change)| {
                DiffLine::new(
                    index,
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
        //     - whether the diff view will scroll beyond the last line
        //     - true:  max(0, diff_height - 1)
        //     - false: max(0, diff_height - (terminal_height - commit_info_outer_height))
        usize::try_from(cmp::max(
            0,
            diff_height - (terminal_height - commit_info_outer_height),
        ))
        .unwrap()
    }

    pub fn can_move_up(&self, index: usize, terminal_height: usize) -> bool {
        index > self.allowed_min_index(terminal_height)
    }

    pub fn can_move_down(&self, index: usize, terminal_height: usize) -> bool {
        index < self.allowed_max_index(terminal_height)
    }

    pub fn nearest_old_index_pair(&self, index: usize) -> IndexPair {
        if let Some(line) = self
            .lines()
            .iter()
            .skip(index)
            .find(|line| line.old_index.is_some())
        {
            assert!(line.index >= index);
            IndexPair::new(line.index - index, line.old_index.unwrap())
        } else if let Some(line) = self
            .lines()
            .iter()
            .take(index)
            .rev()
            .find(|line| line.old_index.is_some())
        {
            assert!(line.index < index);
            IndexPair::new(0, line.old_index.unwrap())
        } else {
            IndexPair::new(0, 0)
        }
    }

    pub fn nearest_new_index_pair(&self, index: usize) -> IndexPair {
        if let Some(line) = self
            .lines()
            .iter()
            .skip(index)
            .find(|line| line.new_index.is_some())
        {
            assert!(line.index >= index);
            IndexPair::new(line.index - index, line.new_index.unwrap())
        } else if let Some(line) = self
            .lines()
            .iter()
            .take(index)
            .rev()
            .find(|line| line.new_index.is_some())
        {
            assert!(line.index < index);
            IndexPair::new(0, line.new_index.unwrap())
        } else {
            IndexPair::new(0, 0)
        }
    }

    pub fn find_index_from_old_index(&self, old_index: usize) -> Option<usize> {
        self.lines()
            .iter()
            .find(|line| {
                line.old_index
                    // use https://doc.rust-lang.org/std/option/enum.Option.html#method.contains in the future
                    .filter(|i| *i == old_index)
                    .is_some()
            })
            .map(|line| line.index)
    }

    pub fn find_index_from_new_index(&self, new_index: usize) -> Option<usize> {
        self.lines()
            .iter()
            .find(|line| {
                line.new_index
                    // use https://doc.rust-lang.org/std/option/enum.Option.html#method.contains in the future
                    .filter(|i| *i == new_index)
                    .is_some()
            })
            .map(|line| line.index)
    }
}

#[derive(Debug)]
pub struct DiffLine {
    index: usize,
    old_index: Option<usize>,
    new_index: Option<usize>,
    tag: ChangeTag,
    text: String,
}

impl DiffLine {
    fn new(
        index: usize,
        old_index: Option<usize>,
        new_index: Option<usize>,
        tag: ChangeTag,
        text: impl Into<String>,
    ) -> Self {
        Self {
            index,
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

#[derive(Debug)]
pub struct IndexPair {
    relative_index: usize, // an index from the top of a diff shown in a terminal
    partial_index: usize,  // old_index or new_index
}

impl IndexPair {
    pub fn new(relative_index: usize, partial_index: usize) -> Self {
        Self {
            relative_index,
            partial_index,
        }
    }

    pub fn relative_index(&self) -> usize {
        self.relative_index
    }

    pub fn partial_index(&self) -> usize {
        self.partial_index
    }
}
