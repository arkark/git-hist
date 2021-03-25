use crate::args::Args;

mod commit;
mod controller;
mod dashboard;
mod diff;
mod git;
mod history;
mod state;
mod terminal;

use dashboard::Dashboard;
use state::State;
use terminal::Terminal;

use anyhow::Result;

pub fn run(args: Args) -> Result<()> {
    let repo = git::get_repository()?;
    let history = git::get_history(&args.file_path, &repo)?;

    terminal::initialize()?;

    let mut terminal = Terminal::new()?;
    let mut current_state = State::first(&history, &terminal);
    let dashboard = Dashboard::new(&current_state);
    dashboard.draw(&mut terminal)?;

    while let Some(next_state) = controller::poll_next_event(current_state, &history)? {
        current_state = next_state;
        let dashboard = Dashboard::new(&current_state);
        dashboard.draw(&mut terminal)?;
    }

    terminal::terminate()?;
    Ok(())
}
