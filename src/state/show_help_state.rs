use std::io::{self, Write};

use crossterm::{
  cursor,
  event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
  style::Stylize,
  QueueableCommand,
};

use super::State;
use crate::global_state::GlobalState;

pub struct ShowHelpState {
  title: String,
  text: String,
  next_state: Box<dyn State>,
  back_state: Option<Box<dyn State>>,
}

impl ShowHelpState {
  pub fn new(
    title: impl Into<String>,
    text: impl Into<String>,
    next_state: Box<dyn State>,
    back_state: Option<Box<dyn State>>,
  ) -> Self {
    Self {
      title: title.into(),
      text: text.into(),
      next_state,
      back_state,
    }
  }
}

impl State for ShowHelpState {
  fn render(&mut self, _: &mut GlobalState) -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.queue(cursor::Hide)?;

    write!(stdout, "{}", self.title.as_str().yellow())?;
    stdout.queue(cursor::MoveToNextLine(2))?;

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

          KeyCode::Enter => {
            return Ok(Some(self.next_state));
          },

          KeyCode::Esc => {
            return Ok(Some(self.back_state.unwrap_or(self.next_state)));
          },

          _ => {},
        },
        _ => {},
      }
    }
  }
}
