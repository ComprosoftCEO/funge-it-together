use rlua::prelude::*;
use serde::Deserialize;
use std::error::Error;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::{self, BufReader, ErrorKind};
use std::iter;
use std::path::Path;
use uuid::Uuid;

use crate::global_state::GlobalState;
use crate::puzzle::{Puzzle, TestCaseSet};

/// Stores all details about the levels
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelPack {
  #[serde(rename = "levels")]
  groups: Vec<LevelGroup>,
}

/// Stores a group of levels that all unlock at once
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelGroup(Vec<MainLevel>);

/// Top-level object for a level, may have optional "challenge" levels
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MainLevel {
  #[serde(flatten)]
  level: Level,

  #[serde(default)]
  challenge_levels: Vec<Level>,
}

/// Single entry in the levels.json file
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Level {
  id: Uuid,
  name: String,
  description: String,
  lua_file: String,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LevelIndex {
  group: usize,
  level_in_group: usize,
  challenge: Option<usize>,
}

impl LevelPack {
  pub fn load() -> io::Result<Self> {
    Self::from_file("levels/pack.json")
  }

  pub fn level_groups(&self) -> &Vec<LevelGroup> {
    &self.groups
  }

  pub fn level_group(&self, index: usize) -> &LevelGroup {
    &self.groups[index]
  }

  pub fn get_absolute_index(&self, level_index: LevelIndex) -> usize {
    (0..level_index.group)
      .map(|group_index| self.groups[group_index].len())
      .sum::<usize>()
      + (0..level_index.level_in_group)
        .map(|level_in_group| self.groups[level_index.group].0[level_in_group].len())
        .sum::<usize>()
      + level_index.challenge.map(|x| x + 1).unwrap_or(0)
  }

  ///
  /// Load a level pack from a folder
  ///   Returns an error if there are no levels inside the pack file
  ///
  fn from_file<P: AsRef<Path>>(json_pack_file: P) -> io::Result<Self> {
    // Parse the level as a JSON file
    let file = File::open(json_pack_file)?;
    let reader = BufReader::new(file);
    let mut me: Self = serde_json::from_reader(reader)?;

    // Remove any groups that have no levels
    me.groups.retain(|g| !g.0.is_empty());

    // Make sure there is at least one level in one group
    if me.groups.is_empty() {
      Err(io::Error::new(
        ErrorKind::InvalidData,
        "No levels provided in pack file".to_string(),
      ))?;
    }

    Ok(me)
  }
}

impl LevelGroup {
  pub fn main_levels(&self) -> &Vec<MainLevel> {
    &self.0
  }

  pub fn main_level(&self, index: usize) -> &MainLevel {
    &self.0[index]
  }

  pub fn is_complete(&self, global_state: &GlobalState) -> bool {
    self.0.iter().all(|l| global_state.is_level_complete(l.level.id))
  }

  pub fn len(&self) -> usize {
    self.0.iter().map(MainLevel::len).sum()
  }
}

impl MainLevel {
  pub fn level(&self) -> &Level {
    &self.level
  }

  pub fn challenge_levels(&self) -> &Vec<Level> {
    &self.challenge_levels
  }

  pub fn challenge_level(&self, index: usize) -> &Level {
    &self.challenge_levels[index]
  }

  pub fn len(&self) -> usize {
    1 + self.challenge_levels.len()
  }
}

impl Level {
  pub fn id(&self) -> Uuid {
    self.id
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn description(&self) -> &str {
    &self.description
  }

  #[allow(unused)]
  pub fn lua_file(&self) -> &str {
    &self.lua_file
  }

  pub fn get_title(&self, level_index: LevelIndex) -> String {
    if level_index.challenge.is_some() {
      format!("Challenge {} - {}", level_index, self.name())
    } else {
      format!("Level {} - {}", level_index, self.name())
    }
  }

  ///
  /// Load and run the Lua code to generate the puzzles
  ///
  pub fn generate_test_cases(&self, seed: u32, n: usize) -> Result<TestCaseSet, Box<dyn Error>> {
    // Try to load the Lua code file into memory
    let lua_code = fs::read_to_string(format!("levels/{}", self.lua_file))?;

    // Generate and run the code within the Lua context
    let test_cases = Lua::new().context::<_, LuaResult<TestCaseSet>>(|ctx| {
      let globals = ctx.globals();

      // Add the levels folder to the path
      ctx
        .load(&r#"package.path = "./levels/?.lua;" .. package.path"#)
        .exec()?;

      // Seed the random number generator
      globals
        .get::<_, LuaTable>("math")?
        .get::<_, LuaFunction>("randomseed")?
        .call::<_, ()>(seed)?;

      // Load the script code
      //  This should define a global function named "generateTestCase"
      ctx.load(&lua_code).exec()?;

      // Generate the test cases one-by-one
      let generate_test_case: LuaFunction = globals.get("generateTestCase")?;
      let test_cases = (0..n)
        .map(|_| {
          let (inputs, outputs): (Vec<i16>, Vec<i16>) = generate_test_case.call(())?;
          Puzzle::new(inputs, outputs).map_err(LuaError::RuntimeError)
        })
        .collect::<Result<_, _>>()?;

      Ok(test_cases)
    })?;

    Ok(test_cases)
  }
}

impl LevelIndex {
  pub fn new(group: usize, level_in_group: usize) -> Self {
    Self {
      group,
      level_in_group,
      challenge: None,
    }
  }

  pub fn new_challenge(group: usize, level_in_group: usize, challenge: usize) -> Self {
    Self {
      group,
      level_in_group,
      challenge: Some(challenge),
    }
  }

  pub fn get_group(&self) -> usize {
    self.group
  }

  pub fn get_level_in_group(&self) -> usize {
    self.level_in_group
  }

  pub fn get_challenge(&self) -> Option<usize> {
    self.challenge
  }
}

impl fmt::Display for LevelIndex {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut counter = self.level_in_group;
    let level_index: String = iter::once(counter)
      .chain(iter::from_fn(|| {
        (counter >= 26).then(|| {
          counter /= 26;
          counter
        })
      }))
      .map(|c| ((c % 26) as u8 + b'A') as char)
      .collect::<String>()
      .chars()
      .rev()
      .collect();

    if let Some(challenge_index) = self.challenge {
      write!(f, "{}{}-{}", self.group + 1, level_index, challenge_index + 1)
    } else {
      write!(f, "{}{}", self.group + 1, level_index)
    }
  }
}
