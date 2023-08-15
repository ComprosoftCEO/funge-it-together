use crossterm::style::{self, Color};
use crossterm::{cursor, QueueableCommand};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::{self, Write};

use crate::printable::Printable;
use crate::vm::Command;

const DEFAULT_GRID_SIZE: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
  values: Vec<Vec<Command>>,

  #[serde(default)]
  breakpoints: HashSet<(usize, usize)>,
}

impl Grid {
  pub fn new(rows: usize, cols: usize) -> Self {
    debug_assert!(rows > 0);
    debug_assert!(cols > 0);

    Self {
      values: vec![vec![Command::Empty; cols]; rows],
      breakpoints: HashSet::new(),
    }
  }

  #[inline]
  pub fn rows(&self) -> usize {
    self.values.len()
  }

  #[inline]
  pub fn cols(&self) -> usize {
    self.values[0].len()
  }

  #[inline]
  pub fn set_value(&mut self, row: usize, col: usize, value: Command) {
    self.values[row][col] = value;
  }

  #[inline]
  pub fn get_value(&self, row: usize, col: usize) -> Command {
    self.values[row][col]
  }

  pub fn has_breakpoint(&self, row: usize, col: usize) -> bool {
    self.breakpoints.contains(&(row, col))
  }

  pub fn toggle_breakpoint(&mut self, row: usize, col: usize) {
    debug_assert!(row < self.rows());
    debug_assert!(col < self.rows());

    let point = (row, col);
    if self.breakpoints.contains(&point) {
      self.breakpoints.remove(&point);
    } else {
      self.breakpoints.insert(point);
    }
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
    let top_bottom_lines: String = "─".repeat(cols);
    write!(stdout, "┌{}┐", top_bottom_lines)?;
    stdout.queue(cursor::MoveLeft(cols as u16 + 2))?;
    stdout.queue(cursor::MoveDown(1))?;

    let mut in_breakpoint = false;
    for (row_index, row) in self.values.iter().enumerate() {
      write!(stdout, "│")?;
      for (col_index, command) in row.iter().enumerate() {
        if self.has_breakpoint(row_index, col_index) {
          if !in_breakpoint {
            stdout
              .queue(style::SetBackgroundColor(Color::DarkCyan))?
              .queue(style::SetForegroundColor(Color::Black))?;
            in_breakpoint = true;
          }
        } else if in_breakpoint {
          stdout.queue(style::ResetColor)?;
          in_breakpoint = false;
        }

        command.print()?;
      }

      if in_breakpoint {
        stdout.queue(style::ResetColor)?;
        in_breakpoint = false;
      }
      write!(stdout, "│")?;

      stdout.queue(cursor::MoveLeft(cols as u16 + 2))?;
      stdout.queue(cursor::MoveDown(1))?;
    }

    write!(stdout, "└{}┘", top_bottom_lines)?;

    Ok(())
  }
}
