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
                code: KeyCode::Left,
                modifiers: _,
            } => Ok(Some(state.backward_commit(history))),
            KeyEvent {
                code: KeyCode::Right,
                modifiers: _,
            } => Ok(Some(state.forward_commit(history))),
            KeyEvent {
                code: KeyCode::Up,
                modifiers: _,
            } => Ok(Some(state.decrement_line_index())),
            KeyEvent {
                code: KeyCode::Down,
                modifiers: _,
            } => Ok(Some(state.increment_line_index())),
            _ => Ok(Some(state)),
        },
        Event::Resize(_width, height) => {
            Ok(Some(state.update_terminal_height(usize::from(height))))
        }
        _ => Ok(Some(state)),
    }
}
