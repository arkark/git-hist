use crate::app::dashboard::Dashboard;
use crate::app::state::State;
use crate::args::Args;
use git2::{Delta, DiffDelta, Oid, Repository};
use once_cell::sync::OnceCell;
use similar::{ChangeTag, TextDiff};
use std::{cmp, ops::Deref};
use tui::style::{Color, Style};

pub struct Diff<'a> {
    status: Delta,
    old_file_oid: Oid,
    new_file_oid: Oid,
    old_path: Option<String>,
    new_path: Option<String>,
    has_old_binary_file: bool,
    has_new_binary_file: bool,
    lines: OnceCell<Vec<DiffLine>>,
    repo: &'a Repository,
    args: &'a Args,
}

impl<'a> Diff<'a> {
    pub fn new(diff_delta: &DiffDelta, repo: &'a Repository, args: &'a Args) -> Self {
        let old_file_oid = diff_delta.old_file().id();
        let new_file_oid = diff_delta.new_file().id();
        Self {
            status: diff_delta.status(),
            old_file_oid,
            new_file_oid,
            old_path: diff_delta
                .old_file()
                .path()
                .map(|p| p.to_string_lossy().to_string()),
            new_path: diff_delta
                .new_file()
                .path()
                .map(|p| p.to_string_lossy().to_string()),
            has_old_binary_file: repo
                .find_blob(old_file_oid)
                .map(|blob| blob.is_binary())
                .unwrap_or(true),
            has_new_binary_file: repo
                .find_blob(new_file_oid)
                .map(|blob| blob.is_binary())
                .unwrap_or(true),
            lines: OnceCell::new(),
            repo,
            args,
        }
    }

    pub fn lines(&self) -> Option<&Vec<DiffLine>> {
        if self.has_new_binary_file {
            None
        } else {
            Some(self.lines.get_or_init(|| self.calc_lines()))
        }
    }

    fn calc_lines(&self) -> Vec<DiffLine> {
        let old_file_text = if self.has_old_binary_file {
            vec![]
        } else {
            self.repo
                .find_blob(self.old_file_oid)
                .map(|blob| blob.content().to_vec())
                .unwrap_or_default()
        };

        assert!(!self.has_new_binary_file);
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
                        .map(|(emphasized, text)| {
                            DiffLinePart::new(text.replace("\t", &self.args.tab_spaces), emphasized)
                        })
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

    pub fn status(&self) -> String {
        match self.status {
            Delta::Modified => format!("* Modified: {}", self.new_path.as_deref().unwrap()),
            Delta::Added => format!("* Added: {}", self.new_path.as_deref().unwrap()),
            Delta::Renamed => format!(
                "* Renamed: {} -> {}",
                self.old_path.as_deref().unwrap(),
                self.new_path.as_deref().unwrap()
            ),
            _ => unreachable!(),
        }
    }

    pub fn max_line_number_len(&self) -> usize {
        self.lines()
            .unwrap_or(&vec![])
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

    pub fn allowed_min_index(&self, _state: &State) -> usize {
        0
    }

    pub fn allowed_max_index(&self, state: &State) -> usize {
        if let Some(lines) = self.lines() {
            let diff_length = lines.len();
            let diff_height = Dashboard::diff_height(state.terminal_height());

            if state.args().beyond_last_line {
                diff_length.saturating_sub(1)
            } else {
                diff_length.saturating_sub(cmp::max(1, diff_height))
            }
        } else {
            0
        }
    }

    pub fn can_move_up(&self, index: usize, state: &State) -> bool {
        index > self.allowed_min_index(state)
    }

    pub fn can_move_down(&self, index: usize, state: &State) -> bool {
        index < self.allowed_max_index(state)
    }

    pub fn nearest_old_index_pair(&self, index: usize) -> IndexPair {
        if let Some(lines) = self.lines() {
            if let Some(line) = lines
                .iter()
                .skip(index)
                .find(|line| line.old_index.is_some())
            {
                assert!(line.index >= index);
                IndexPair::new(line.index - index, line.old_index.unwrap())
            } else if let Some(line) = lines
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
        } else {
            IndexPair::new(0, 0)
        }
    }

    pub fn nearest_new_index_pair(&self, index: usize) -> IndexPair {
        if let Some(lines) = self.lines() {
            if let Some(line) = lines
                .iter()
                .skip(index)
                .find(|line| line.new_index.is_some())
            {
                assert!(line.index >= index);
                IndexPair::new(line.index - index, line.new_index.unwrap())
            } else if let Some(line) = lines
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
        } else {
            IndexPair::new(0, 0)
        }
    }

    pub fn find_index_from_old_index(&self, old_index: usize) -> Option<usize> {
        self.lines().and_then(|lines| {
            lines
                .iter()
                .find(|line| {
                    line.old_index
                        // use https://doc.rust-lang.org/std/option/enum.Option.html#method.contains in the future
                        .filter(|i| *i == old_index)
                        .is_some()
                })
                .map(|line| line.index)
        })
    }

    pub fn find_index_from_new_index(&self, new_index: usize) -> Option<usize> {
        self.lines().and_then(|lines| {
            lines
                .iter()
                .find(|line| {
                    line.new_index
                        // use https://doc.rust-lang.org/std/option/enum.Option.html#method.contains in the future
                        .filter(|i| *i == new_index)
                        .is_some()
                })
                .map(|line| line.index)
        })
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
