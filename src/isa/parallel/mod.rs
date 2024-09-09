use crate::isa::InstructionSetArchitecture;
use puzzle::ProcessorIO;
use rlua::prelude::*;
use std::error::Error;
use std::fs;

mod editor_state;
mod execute_state;
mod puzzle;
mod solution;
mod vm;

pub use solution::Solution;

/// Parallel level type
pub struct Parallel;

impl InstructionSetArchitecture for Parallel {
  type Solution = Solution;
  type Puzzle = puzzle::Puzzle;

  ///
  /// Load and run the Lua code to generate the puzzles
  ///
  fn generate_test_cases(lua_file: &str, seed: u32, n: usize) -> Result<Vec<Self::Puzzle>, Box<dyn Error>> {
    // Try to load the Lua code file into memory
    let lua_code = fs::read_to_string(format!("levels/{}", lua_file))?;

    // Generate and run the code within the Lua context
    let test_cases = Lua::new().context::<_, LuaResult<Vec<Self::Puzzle>>>(|ctx| {
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
          let (p0_inputs, p0_outputs, p1_inputs, p1_outputs): (Vec<i16>, Vec<i16>, Vec<i16>, Vec<i16>) =
            generate_test_case.call(())?;

          let p0 = ProcessorIO::new(p0_inputs, p0_outputs).map_err(|e| LuaError::RuntimeError(e))?;
          let p1 = ProcessorIO::new(p1_inputs, p1_outputs).map_err(|e| LuaError::RuntimeError(e))?;
          Ok(Self::Puzzle::new(p0, p1))
        })
        .collect::<Result<_, _>>()?;

      Ok(test_cases)
    })?;

    Ok(test_cases)
  }
}
