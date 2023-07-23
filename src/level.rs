use rlua::prelude::*;
use serde::{Deserialize, Deserializer};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{self, BufReader, ErrorKind};
use std::path::Path;
use uuid::Uuid;

use crate::puzzle::{Puzzle, TestCaseSet};

/// Stores all details about the levels
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelPack {
  groups: Vec<LevelGroup>,
}

/// Stores a group of levels that all unlock at once
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelGroup {
  // If unset, requires all levels to unlock
  #[serde(default)]
  requred_to_unlock: UnlockRequirements,

  levels: Vec<MainLevel>,
}

/// Top-level object for a level
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MainLevel {
  #[serde(flatten)]
  level: Level,
  challenge_levels: Vec<Level>,
}

/// Stores the unlock requirements
#[derive(Debug, Clone, Copy)]
pub enum UnlockRequirements {
  /// Complete at least X levels from the previous group to unlock.
  /// This is indicated by a positive number.
  AtLeast(u16),

  /// Complete n - x levels from the previous group to unlock,
  /// where n is the number of levels in the previous group.
  /// This is indicated by a negative number or 0.
  ///
  /// If set to 0, it requires all levels to be completed.
  AllExcept(u16),
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

#[allow(unused)]
impl LevelPack {
  pub fn load() -> io::Result<Self> {
    Self::from_file("levels/pack.json")
  }

  // pub fn levels(&self) -> &Vec<Level> {
  //   &self.levels
  // }

  // pub fn level(&self, index: usize) -> &Level {
  //   &self.levels[index]
  // }

  ///
  /// Load a level pack from a folder
  ///   Returns an error if there are no levels inside the pack file
  ///
  fn from_file<P: AsRef<Path>>(json_pack_file: P) -> io::Result<Self> {
    // Parse the level as a JSON file
    let file = File::open(json_pack_file)?;
    let reader = BufReader::new(file);
    let mut me: Self = serde_json::from_reader(reader)?;

    // Make sure there is at least one level in one group
    if me.groups.iter().find(|group| group.levels.len() > 0).is_none() {
      Err(io::Error::new(
        ErrorKind::InvalidData,
        format!("No levels provided in pack file"),
      ))?;
    }

    Ok(me)
  }
}

impl Default for UnlockRequirements {
  fn default() -> Self {
    Self::AllExcept(0)
  }
}

impl<'de> Deserialize<'de> for UnlockRequirements {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let val: i16 = Deserialize::deserialize(deserializer)?;
    if val <= 0 {
      Ok(Self::AllExcept(val.abs() as u16))
    } else {
      Ok(Self::AtLeast(val as u16))
    }
  }
}

#[allow(unused)]
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

  pub fn get_title(&self, level_index: usize) -> String {
    format!("Level {} - {}", level_index + 1, self.name())
  }

  ///
  /// Print the full level details along with some examples
  ///
  pub fn print_level_details(&self, level_number: usize) {
    println!("Level {}: {}", level_number, self.name);
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
