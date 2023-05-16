use rlua::prelude::*;
use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{self, BufReader, ErrorKind};
use std::path::Path;
use uuid::Uuid;

use crate::puzzle::{Puzzle, TestCaseSet};

static DEFAULT_WIN_MESSAGE: &str = "Congratulations! You solved all puzzles. Good job!";

/// Stores all details about the levels
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Levels {
  levels: Vec<Level>,
  win_message: Option<String>,
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
impl Levels {
  pub fn load() -> io::Result<Self> {
    Self::from_file("levels/pack.json")
  }

  pub fn levels(&self) -> &Vec<Level> {
    &self.levels
  }

  pub fn level(&self, index: usize) -> &Level {
    &self.levels[index]
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

    // Make sure there is at least one level
    if me.levels.len() == 0 {
      Err(io::Error::new(
        ErrorKind::InvalidData,
        format!("No levels provided in pack file"),
      ))?;
    }

    Ok(me)
  }

  pub fn win_message(&self) -> &str {
    self
      .win_message
      .as_ref()
      .map(String::as_str)
      .unwrap_or(DEFAULT_WIN_MESSAGE)
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

  pub fn get_full_text(&self, level_index: usize) -> String {
    format!("Level {} - {}\n\n{}", level_index + 1, self.name(), self.description())
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
