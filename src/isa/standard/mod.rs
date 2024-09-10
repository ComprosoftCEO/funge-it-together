use crate::isa::InstructionSetArchitecture;
use editor_state::EditorState;
use rlua::prelude::*;
use std::error::Error;
use std::fs;

mod editor_state;
mod execute_state;
mod puzzle;
mod solution;
mod vm;

pub use solution::Solution;

/// Standard level type
pub struct Standard;

impl InstructionSetArchitecture for Standard {
  type Solution = Solution;
  type Puzzle = puzzle::Puzzle;

  ///
  /// Load and run the Lua code to generate the puzzles
  ///
  fn generate_test_cases(
    folder: &str,
    lua_file: &str,
    seed: u32,
    n: usize,
  ) -> Result<Vec<Self::Puzzle>, Box<dyn Error>> {
    // Try to load the Lua code file into memory
    let lua_code = fs::read_to_string(format!("{folder}/{lua_file}"))?;

    // Generate and run the code within the Lua context
    let test_cases = Lua::new().context::<_, LuaResult<Vec<Self::Puzzle>>>(|ctx| {
      let globals = ctx.globals();

      // Add the levels folder to the path
      ctx
        .load(&format!(r#"package.path = "./{folder}/?.lua;" .. package.path"#))
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
          Self::Puzzle::new(inputs, outputs).map_err(LuaError::RuntimeError)
        })
        .collect::<Result<_, _>>()?;

      Ok(test_cases)
    })?;

    Ok(test_cases)
  }

  fn open_editor(
    level_index: crate::level::LevelIndex,
    solution_index: usize,
    solution: Self::Solution,
    test_cases: Vec<Self::Puzzle>,
    test_case_index: usize,
  ) -> impl crate::state::State {
    EditorState::new(level_index, solution_index, solution, test_cases, test_case_index)
  }
}
