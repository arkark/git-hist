use super::history::{History, TurningPoint};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug)]
pub struct State<'a> {
    point: &'a TurningPoint,
    line_index: usize,
}

impl<'a> State<'a> {
    pub fn new(point: &'a TurningPoint, line_index: usize) -> Self {
        Self { point, line_index }
    }

    pub fn poll_next_event(self, history: &'a History) -> Result<Option<State<'a>>> {
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
                } => Ok(Some(self)),
                KeyEvent {
                    code: KeyCode::Down,
                    modifiers: _,
                } => Ok(Some(self)),
                KeyEvent {
                    code: KeyCode::Right,
                    modifiers: _,
                } => Ok(Some(if let Some(point) = history.forward(self.point) {
                    State::new(point, self.line_index)
                } else {
                    self
                })),
                KeyEvent {
                    code: KeyCode::Left,
                    modifiers: _,
                } => Ok(Some(if let Some(point) = history.backward(self.point) {
                    State::new(point, self.line_index)
                } else {
                    self
                })),
                _ => Ok(Some(self)),
            },
            _ => Ok(Some(self)),
        }
    }

    pub fn point(&self) -> &TurningPoint {
        self.point
    }

    pub fn line_index(&self) -> usize {
        self.line_index
    }
}
