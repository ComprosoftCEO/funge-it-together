use rlua::prelude::*;
use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{self, BufReader, ErrorKind};
use std::path::Path;
use uuid::Uuid;

use crate::global_state::{GlobalState, LevelIndex};
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
pub struct LevelGroup {
  #[serde(flatten)]
  levels: Vec<MainLevel>,
}

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
    me.groups = me.groups.into_iter().filter(|g| !g.levels.is_empty()).collect();

    // Make sure there is at least one level in one group
    if me.groups.is_empty() {
      Err(io::Error::new(
        ErrorKind::InvalidData,
        format!("No levels provided in pack file"),
      ))?;
    }

    Ok(me)
  }
}

impl LevelGroup {
  pub fn main_levels(&self) -> &Vec<MainLevel> {
    &self.levels
  }

  pub fn main_level(&self, index: usize) -> &MainLevel {
    &self.levels[index]
  }

  pub fn is_complete(&self, global_state: &GlobalState) -> bool {
    self.levels.iter().all(|l| global_state.is_level_complete(l.level.id))
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

  pub fn lua_file(&self) -> &str {
    &self.lua_file
  }

  pub fn get_title(&self, level_index: LevelIndex) -> String {
    format!("Puzzle {} - {}", level_index, self.name())
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
        .load(&format!(r#"package.path = "./levels/?.lua;" .. package.path"#))
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
          Puzzle::new(inputs, outputs).map_err(|e| LuaError::RuntimeError(e))
        })
        .collect::<Result<_, _>>()?;

      Ok(test_cases)
    })?;

    Ok(test_cases)
  }
}
