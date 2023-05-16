use std::io::{self, Write};

use crossterm::{
  cursor,
  event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
  style::Stylize,
  QueueableCommand,
};

use super::State;
use crate::global_state::GlobalState;

pub struct ShowTextState {
  text: String,
  prev_state: Box<dyn State>,
}

impl ShowTextState {
  pub fn new(text: impl Into<String>, prev_state: Box<dyn State>) -> Self {
    Self {
      text: text.into(),
      prev_state,
    }
  }
}

impl State for ShowTextState {
  fn render(&mut self, _: &mut GlobalState) -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.queue(cursor::Hide)?;

    for line in self.text.lines() {
      write!(stdout, "{}", line)?;
      stdout.queue(cursor::MoveToNextLine(1))?;
    }

    stdout.queue(cursor::MoveToNextLine(2))?;
    write!(stdout, "{} Continue", "►".green())?;
    stdout.flush()?;

    Ok(())
  }

  fn execute(self: Box<Self>, _: &mut GlobalState) -> io::Result<Option<Box<dyn State>>> {
    loop {
      // `read()` blocks until an `Event` is available
      let event = match event::read() {
        Ok(e) => e,
        Err(_) => return Ok(None),
      };

      match event {
        Event::Resize(_, _) => {
          return Ok(Some(self));
        },

        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
          KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Ok(None);
          },

          KeyCode::Enter | KeyCode::Esc => {
            return Ok(Some(self.prev_state));
          },

          _ => {},
        },
        _ => {},
      }
    }
  }
}
