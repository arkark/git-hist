use anyhow::Result;
use std::panic;

mod commit;
mod controller;
mod dashboard;
mod diff;
mod git;
mod history;
mod state;
mod terminal;

use crate::args::Args;
use dashboard::Dashboard;
use state::State;
use terminal::Terminal;

pub fn run(args: Args) -> Result<()> {
    let repo = git::get_repository()?;
    let history = git::get_history(&args.file_path, &repo, &args)?;

    terminal::initialize()?;

    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = exit();
        default_hook(panic_info);
    }));

    (|| -> Result<()> {
        let mut terminal = Terminal::new()?;
        let mut current_state = State::first(&history, &terminal, &args);
        let dashboard = Dashboard::new(&current_state);
        dashboard.draw(&mut terminal)?;

        while let Some(next_state) = controller::poll_next_event(current_state, &history)? {
            current_state = next_state;
            let dashboard = Dashboard::new(&current_state);
            dashboard.draw(&mut terminal)?;
        }

        Ok(())
    })()
    .map_err(|e| {
        let _ = exit();
        e
    })?;

    exit()
}

fn exit() -> Result<()> {
    terminal::terminate()?;
    Ok(())
}
