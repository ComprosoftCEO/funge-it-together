use crossterm::style::Stylize;
use crossterm::{cursor, QueueableCommand};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

use crate::grid::Grid;
use crate::printable::Printable;
use crate::vm::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Solution {
  #[serde(default = "default_name")]
  name: String,
  programs: [Program; 2],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Program {
  grid: Grid,
  start_row: usize,
  start_col: usize,
}

fn default_name() -> String {
  "Solution 1".into()
}

#[allow(unused)]
impl Solution {
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      ..Default::default()
    }
  }

  pub fn name(&self) -> &String {
    &self.name
  }

  pub fn rename(&mut self, new_name: impl Into<String>) {
    self.name = new_name.into();
  }

  pub fn symbols_used(&self) -> usize {
    self.programs.iter().map(Program::symbols_used).sum()
  }

  pub fn program_0(&mut self) -> &mut Program {
    &mut self.programs[0]
  }

  pub fn program_1(&mut self) -> &mut Program {
    &mut self.programs[1]
  }

  pub fn into_programs(self) -> (Program, Program) {
    let [p0, p1] = self.programs;
    (p0, p1)
  }
}

impl Printable for Solution {
  fn print(&self) -> io::Result<()> {
    let mut stdout = io::stdout();

    self.programs[0].print()?;
    stdout.queue(cursor::MoveDown(self.programs[0].rows() as u16 + 1))?;
    self.programs[1].print()?;
    write!(stdout, "├{}┤", "─".repeat(self.programs[1].cols()))?;

    Ok(())
  }
}

impl Default for Solution {
  fn default() -> Self {
    Self {
      name: "New Solution".into(),
      programs: [Program::default(), Program::default()],
    }
  }
}

impl Program {
  pub fn into_grid(self) -> Grid {
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

  pub fn symbols_used(&self) -> usize {
    self.grid.count_symbols()
  }

  pub fn toggle_breakpoint(&mut self, row: usize, col: usize) {
    self.grid.toggle_breakpoint(row, col);
  }
}

impl Default for Program {
  fn default() -> Self {
    Self {
      grid: Grid::default(),
      start_row: 0,
      start_col: 0,
    }
  }
}

impl Printable for Program {
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

    stdout.queue(cursor::RestorePosition)?;

    Ok(())
  }
}
