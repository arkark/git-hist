use chrono::TimeZone;
use git2::{Commit as GitCommit, Oid, Repository};
use itertools::Itertools;
use once_cell::sync::OnceCell;
use std::{collections::HashMap, fmt};

const HEAD_NAME: &str = "HEAD";

pub struct Commit<'a> {
    oid: Oid,
    short_id: String,
    long_id: String,
    author_name: String,
    author_date: chrono::DateTime<chrono::Local>,
    committer_name: String,
    committer_date: chrono::DateTime<chrono::Local>,
    summary: String,
    references: OnceCell<References>,
    repo: &'a Repository,
}

impl<'a> Commit<'a> {
    pub fn new(commit: &GitCommit, repo: &'a Repository) -> Self {
        let oid = commit.id();
        let short_id = commit
            .as_object()
            .short_id()
            .unwrap()
            .as_str()
            .unwrap_or_default()
            .to_string();
        let long_id = format!("{}", oid);
        let author = commit.author().name().unwrap_or_default().to_string();
        let author_date = chrono::DateTime::<chrono::Local>::from(
            chrono::Utc.timestamp(commit.author().when().seconds(), 0),
        );
        let committer = commit.committer().name().unwrap_or_default().to_string();
        let committer_date = chrono::DateTime::<chrono::Local>::from(
            chrono::Utc.timestamp(commit.committer().when().seconds(), 0),
        );
        let summary = commit.summary().unwrap_or_default().to_string();

        Self {
            oid,
            short_id,
            long_id,
            author_name: author,
            author_date,
            committer_name: committer,
            committer_date,
            summary,
            references: OnceCell::new(),
            repo,
        }
    }

    pub fn short_id(&self) -> &str {
        &self.short_id
    }

    pub fn long_id(&self) -> &str {
        &self.long_id
    }

    pub fn author_name(&self) -> &str {
        &self.author_name
    }

    pub fn author_date(&self) -> &chrono::DateTime<chrono::Local> {
        &self.author_date
    }

    pub fn committer_name(&self) -> &str {
        &self.committer_name
    }

    pub fn committer_date(&self) -> &chrono::DateTime<chrono::Local> {
        &self.committer_date
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }

    pub fn references(&self) -> &References {
        self.references.get_or_init(|| self.calc_references())
    }

    fn calc_references(&self) -> References {
        let head = self.repo.head().unwrap();

        let references = self
            .repo
            .references()
            .unwrap()
            .filter_map(|r| r.ok())
            .filter(|r| {
                r.target()
                    // use https://doc.rust-lang.org/std/option/enum.Option.html#method.contains in the future
                    .filter(|oid| *oid == self.oid)
                    .is_some()
            });
        let reference_groups: HashMap<ReferenceType, Vec<_>> =
            references.into_group_map_by(|r| match r {
                _ if r.is_branch() => ReferenceType::LocalBranch,
                _ if r.is_remote() => ReferenceType::RemoteBranch,
                _ if r.is_tag() => ReferenceType::Tag,
                _ => unreachable!(),
            });

        let local_branches: Vec<LocalBranch> = reference_groups
            .get(&ReferenceType::LocalBranch)
            .map(|rs| rs.iter().collect::<Vec<_>>())
            .unwrap_or_else(Vec::new)
            .iter()
            .filter_map(|r| {
                r.shorthand()
                    .map(|name| LocalBranch::new(name, r.name() == head.name()))
            })
            .collect();

        let remote_branches: Vec<RemoteBranch> = reference_groups
            .get(&ReferenceType::RemoteBranch)
            .map(|rs| rs.iter().collect::<Vec<_>>())
            .unwrap_or_else(Vec::new)
            .iter()
            .filter_map(|r| r.shorthand().map(RemoteBranch::new))
            .collect();

        let tags: Vec<Tag> = reference_groups
            .get(&ReferenceType::Tag)
            .map(|rs| rs.iter().collect::<Vec<_>>())
            .unwrap_or_else(Vec::new)
            .iter()
            .filter_map(|r| r.shorthand().map(Tag::new))
            .collect();

        let is_head = head.target().unwrap() == self.oid && head.name() == Some(HEAD_NAME);

        References::new(local_branches, remote_branches, tags, is_head)
    }
}

#[derive(Debug)]
pub struct References {
    local_branches: Vec<LocalBranch>,
    remote_branches: Vec<RemoteBranch>,
    tags: Vec<Tag>,
    is_head: bool,
}

impl References {
    pub fn new(
        local_branches: Vec<LocalBranch>,
        remote_branches: Vec<RemoteBranch>,
        tags: Vec<Tag>,
        is_head: bool,
    ) -> Self {
        Self {
            local_branches,
            remote_branches,
            tags,
            is_head,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.local_branches.is_empty()
            && self.remote_branches.is_empty()
            && self.tags.is_empty()
            && !self.is_head
    }

    pub fn head_names(&self) -> Vec<String> {
        if self.is_head {
            vec![String::from(HEAD_NAME)]
        } else {
            vec![]
        }
    }

    pub fn local_branch_names(&self) -> Vec<String> {
        self.local_branches
            .iter()
            .map(|x| format!("{}", x))
            .collect()
    }

    pub fn remote_branch_names(&self) -> Vec<String> {
        self.remote_branches
            .iter()
            .map(|x| format!("{}", x))
            .collect()
    }

    pub fn tag_names(&self) -> Vec<String> {
        self.tags.iter().map(|x| format!("{}", x)).collect()
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum ReferenceType {
    LocalBranch,
    RemoteBranch,
    Tag,
}

#[derive(Debug)]
pub struct LocalBranch {
    name: String,
    is_head: bool,
}

impl LocalBranch {
    pub fn new(name: impl Into<String>, is_head: bool) -> Self {
        Self {
            name: name.into(),
            is_head,
        }
    }
}

impl fmt::Display for LocalBranch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_head {
            write!(f, "{} -> {}", HEAD_NAME, self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

#[derive(Debug)]
pub struct RemoteBranch {
    name: String,
}

impl RemoteBranch {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl fmt::Display for RemoteBranch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug)]
pub struct Tag {
    name: String,
}

impl Tag {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "tag: {}", self.name)
    }
}
