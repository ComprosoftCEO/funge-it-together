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
  solutions: HashMap<Uuid, Vec<Solution>>,
  unlocked: HashMap<Uuid, Statistics>,

  #[serde(skip)]
  levels: Levels,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Solution {
  name: String,
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

  // Create a new solution and return it's index
  pub fn new_solution(&mut self, level_id: Uuid) -> usize {
    let vec = self.solutions.entry(level_id).or_default();
    vec.push(Solution::new(format!("Solution {}", vec.len() + 1)));
    vec.len()
  }

  pub fn save_solution(&mut self, level_id: Uuid, solution_index: usize, solution: &Solution) {
    self.solutions.get_mut(&level_id).unwrap()[solution_index] = solution.clone();
  }

  pub fn get_solutions_mut(&mut self, level_id: Uuid) -> &mut Vec<Solution> {
    self.solutions.entry(level_id).or_default()
  }

  pub fn is_level_complete(&self, level_id: Uuid) -> bool {
    self.unlocked.contains_key(&level_id)
  }

  pub fn get_statistics(&self, level_id: Uuid) -> Option<Statistics> {
    self.unlocked.get(&level_id).cloned()
  }

  // Returns the best statistics overall
  pub fn complete_level(&mut self, level_id: Uuid, statistics: Statistics) -> Statistics {
    let best = self
      .unlocked
      .entry(level_id)
      .and_modify(|s| s.set_to_best(&statistics))
      .or_insert(statistics)
      .clone();

    // A level complete is very important, so ALWAYS try to save right away!
    // Silently ignore any errors (hopefully won't happen in practice)
    self.save().ok();

    best
  }
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
}

impl Default for Solution {
  fn default() -> Self {
    Self {
      name: "New Solution".into(),
      grid: Grid::default(),
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

  pub fn set_to_best(&mut self, other: &Statistics) {
    if other.average_cycles < self.average_cycles {
      self.average_cycles = other.average_cycles;
    }
    if other.symbols_used < self.symbols_used {
      self.symbols_used = other.symbols_used;
    }
  }
}
