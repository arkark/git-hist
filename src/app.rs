use crate::args::Args;

use anyhow::{anyhow, Context, Result};
use crossterm::{
    cursor,
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use git2::{Blob, Commit, Delta, ObjectType, Oid, Reference, Repository};
use itertools::Itertools;
use std::io;
use std::{collections::HashMap, env};
use tui::{backend::CrosstermBackend, layout, widgets, Terminal};

pub struct App;

#[derive(Debug)]
struct TurningPoint {
    commit_oid: Oid,
    old_file_oid: Oid,
    new_file_oid: Oid,
    change_status: Delta,
}

impl TurningPoint {
    pub fn new(
        commit_oid: Oid,
        old_file_oid: Oid,
        new_file_oid: Oid,
        change_status: Delta,
    ) -> Self {
        Self {
            commit_oid,
            old_file_oid,
            new_file_oid,
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

    pub fn go_foreward(&mut self) -> Option<&TurningPoint> {
        if self.current_index > 0 {
            self.current_index -= 1;
            Some(self.current())
        } else {
            None
        }
    }
}

impl App {
    pub fn run(args: Args) -> Result<()> {
        let repo = Repository::discover(env::current_dir()?)
            .context("Faild to open a git repository for the current directory")?;

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
            .get_path(args.file_path.as_ref())
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
        let mut history = History::new(commits.iter().filter_map(|commit| {
            let old_tree = commit.parent(0).and_then(|p| p.tree()).ok();
            let new_tree = commit.tree().ok();
            assert!(new_tree.is_some());

            let mut diff = repo
                .diff_tree_to_tree(old_tree.as_ref(), new_tree.as_ref(), None)
                .unwrap();

            // detect file renames, copies, etc.
            diff.find_similar(None).unwrap();

            let delta = diff
                .deltas()
                .find(|delta| delta.new_file().id() == file_oid);
            if let Some(delta) = delta.as_ref() {
                file_oid = delta.old_file().id();
            }

            delta.map(|delta| {
                TurningPoint::new(
                    commit.id(),
                    delta.old_file().id(),
                    delta.new_file().id(),
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
                        code: KeyCode::Esc,
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
                        if let Some(_) = history.go_foreward() {
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
    let commit_id = commit.as_object().short_id()?;

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
        });
    let remote_names = references_groups
        .get("remote")
        .unwrap_or(&empty_vec)
        .into_iter()
        .filter_map(|r| r.shorthand().map(|name| format!("{}", name)));
    let tag_names = references_groups
        .get("tag")
        .unwrap_or(&empty_vec)
        .into_iter()
        .filter_map(|r| r.shorthand().map(|name| format!("tag: {}", name)));

    let reference_text = head_names
        .into_iter()
        .chain(branch_names)
        .chain(remote_names)
        .chain(tag_names)
        .join(", ");

    let mut title = format!(" Commit: {} ", commit_id.as_str().unwrap());
    if !reference_text.is_empty() {
        title.push_str(&format!("({}) ", reference_text));
    }

    terminal.draw(|frame| {
        let chunks = layout::Layout::default()
            .direction(layout::Direction::Vertical)
            .constraints(
                [
                    layout::Constraint::Length(3 + 2),
                    layout::Constraint::Min(0),
                ]
                .as_ref(),
            )
            .split(frame.size());
        let block = widgets::Block::default()
            .title(title)
            .borders(widgets::Borders::ALL);
        frame.render_widget(block, chunks[0]);
        let block = widgets::Block::default()
            .title(format!(
                "{}",
                history.current().get_commit(&repo).summary().unwrap_or("")
            ))
            .borders(widgets::Borders::ALL);
        frame.render_widget(block, chunks[1]);
    })?;

    Ok(())
}
