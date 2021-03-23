use crate::app::history::History;
use crate::app::state::State;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

pub fn poll_next_event<'a>(state: State<'a>, history: &'a History) -> Result<Option<State<'a>>> {
    match event::read()? {
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
            } => Ok(None),
            KeyEvent {
                code: KeyCode::Up,
                modifiers: _,
            } => Ok(Some(state)),
            KeyEvent {
                code: KeyCode::Down,
                modifiers: _,
            } => Ok(Some(state)),
            KeyEvent {
                code: KeyCode::Right,
                modifiers: _,
            } => Ok(Some(if let Some(point) = history.forward(state.point()) {
                let is_latest_commit = history.forward(point).is_none();
                let is_earliest_commit = history.backward(point).is_none();
                State::new(
                    point,
                    state.line_index(),
                    is_latest_commit,
                    is_earliest_commit,
                )
            } else {
                state
            })),
            KeyEvent {
                code: KeyCode::Left,
                modifiers: _,
            } => Ok(Some(if let Some(point) = history.backward(state.point()) {
                let is_latest_commit = history.forward(point).is_none();
                let is_earliest_commit = history.backward(point).is_none();
                State::new(
                    point,
                    state.line_index(),
                    is_latest_commit,
                    is_earliest_commit,
                )
            } else {
                state
            })),
            _ => Ok(Some(state)),
        },
        _ => Ok(Some(state)),
    }
}
