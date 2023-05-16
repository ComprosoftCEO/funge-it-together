use std::io::{self, Write};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::style::Stylize;
use crossterm::{cursor, QueueableCommand};

use super::{LevelSelectState, State};
use crate::global_state::{GlobalState, Statistics};

pub struct SuccessState {
  level_index: usize,
  statistics: Statistics,
}

impl SuccessState {
  pub fn new(level_index: usize, statistics: Statistics) -> Self {
    Self {
      level_index,
      statistics,
    }
  }
}

impl State for SuccessState {
  fn render(&mut self, global_state: &mut GlobalState) -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.queue(cursor::Hide)?;

    let level = &global_state.levels()[self.level_index];
    write!(stdout, "Level {} - {}", self.level_index + 1, level.name())?;
    stdout.queue(cursor::MoveToNextLine(2))?;
    write!(stdout, "{}", "☺☺☺ Success! ☺☺☺".green())?;
    stdout.queue(cursor::MoveToNextLine(2))?;

    write!(
      stdout,
      "{} {:.2}",
      "Average Cycles:".dark_yellow(),
      self.statistics.average_cycles()
    )?;
    stdout.queue(cursor::MoveToNextLine(1))?;
    write!(
      stdout,
      "{}   {}",
      "Symbols Used:".dark_cyan(),
      self.statistics.symbols_used()
    )?;
    stdout.queue(cursor::MoveToNextLine(3))?;
    write!(stdout, "{} Continue", "►".green())?;

    stdout.flush()?;

    Ok(())
  }

  fn execute(self: Box<Self>, _: &mut GlobalState) -> io::Result<Option<Box<dyn State>>> {
    loop {
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

          KeyCode::Enter => return Ok(Some(Box::new(LevelSelectState::new(self.level_index)))),

          _ => {},
        },

        _ => {},
      }
    }
  }
}
