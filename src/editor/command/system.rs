use crate::prelude::*;
use super::Move;
use crossterm::event::{
    KeyCode::{self, Char},
    KeyEvent, KeyModifiers,
};

#[derive(Clone, Copy)]
pub enum System {
    Save,
    Resize(Size),
    Quit,
    Dismiss,
    Search,
    Select(Move)
}

impl TryFrom<KeyEvent> for System {
    type Error = String;
    fn try_from(event: KeyEvent) -> Result<Self, Self::Error> {
        let KeyEvent {
            code, modifiers, ..
        } = event;

        if modifiers == KeyModifiers::CONTROL {
            match code {
                Char('q') => Ok(Self::Quit),
                Char('s') => Ok(Self::Save),
                Char('f') => Ok(Self::Search),
                _ => Err(format!("Unsupported CONTROL+{code:?} combination")),
            }
        } else if modifiers == KeyModifiers::SHIFT {
            match code {
                KeyCode::Up => Ok(Self::Select(Move::Up)),
                KeyCode::Down => Ok(Self::Select(Move::Down)),
                KeyCode::Left => Ok(Self::Select(Move::Left)),
                KeyCode::Right => Ok(Self::Select(Move::Right)),
                KeyCode::PageDown => Ok(Self::Select(Move::PageDown)),
                KeyCode::PageUp => Ok(Self::Select(Move::PageUp)),
                KeyCode::Home => Ok(Self::Select(Move::StartOfLine)),
                KeyCode::End => Ok(Self::Select(Move::EndOfLine)),
                _ => Err(format!("Unsupported code: {code:?}")),
            }
        } else if modifiers == KeyModifiers::NONE && matches!(code, KeyCode::Esc) {
            Ok(Self::Dismiss)
        } else {
            Err(format!(
                "Unsupported key code {code:?} or modifier {modifiers:?}"
            ))
        }
    }
}
