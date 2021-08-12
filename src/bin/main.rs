use anyhow::Result;
use git_hist::{app, args::Args};

fn main() -> Result<()> {
    let args = Args::load()?;
    app::run(args)?;

    Ok(())
}
