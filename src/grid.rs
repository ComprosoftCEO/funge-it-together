use crossterm::{cursor, QueueableCommand};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::iter;

use crate::printable::Printable;
use crate::vm::Command;

const DEFAULT_GRID_SIZE: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
  values: Vec<Vec<Command>>,
}

impl Grid {
  pub fn new(rows: usize, cols: usize) -> Self {
    debug_assert!(rows > 0);
    debug_assert!(cols > 0);

    Self {
      values: vec![vec![Command::Empty; cols]; rows],
    }
  }

  pub fn rows(&self) -> usize {
    self.values.len()
  }

  pub fn cols(&self) -> usize {
    self.values[0].len()
  }

  pub fn set_value(&mut self, row: usize, col: usize, value: Command) {
    self.values[row][col] = value;
  }

  pub fn get_value(&self, row: usize, col: usize) -> Command {
    self.values[row][col]
  }

  pub fn count_symbols(&self) -> usize {
    self
      .values
      .iter()
      .map(|row| row.iter().filter(|x| **x != Command::Empty).count())
      .sum()
  }
}

impl Default for Grid {
  fn default() -> Self {
    Self::new(DEFAULT_GRID_SIZE, DEFAULT_GRID_SIZE)
  }
}

impl Printable for Grid {
  fn print(&self) -> io::Result<()> {
    let cols = self.values[0].len();
    let mut stdout = io::stdout();

    // ┌─┐
    // │ │
    // └─┘
    let top_bottom_lines: String = iter::repeat("─").take(cols).collect();
    write!(stdout, "┌{}┐", top_bottom_lines)?;
    stdout.queue(cursor::MoveLeft(cols as u16 + 2))?;
    stdout.queue(cursor::MoveDown(1))?;

    for row in self.values.iter() {
      write!(stdout, "│")?;
      for command in row {
        command.print()?;
      }
      write!(stdout, "│")?;

      stdout.queue(cursor::MoveLeft(cols as u16 + 2))?;
      stdout.queue(cursor::MoveDown(1))?;
    }

    write!(stdout, "└{}┘", top_bottom_lines)?;

    Ok(())
  }
}
