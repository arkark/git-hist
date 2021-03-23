use crate::app::state::State;
use crate::app::terminal::Terminal;
use anyhow::Result;
use chrono::TimeZone;
use git2::{Delta, Reference, Repository};
use itertools::Itertools;
use similar::{ChangeTag, TextDiff};
use std::cmp;
use std::collections::HashMap;
use tui::{layout, text, widgets};

pub struct Dashboard<'a> {
    commit_info_title: text::Spans<'a>,
    commit_info_text: Vec<text::Spans<'a>>,
    left_navi_text: Vec<text::Spans<'a>>,
    right_navi_text: Vec<text::Spans<'a>>,
    diff_text: Vec<text::Spans<'a>>,
}

impl<'a> Dashboard<'a> {
    pub fn new(current_state: &'a State, repo: &'a Repository) -> Result<Self> {
        let left_navi_text = get_left_navi_text(&current_state, &repo)?;
        let right_navi_text = get_right_navi_text(&current_state, &repo)?;
        let commit_info_title = get_commit_info_title(&current_state, &repo)?;
        let commit_info_text = get_commit_info_text(&current_state, &repo)?;
        let diff_text = get_diff_text(&current_state, &repo)?;

        Ok(Self {
            commit_info_title,
            commit_info_text,
            left_navi_text,
            right_navi_text,
            diff_text,
        })
    }

    pub fn draw(self, terminal: &mut Terminal) -> Result<()> {
        terminal.draw(|frame| {
            let vertical_chunks = layout::Layout::default()
                .direction(layout::Direction::Vertical)
                .constraints(
                    [
                        layout::Constraint::Length(2 + 2),
                        layout::Constraint::Min(0),
                    ]
                    .as_ref(),
                )
                .split(frame.size());

            let commit_chunk = vertical_chunks[0];
            let diff_chunk = vertical_chunks[1];

            let horizontal_chunks = layout::Layout::default()
                .direction(layout::Direction::Horizontal)
                .constraints(
                    [
                        layout::Constraint::Length(2),
                        layout::Constraint::Min(0),
                        layout::Constraint::Length(2),
                    ]
                    .as_ref(),
                )
                .split(commit_chunk);
            let left_navi_chunk = horizontal_chunks[0];
            let right_navi_chunk = horizontal_chunks[2];
            let commit_info_chunk = layout::Layout::default()
                .horizontal_margin(1)
                .constraints([layout::Constraint::Min(0)].as_ref())
                .split(horizontal_chunks[1])[0];

            let left_navi_paragraph = widgets::Paragraph::new(self.left_navi_text);
            frame.render_widget(left_navi_paragraph, left_navi_chunk);

            let right_navi_paragraph = widgets::Paragraph::new(self.right_navi_text);
            frame.render_widget(right_navi_paragraph, right_navi_chunk);

            let block = widgets::Block::default()
                .title(self.commit_info_title)
                .borders(widgets::Borders::ALL)
                .border_type(widgets::BorderType::Rounded);
            let commit_info_paragraph = widgets::Paragraph::new(self.commit_info_text).block(block);
            frame.render_widget(commit_info_paragraph, commit_info_chunk);

            // diff
            let diff_paragraph = widgets::Paragraph::new(self.diff_text);
            frame.render_widget(diff_paragraph, diff_chunk);
        })?;

        Ok(())
    }
}

fn get_left_navi_text<'a>(
    current_state: &'a State,
    _repo: &'a Repository,
) -> Result<Vec<text::Spans<'a>>> {
    let backward_symbol = if current_state.is_earliest_commit() {
        " "
    } else {
        "<<"
    };

    Ok(vec![
        text::Spans::from(""),
        text::Spans::from(backward_symbol),
        text::Spans::from(backward_symbol),
        text::Spans::from(""),
    ])
}

fn get_right_navi_text<'a>(
    current_state: &'a State,
    _repo: &'a Repository,
) -> Result<Vec<text::Spans<'a>>> {
    let forward_symbol = if current_state.is_latest_commit() {
        "  "
    } else {
        ">>"
    };
    Ok(vec![
        text::Spans::from(""),
        text::Spans::from(forward_symbol),
        text::Spans::from(forward_symbol),
        text::Spans::from(""),
    ])
}

fn get_commit_info_title<'a>(
    current_state: &'a State,
    repo: &'a Repository,
) -> Result<text::Spans<'a>> {
    let commit = current_state.point().get_commit(&repo);

    let commit_short_id = commit.as_object().short_id()?;

    let references = repo.references()?.filter_map(|r| r.ok()).filter(|r| {
        if let Some(oid) = r.target() {
            oid == commit.id()
        } else {
            false
        }
    });
    let references_groups: HashMap<&str, Vec<Reference<'_>>> =
        references.into_group_map_by(|r| match r {
            _ if r.is_branch() => "branch",
            _ if r.is_remote() => "remote",
            _ if r.is_tag() => "tag",
            _ => "",
        });

    let empty_vec = Vec::new();

    let head = repo.head().unwrap();
    let head_names = if head.target().unwrap() == commit.id() && head.name() == Some("HEAD") {
        vec![String::from("HEAD")]
    } else {
        vec![]
    };
    let branch_names = references_groups
        .get("branch")
        .unwrap_or(&empty_vec)
        .into_iter()
        .filter_map(|r| {
            r.shorthand().map(|name| {
                let head_prefix = if r.name() == head.name() {
                    "HEAD -> "
                } else {
                    ""
                };
                format!("{}{}", head_prefix, name)
            })
        })
        .collect::<Vec<_>>();
    let remote_names = references_groups
        .get("remote")
        .unwrap_or(&empty_vec)
        .into_iter()
        .filter_map(|r| r.shorthand().map(|name| format!("{}", name)))
        .collect::<Vec<_>>();
    let tag_names = references_groups
        .get("tag")
        .unwrap_or(&empty_vec)
        .into_iter()
        .filter_map(|r| r.shorthand().map(|name| format!("tag: {}", name)))
        .collect::<Vec<_>>();

    // TODO:
    //   - option: date format (default: "[%Y-%m-%d]")
    //     - ref. https://docs.rs/chrono/0.4.19/chrono/format/strftime/index.html
    //   - option: author date (default) or committer date
    let commit_date = chrono::DateTime::<chrono::Local>::from(
        chrono::Utc.timestamp(commit.author().when().seconds(), 0),
    )
    .format("[%Y-%m-%d]")
    .to_string();

    let commit_author = format!("@{}", commit.author().name().unwrap_or_default());

    let mut commit_info_title = vec![];
    commit_info_title.push(text::Span::raw(" "));
    commit_info_title.push(text::Span::raw("Commit:"));
    commit_info_title.push(text::Span::raw(" "));
    commit_info_title.push(text::Span::raw(String::from(
        commit_short_id.as_str().unwrap_or_default(),
    )));
    commit_info_title.push(text::Span::raw(" "));
    commit_info_title.push(text::Span::raw(commit_date));
    commit_info_title.push(text::Span::raw(" "));
    if head_names.len() > 0
        || branch_names.len() > 0
        || remote_names.len() > 0
        || tag_names.len() > 0
    {
        commit_info_title.push(text::Span::raw("("));
        for name in head_names {
            commit_info_title.push(text::Span::raw(name));
            commit_info_title.push(text::Span::raw(", "));
        }
        for name in branch_names {
            commit_info_title.push(text::Span::raw(name));
            commit_info_title.push(text::Span::raw(", "));
        }
        for name in remote_names {
            commit_info_title.push(text::Span::raw(name));
            commit_info_title.push(text::Span::raw(", "));
        }
        for name in tag_names {
            commit_info_title.push(text::Span::raw(name));
            commit_info_title.push(text::Span::raw(", "));
        }
        commit_info_title.pop();
        commit_info_title.push(text::Span::raw(")"));
        commit_info_title.push(text::Span::raw(" "));
    }
    commit_info_title.push(text::Span::raw(commit_author));
    commit_info_title.push(text::Span::raw(" "));
    let commit_info_title = text::Spans::from(commit_info_title);

    Ok(commit_info_title)
}

fn get_commit_info_text<'a>(
    current_state: &'a State,
    repo: &'a Repository,
) -> Result<Vec<text::Spans<'a>>> {
    let commit = current_state.point().get_commit(&repo);

    let commit_summary = String::from(commit.summary().unwrap_or_default());
    let commit_summary = text::Spans::from(vec![text::Span::raw(commit_summary)]);

    let old_path = current_state.point().old_path();
    let new_path = current_state.point().new_path();
    assert!(new_path.is_some());

    let change_status = match current_state.point().diff_status() {
        Delta::Modified => vec![
            text::Span::raw("* Modified: "),
            text::Span::raw(new_path.unwrap()),
        ],
        Delta::Added => vec![
            text::Span::raw("* Added: "),
            text::Span::raw(new_path.unwrap()),
        ],
        Delta::Renamed => vec![
            text::Span::raw("* Renamed: "),
            text::Span::raw(old_path.unwrap()),
            text::Span::raw(" -> "),
            text::Span::raw(new_path.unwrap()),
        ],
        _ => unreachable!(),
    };
    let change_status = text::Spans(change_status);

    Ok(vec![commit_summary, change_status])
}

fn get_diff_text<'a>(
    current_state: &'a State,
    repo: &'a Repository,
) -> Result<Vec<text::Spans<'a>>> {
    let old_file_text = current_state
        .point()
        .get_old_blob(repo)
        .map(|blob| blob.content().to_vec())
        .unwrap_or_default();
    let new_file_text = current_state
        .point()
        .get_new_blob(repo)
        .map(|blob| blob.content().to_vec())
        .unwrap_or_default();

    let mut diff_text = vec![];
    let text_diff = TextDiff::from_lines(&old_file_text, &new_file_text);
    let max_line_number_len = text_diff
        .iter_all_changes()
        .filter_map(|change| {
            cmp::max(change.old_index(), change.new_index()).map(|x| {
                // 0-indexed to 1-indexed
                x + 1
            })
        })
        .fold(0, |acc, number| cmp::max(acc, number))
        .to_string()
        .len();
    for change in text_diff.iter_all_changes() {
        let old_line_number = format!(
            "{:>1$}",
            if let Some(index) = change.old_index() {
                (index + 1).to_string()
            } else {
                String::new()
            },
            max_line_number_len,
        );
        let new_line_number = format!(
            "{:>1$}",
            if let Some(index) = change.new_index() {
                (index + 1).to_string()
            } else {
                String::new()
            },
            max_line_number_len,
        );
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        diff_text.push(text::Spans::from(vec![
            text::Span::raw(old_line_number),
            text::Span::raw(" "),
            text::Span::raw(new_line_number),
            text::Span::raw("|"),
            text::Span::raw(sign),
            text::Span::raw(" "),
            text::Span::raw(String::from(change.to_string_lossy())),
        ]))
    }

    Ok(diff_text)
}
