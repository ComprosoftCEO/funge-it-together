use std::io::{self, Stdout, Write};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::style::Stylize;
use crossterm::{cursor, QueueableCommand};

use super::{LevelSelectState, State};
use crate::global_state::GlobalState;
use crate::level::LevelIndex;
use crate::statistics::Statistics;

pub struct SuccessState {
  level_index: LevelIndex,
  statistics: Statistics,
  best: Statistics,
  editor: Box<dyn State>,
}

impl SuccessState {
  pub fn new(level_index: LevelIndex, statistics: Statistics, best: Statistics, editor: Box<dyn State>) -> Self {
    Self {
      level_index,
      statistics,
      best,
      editor,
    }
  }
}

impl State for SuccessState {
  fn render(&mut self, global_state: &mut GlobalState) -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.queue(cursor::Hide)?;

    let level = &global_state.level(self.level_index);
    write!(stdout, "{}", level.get_title(self.level_index).yellow())?;
    stdout.queue(cursor::MoveToNextLine(2))?;
    write!(stdout, "{}", "☺☺☺ Success! ☺☺☺".green())?;
    stdout.queue(cursor::MoveToNextLine(3))?;

    write!(stdout, "Current Solution:")?;
    stdout.queue(cursor::MoveToNextLine(1))?;
    write_statistics(&mut stdout, &self.statistics)?;

    write!(stdout, "Personal Best:")?;
    stdout.queue(cursor::MoveToNextLine(1))?;
    write_statistics(&mut stdout, &self.best)?;

    stdout.queue(cursor::MoveToNextLine(1))?;
    write!(stdout, "{} Continue", "►".green())?;

    stdout.flush()?;

    Ok(())
  }

  fn execute(self: Box<Self>, state: &mut GlobalState) -> io::Result<Option<Box<dyn State>>> {
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

          KeyCode::Enter => return Ok(Some(Box::new(LevelSelectState::new(self.level_index, state)))),
          KeyCode::Esc => return Ok(Some(self.editor)),

          _ => {},
        },

        _ => {},
      }
    }
  }
}

fn write_statistics(stdout: &mut Stdout, statistics: &Statistics) -> io::Result<()> {
  write!(
    stdout,
    "∙ {} {:.2}",
    "Average Cycles:".dark_yellow(),
    statistics.average_cycles()
  )?;
  stdout.queue(cursor::MoveToNextLine(1))?;

  write!(
    stdout,
    "∙ {}   {}",
    "Symbols Used:".dark_cyan(),
    statistics.symbols_used()
  )?;
  stdout.queue(cursor::MoveToNextLine(2))?;

  Ok(())
}
