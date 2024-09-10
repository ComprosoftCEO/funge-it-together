use crossterm::style::Stylize;
use crossterm::{cursor, QueueableCommand};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

use super::vm::Command;
use crate::grid::Grid;
use crate::isa;
use crate::printable::Printable;

// Standard levels use a 10x10 grid
const GRID_SIZE: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Solution {
  #[serde(default = "default_name")]
  name: String,
  grid: Grid<Command>,
  start_row: usize,
  start_col: usize,
}

fn default_name() -> String {
  "Solution 1".into()
}

impl isa::Solution for Solution {
  fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      ..Default::default()
    }
  }

  fn name(&self) -> &str {
    &self.name
  }

  fn rename(&mut self, new_name: impl Into<String>) {
    self.name = new_name.into();
  }

  fn symbols_used(&self) -> usize {
    self.grid.count_symbols()
  }
}

#[allow(unused)]
impl Solution {
  pub fn into_grid(self) -> Grid<Command> {
    self.grid
  }

  pub fn rows(&self) -> usize {
    self.grid.rows()
  }

  pub fn cols(&self) -> usize {
    self.grid.cols()
  }

  pub fn set_grid_value(&mut self, row: usize, col: usize, value: Command) {
    self.grid.set_value(row, col, value);
  }

  pub fn start_row(&self) -> usize {
    self.start_row
  }

  pub fn start_col(&self) -> usize {
    self.start_col
  }

  pub fn set_start(&mut self, start_row: usize, start_col: usize) {
    debug_assert!(self.start_row < self.grid.rows());
    debug_assert!(self.start_col < self.grid.cols());

    self.start_row = start_row;
    self.start_col = start_col;
  }

  pub fn toggle_breakpoint(&mut self, row: usize, col: usize) {
    self.grid.toggle_breakpoint(row, col);
  }
}

impl Default for Solution {
  fn default() -> Self {
    Self {
      name: "New Solution".into(),
      grid: Grid::new(GRID_SIZE, GRID_SIZE),
      start_row: 0,
      start_col: 0,
    }
  }
}

impl Printable for Solution {
  fn print(&self) -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.queue(cursor::SavePosition)?;
    self.grid.print()?;

    stdout
      .queue(cursor::RestorePosition)?
      .queue(cursor::MoveRight(self.start_col as u16 + 1))?
      .queue(cursor::MoveDown(self.start_row as u16 + 1))?;

    write!(
      stdout,
      "{}",
      self
        .grid
        .get_value(self.start_row, self.start_col)
        .get_char()
        .green()
        .reverse()
    )?;

    Ok(())
  }
}
