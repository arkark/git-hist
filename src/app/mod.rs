use crate::args::Args;

mod controller;
mod git;
mod history;
mod terminal;

use controller::State;
use history::History;
use terminal::Terminal;

use anyhow::Result;
use chrono::TimeZone;
use git2::{Delta, Reference, Repository};
use itertools::Itertools;
use similar::{ChangeTag, TextDiff};
use std::cmp;
use std::collections::HashMap;
use tui::{layout, text, widgets};

pub fn run(args: Args) -> Result<()> {
    let repo = git::get_repository()?;
    let history = git::get_history(&args.file_path)?;

    terminal::initialize()?;

    let mut terminal = Terminal::new()?;
    let mut current_state = State::new(history.latest().unwrap(), 0);
    display(&mut terminal, &current_state, &history, &repo)?;

    loop {
        if let Some(next_state) = current_state.poll_next_event(&history)? {
            current_state = next_state;
            display(&mut terminal, &current_state, &history, &repo)?;
        } else {
            break;
        }
    }

    terminal::terminate()?;
    Ok(())
}

fn display(
    terminal: &mut Terminal,
    current_state: &State,
    history: &History,
    repo: &Repository,
) -> Result<()> {
    let commit = current_state.point().get_commit(&repo);

    let backward_symbol = if let Some(_) = history.backward(current_state.point()) {
        "<<"
    } else {
        " "
    };
    let backward_text = vec![
        text::Spans::from(""),
        text::Spans::from(backward_symbol),
        text::Spans::from(backward_symbol),
        text::Spans::from(""),
    ];

    let forward_symbol = if let Some(_) = history.forward(current_state.point()) {
        ">>"
    } else {
        "  "
    };
    let forward_text = vec![
        text::Spans::from(""),
        text::Spans::from(forward_symbol),
        text::Spans::from(forward_symbol),
        text::Spans::from(""),
    ];

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

    let commit_author = &format!("@{}", commit.author().name().unwrap_or_default());

    let mut title = vec![];
    title.push(text::Span::raw(" "));
    title.push(text::Span::raw("Commit:"));
    title.push(text::Span::raw(" "));
    title.push(text::Span::raw(
        commit_short_id.as_str().unwrap_or_default(),
    ));
    title.push(text::Span::raw(" "));
    title.push(text::Span::raw(commit_date));
    title.push(text::Span::raw(" "));
    if head_names.len() > 0
        || branch_names.len() > 0
        || remote_names.len() > 0
        || tag_names.len() > 0
    {
        title.push(text::Span::raw("("));
        for name in head_names {
            title.push(text::Span::raw(name));
            title.push(text::Span::raw(", "));
        }
        for name in branch_names {
            title.push(text::Span::raw(name));
            title.push(text::Span::raw(", "));
        }
        for name in remote_names {
            title.push(text::Span::raw(name));
            title.push(text::Span::raw(", "));
        }
        for name in tag_names {
            title.push(text::Span::raw(name));
            title.push(text::Span::raw(", "));
        }
        title.pop();
        title.push(text::Span::raw(")"));
        title.push(text::Span::raw(" "));
    }
    title.push(text::Span::raw(commit_author));
    title.push(text::Span::raw(" "));
    let title = text::Spans::from(title);

    let commit_summary = commit.summary().unwrap_or_default();
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

    let commit_paragraph_lines = vec![commit_summary, change_status];

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

    let mut diff_lines = vec![];
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
        diff_lines.push(text::Spans::from(vec![
            text::Span::raw(old_line_number),
            text::Span::raw(" "),
            text::Span::raw(new_line_number),
            text::Span::raw("|"),
            text::Span::raw(sign),
            text::Span::raw(" "),
            text::Span::raw(change.to_string_lossy()),
        ]))
    }

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

        // commit
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
        let backward_chunk = horizontal_chunks[0];
        let backward_paragraph = widgets::Paragraph::new(backward_text);
        frame.render_widget(backward_paragraph, backward_chunk);
        let forward_chunk = horizontal_chunks[2];
        let forward_paragraph = widgets::Paragraph::new(forward_text);
        frame.render_widget(forward_paragraph, forward_chunk);

        let commit_content_chunk = layout::Layout::default()
            .horizontal_margin(1)
            .constraints([layout::Constraint::Min(0)].as_ref())
            .split(horizontal_chunks[1])[0];
        let block = widgets::Block::default()
            .title(title)
            .borders(widgets::Borders::ALL)
            .border_type(widgets::BorderType::Rounded);
        let commit_paragraph = widgets::Paragraph::new(commit_paragraph_lines).block(block);
        frame.render_widget(commit_paragraph, commit_content_chunk);

        // diff
        let diff_paragraph = widgets::Paragraph::new(diff_lines);
        frame.render_widget(diff_paragraph, diff_chunk);
    })?;

    Ok(())
}
