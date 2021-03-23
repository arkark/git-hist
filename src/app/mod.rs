use crate::args::Args;

mod controller;
mod dashboard;
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
    let history = git::get_history(&args.file_path)?;

    terminal::initialize()?;

    let mut terminal = Terminal::new()?;
    let mut current_state = State::from(&history);
    let dashboard = Dashboard::new(&current_state, &repo)?;
    dashboard.draw(&mut terminal)?;

    loop {
        if let Some(next_state) = controller::poll_next_event(current_state, &history)? {
            current_state = next_state;
            let dashboard = Dashboard::new(&current_state, &repo)?;
            dashboard.draw(&mut terminal)?;
        } else {
            break;
        }
    }

    terminal::terminate()?;
    Ok(())
}
