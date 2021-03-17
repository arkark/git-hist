use anyhow::Result;
use git_hist::{app::App, args::Args};

fn main() -> Result<()> {
    let args = Args::load();

    App::run(args)?;

    Ok(())
}
