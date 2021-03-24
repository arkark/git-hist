use git2::{Delta, Oid, Repository};
use once_cell::sync::OnceCell;
use similar::{ChangeTag, TextDiff};
use std::cmp;

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
