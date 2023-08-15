use crossterm::style::Stylize;
use crossterm::{cursor, QueueableCommand};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{self, BufReader, BufWriter, Write};
use std::path::Path;
use uuid::Uuid;

use crate::grid::Grid;
use crate::level::{Level, LevelIndex, LevelPack};
use crate::printable::Printable;
use crate::vm::Command;

static SAVE_FILE: &str = "save.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalState {
  #[serde(deserialize_with = "deserialize_one_or_many_map")]
  solutions: HashMap<Uuid, Vec<Solution>>,
  unlocked: HashMap<Uuid, Statistics>,

  #[serde(skip)]
  pack: LevelPack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Solution {
  #[serde(default = "default_name")]
  name: String,
  grid: Grid,
  start_row: usize,
  start_col: usize,
}

fn default_name() -> String {
  "Solution 1".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
  average_cycles: f64,
  symbols_used: usize,
}

impl GlobalState {
  pub fn load(levels: LevelPack) -> Self {
    let mut state = Self::from_file(SAVE_FILE).unwrap_or_default();
    state.pack = levels;
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

  pub fn get_pack(&self) -> &LevelPack {
    &self.pack
  }

  pub fn level(&self, index: LevelIndex) -> &Level {
    let main_level = self
      .pack
      .level_group(index.get_group())
      .main_level(index.get_level_in_group());
    if let Some(challenge) = index.get_challenge() {
      main_level.challenge_level(challenge)
    } else {
      main_level.level()
    }
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

  pub fn toggle_breakpoint(&mut self, row: usize, col: usize) {
    self.grid.toggle_breakpoint(row, col);
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
        .get_value(self.start_row, self.start_col)
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

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum OneOrManyMap<K, V>
where
  K: Eq + Hash,
{
  One(HashMap<K, V>),
  Vec(HashMap<K, Vec<V>>),
}

impl<K, V> From<OneOrManyMap<K, V>> for HashMap<K, Vec<V>>
where
  K: Eq + Hash,
{
  fn from(from: OneOrManyMap<K, V>) -> Self {
    match from {
      OneOrManyMap::One(val) => val.into_iter().map(|(k, v)| (k, vec![v])).collect(),
      OneOrManyMap::Vec(vec) => vec,
    }
  }
}

fn deserialize_one_or_many_map<'de, D, K, V>(deserializer: D) -> Result<HashMap<K, Vec<V>>, D::Error>
where
  D: Deserializer<'de>,
  K: Eq + Hash + Deserialize<'de>,
  V: Deserialize<'de>,
{
  Ok(OneOrManyMap::<K, V>::deserialize(deserializer)?.into())
}
