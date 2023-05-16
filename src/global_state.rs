use crossterm::style::Stylize;
use crossterm::{cursor, QueueableCommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Write};
use std::path::Path;
use uuid::Uuid;

use crate::grid::Grid;
use crate::level::{Level, Levels};
use crate::printable::Printable;
use crate::vm::Command;

static SAVE_FILE: &str = "save.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalState {
  solutions: HashMap<Uuid, Solution>,
  unlocked: HashMap<Uuid, Statistics>,

  #[serde(skip)]
  levels: Levels,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Solution {
  grid: Grid,
  start_row: usize,
  start_col: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
  average_cycles: f64,
  symbols_used: usize,
}

impl GlobalState {
  pub fn load(levels: Levels) -> Self {
    let mut state = Self::from_file(SAVE_FILE).unwrap_or_default();
    state.levels = levels;
    state
  }

  fn from_file<P: AsRef<Path>>(json_save_file: P) -> io::Result<Self> {
    // Parse the level as a JSON file
    let file = File::open(json_save_file)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
  }

  pub fn save(&self) -> io::Result<()> {
    let file = File::create(SAVE_FILE)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, self)?;
    Ok(())
  }

  pub fn message(&self) -> &str {
    self.levels.win_message()
  }

  pub fn level(&self, index: usize) -> &Level {
    self.levels.level(index)
  }

  pub fn levels(&self) -> &Vec<Level> {
    self.levels.levels()
  }

  pub fn completed_levels(&self) -> Vec<Level> {
    self
      .levels()
      .iter()
      .take_while(|l| self.is_level_complete(l.id()))
      .cloned()
      .collect()
  }

  pub fn save_solution(&mut self, level_id: Uuid, solution: &Solution) {
    self.solutions.insert(level_id, solution.clone());
  }

  // Returns an empty grid if no solution exists
  pub fn get_solution(&self, level_id: Uuid) -> Solution {
    self.solutions.get(&level_id).cloned().unwrap_or_default()
  }

  pub fn is_level_complete(&self, level_id: Uuid) -> bool {
    self.unlocked.contains_key(&level_id)
  }

  pub fn get_statistics(&self, level_id: Uuid) -> Option<Statistics> {
    self.unlocked.get(&level_id).cloned()
  }

  pub fn complete_level(&mut self, level_id: Uuid, statistics: Statistics) {
    self.unlocked.insert(level_id, statistics);
  }
}

#[allow(unused)]
impl Solution {
  pub fn new(grid: Grid, start_row: usize, start_col: usize) -> Self {
    Self {
      grid,
      start_row,
      start_col,
    }
  }

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
        .get_value(self.start_row as usize, self.start_col as usize)
        .get_char()
        .green()
        .reverse()
    )?;

    Ok(())
  }
}

impl Statistics {
  pub fn new(average_cycles: f64, symbols_used: usize) -> Self {
    Self {
      average_cycles,
      symbols_used,
    }
  }

  pub fn average_cycles(&self) -> f64 {
    self.average_cycles
  }

  pub fn symbols_used(&self) -> usize {
    self.symbols_used
  }
}
