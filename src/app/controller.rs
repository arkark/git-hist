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
            } => Ok(Some(state.move_line_up())),
            KeyEvent {
                code: KeyCode::Down,
                modifiers: _,
            } => Ok(Some(state.move_line_down())),
            KeyEvent {
                code: KeyCode::PageUp,
                modifiers: _,
            } => Ok(Some(state.move_line_up_for_page())),
            KeyEvent {
                code: KeyCode::PageDown,
                modifiers: _,
            } => Ok(Some(state.move_line_down_for_page())),
            KeyEvent {
                code: KeyCode::Home,
                modifiers: _,
            } => Ok(Some(state.move_line_to_top())),
            KeyEvent {
                code: KeyCode::End,
                modifiers: _,
            } => Ok(Some(state.move_line_to_bottom())),
            _ => Ok(Some(state)),
        },
        Event::Resize(_width, height) => {
            Ok(Some(state.update_terminal_height(usize::from(height))))
        }
        _ => Ok(Some(state)),
    }
}
