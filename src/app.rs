use crate::args::Args;

use anyhow::{anyhow, Context, Result};
use chrono::TimeZone;
use crossterm::{
    cursor,
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::style,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use git2::{Blob, Commit, Delta, DiffFindOptions, ObjectType, Oid, Reference, Repository};
use itertools::Itertools;
use std::io;
use std::{collections::HashMap, env};
use tui::{backend::CrosstermBackend, layout, text, widgets, Terminal};

pub struct App;

#[derive(Debug)]
struct TurningPoint {
    commit_oid: Oid,
    old_file_oid: Oid,
    new_file_oid: Oid,
    old_path: Option<String>,
    new_path: Option<String>,
    change_status: Delta,
}

impl TurningPoint {
    pub fn new<S: Into<String>>(
        commit_oid: Oid,
        old_file_oid: Oid,
        new_file_oid: Oid,
        old_path: Option<S>,
        new_path: Option<S>,
        change_status: Delta,
    ) -> Self {
        Self {
            commit_oid,
            old_file_oid,
            new_file_oid,
            old_path: old_path.map(|path| path.into()),
            new_path: new_path.map(|path| path.into()),
            change_status,
        }
    }

    fn get_commit<'repo>(&self, repo: &'repo Repository) -> Commit<'repo> {
        repo.find_commit(self.commit_oid).unwrap()
    }

    fn get_old_blob<'repo>(&self, repo: &'repo Repository) -> Option<Blob<'repo>> {
        repo.find_blob(self.old_file_oid).ok()
    }

    fn get_new_blob<'repo>(&self, repo: &'repo Repository) -> Blob<'repo> {
        repo.find_blob(self.new_file_oid).unwrap()
    }
}

struct History {
    points: Vec<TurningPoint>,
    current_index: usize,
}

impl History {
    pub fn new<I: Iterator<Item = TurningPoint>>(points: I) -> Self {
        Self {
            points: points.collect(),
            current_index: 0,
        }
    }

    pub fn current(&self) -> &TurningPoint {
        self.points.get(self.current_index).unwrap()
    }

    pub fn go_backward(&mut self) -> Option<&TurningPoint> {
        if self.current_index + 1 < self.points.len() {
            self.current_index += 1;
            Some(self.current())
        } else {
            None
        }
    }

    pub fn go_forward(&mut self) -> Option<&TurningPoint> {
        if self.current_index > 0 {
            self.current_index -= 1;
            Some(self.current())
        } else {
            None
        }
    }

    pub fn is_latest(&self) -> bool {
        self.current_index == 0
    }

    pub fn is_earliest(&self) -> bool {
        self.current_index + 1 == self.points.len()
    }
}

impl App {
    pub fn run(args: Args) -> Result<()> {
        let repo = Repository::discover(env::current_dir()?)
            .context("Faild to open a git repository for the current directory")?;
        if repo.is_bare() {
            Err(anyhow!("git-hist dose not support a bare repository"))?;
        }

        let file_path_from_repository = env::current_dir()
            .unwrap()
            .join(&args.file_path)
            .strip_prefix(repo.path().parent().unwrap())
            .unwrap()
            .to_path_buf();

        let mut revwalk = repo
            .revwalk()
            .context("Failed to traverse the commit graph")?;
        revwalk.push_head().context("Failed to find HEAD")?;
        revwalk.simplify_first_parent()?;

        let commits = revwalk
            .map(|oid| oid.and_then(|oid| repo.find_commit(oid)).unwrap())
            .collect::<Vec<_>>();
        let latest_file_oid = commits
            .first()
            .context("Failed to get any commit")?
            .tree()
            .unwrap()
            .get_path(&file_path_from_repository)
            .with_context(|| format!("Failed to find the file '{}' on HEAD", args.file_path))
            .and_then(|entry| {
                if let Some(ObjectType::Blob) = entry.kind() {
                    Ok(entry)
                } else {
                    Err(anyhow!(
                        "Failed to find the path '{}' as a blob on HEAD",
                        args.file_path
                    ))
                }
            })?
            .id();

        let mut file_oid = latest_file_oid;
        let mut file_path = file_path_from_repository;
        let mut history = History::new(commits.iter().filter_map(|commit| {
            let old_tree = commit.parent(0).and_then(|p| p.tree()).ok();
            let new_tree = commit.tree().ok();
            assert!(new_tree.is_some());

            let mut diff = repo
                .diff_tree_to_tree(old_tree.as_ref(), new_tree.as_ref(), None)
                .unwrap();

            // detect file renames
            diff.find_similar(Some(DiffFindOptions::new().renames(true)))
                .unwrap();

            let delta = diff.deltas().find(|delta| {
                delta.new_file().id() == file_oid
                    && delta
                        .new_file()
                        .path()
                        .filter(|path| *path == file_path)
                        .is_some()
            });
            if let Some(delta) = delta.as_ref() {
                file_oid = delta.old_file().id();
                file_path = delta.old_file().path().unwrap().to_path_buf();
            }

            delta.map(|delta| {
                TurningPoint::new(
                    commit.id(),
                    delta.old_file().id(),
                    delta.new_file().id(),
                    delta.old_file().path().map(|p| p.to_string_lossy()),
                    delta.new_file().path().map(|p| p.to_string_lossy()),
                    delta.status(),
                )
            })
        }));

        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, cursor::Hide)?;

        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        loop {
            display(&mut terminal, &history, &repo)?;
            match read()? {
                Event::Key(event) => match event {
                    KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                    }
                    | KeyEvent {
                        code: KeyCode::Char('d'),
                        modifiers: KeyModifiers::CONTROL,
                    }
                    | KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: _,
                    } => break,
                    KeyEvent {
                        code: KeyCode::Up,
                        modifiers: _,
                    } => {}
                    KeyEvent {
                        code: KeyCode::Down,
                        modifiers: _,
                    } => {}
                    KeyEvent {
                        code: KeyCode::Right,
                        modifiers: _,
                    } => {
                        if let Some(_) = history.go_forward() {
                            display(&mut terminal, &history, &repo)?;
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Left,
                        modifiers: _,
                    } => {
                        if let Some(_) = history.go_backward() {
                            display(&mut terminal, &history, &repo)?;
                        }
                    }
                    _ => {
                        //
                    }
                },
                _ => {}
            }
        }

        execute!(io::stdout(), cursor::Show, LeaveAlternateScreen)?;
        disable_raw_mode()?;

        Ok(())
    }
}

fn display<W: io::Write>(
    terminal: &mut Terminal<CrosstermBackend<W>>,
    history: &History,
    repo: &Repository,
) -> Result<()> {
    let commit = history.current().get_commit(&repo);

    let backward_symbol = if history.is_earliest() { " " } else { "<<" };
    let backward_text = vec![
        text::Spans::from(""),
        text::Spans::from(backward_symbol),
        text::Spans::from(backward_symbol),
        text::Spans::from(""),
    ];

    let forward_symbol = if history.is_latest() { " " } else { ">>" };
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

    let old_path = history.current().old_path.as_ref();
    let new_path = history.current().new_path.as_ref();
    assert!(new_path.is_some());

    let change_status = match history.current().change_status {
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
        let block = widgets::Block::default()
            .title(" TODO: file diff ")
            .borders(widgets::Borders::ALL);
        frame.render_widget(block, diff_chunk);
    })?;

    Ok(())
}
