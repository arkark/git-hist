use crate::app::commit::Commit;
use crate::app::diff::Diff;
use crate::app::history::{History, TurningPoint};
use crate::args::Args;
use anyhow::{anyhow, Context, Result};
use git2::{DiffFindOptions, ObjectType, Repository};
use std::env;
use std::path;

pub fn get_repository() -> Result<Repository> {
    let repo = Repository::discover(env::current_dir()?)
        .context("Faild to open a git repository for the current directory")?;
    if repo.is_bare() {
        return Err(anyhow!("git-hist dose not support a bare repository"));
    }
    Ok(repo)
}

pub fn get_history<'a, P: AsRef<path::Path>>(
    file_path: P,
    repo: &'a Repository,
    args: &'a Args,
) -> Result<History<'a>> {
    let file_path_from_repository = env::current_dir()
        .unwrap()
        .join(&file_path)
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
        .with_context(|| {
            format!(
                "Failed to find the file '{}' on HEAD",
                file_path.as_ref().to_string_lossy()
            )
        })
        .and_then(|entry| {
            if let Some(ObjectType::Blob) = entry.kind() {
                Ok(entry)
            } else {
                Err(anyhow!(
                    "Failed to find the path '{}' as a blob on HEAD",
                    file_path.as_ref().to_string_lossy()
                ))
            }
        })?
        .id();

    let mut file_oid = latest_file_oid;
    let mut file_path = file_path_from_repository;
    let history = History::new(commits.iter().filter_map(|git_commit| {
        let old_tree = git_commit.parent(0).and_then(|p| p.tree()).ok();
        let new_tree = git_commit.tree().ok();
        assert!(new_tree.is_some());

        let mut git_diff = repo
            .diff_tree_to_tree(old_tree.as_ref(), new_tree.as_ref(), None)
            .unwrap();

        // detect file renames
        git_diff
            .find_similar(Some(DiffFindOptions::new().renames(true)))
            .unwrap();

        let delta = git_diff.deltas().find(|delta| {
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
            let commit = Commit::new(git_commit, repo);
            let diff = Diff::new(&delta, repo, args);
            TurningPoint::new(commit, diff)
        })
    }));

    Ok(history)
}
