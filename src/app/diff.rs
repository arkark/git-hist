use crate::app::dashboard::Dashboard;
use git2::{Delta, Oid, Repository};
use once_cell::sync::OnceCell;
use similar::{ChangeTag, TextDiff};
use std::convert::TryFrom;
use std::{cmp, ops::Deref};
use tui::style::{Color, Style};

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
        // TODO: show "binary file" if the file is a binary file
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
        let text_diff = TextDiff::from_lines(&old_file_text, &new_file_text);
        text_diff
            .ops()
            .iter()
            .map(|op| {
                text_diff.iter_inline_changes(op).map(|change| {
                    let parts = change
                        .iter_strings_lossy()
                        .map(|(emphasized, text)| DiffLinePart::new(text, emphasized))
                        .collect();
                    DiffLine::new(change.old_index(), change.new_index(), change.tag(), parts)
                })
            })
            .flatten()
            .enumerate()
            .map(|(index, mut line)| {
                line.index = index;
                line
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
        let diff_length = isize::try_from(self.lines().len()).unwrap();
        let diff_height = isize::try_from(Dashboard::diff_height(terminal_height)).unwrap();

        if diff_length == 0 {
            0
        } else {
            // TODO:
            //   - option: beyond-last-line (default: false)
            //     - whether the diff view will scroll beyond the last line
            //     - true:  diff_length - 1
            //     - false: diff_length - diff_height
            usize::try_from((diff_length - diff_height).clamp(0, diff_length - 1)).unwrap()
        }
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
    parts: Vec<DiffLinePart>,
}

impl DiffLine {
    fn new(
        old_index: Option<usize>,
        new_index: Option<usize>,
        tag: ChangeTag,
        parts: Vec<DiffLinePart>,
    ) -> Self {
        Self {
            index: 0,
            old_index,
            new_index,
            tag,
            parts,
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

    pub fn style(&self) -> Style {
        match self.tag {
            ChangeTag::Delete => Style::default().fg(Color::Red),
            ChangeTag::Insert => Style::default().fg(Color::Green),
            ChangeTag::Equal => Style::default(),
        }
    }

    pub fn parts(&self) -> &Vec<DiffLinePart> {
        &self.parts
    }
}

#[derive(Debug)]
pub struct DiffLinePart {
    text: String,
    emphasized: bool,
}

impl DiffLinePart {
    pub fn new(text: impl Into<String>, emphasized: bool) -> Self {
        Self {
            text: text.into(),
            emphasized,
        }
    }

    pub fn text(&self) -> &str {
        self.text.deref()
    }

    pub fn emphasize(&self, style: Style) -> Style {
        if self.emphasized {
            style.bg(Color::DarkGray)
        } else {
            style
        }
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
