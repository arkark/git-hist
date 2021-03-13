use std::path::Path;

use crate::args::Args;
use anyhow::{Context, Result};
use git2::{Repository, RepositoryOpenFlags};
pub struct App;

impl App {
    pub fn run(args: Args) -> Result<()> {
        let repo = Repository::open_ext(
            &args.file_path,
            RepositoryOpenFlags::empty(),
            Vec::<&Path>::new(),
        )
        .with_context(|| format!("Faild to open a git repository for {:?}", args.file_path))?;

        println!("{:?}", repo.path());

        Ok(())
    }
}
