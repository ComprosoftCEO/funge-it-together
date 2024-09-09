use crossterm::style::Stylize;
use crossterm::{cursor, QueueableCommand};
use rand::Rng;
use rlua::prelude::*;
use std::collections::VecDeque;
use std::error::Error;
use std::fs;
use std::io::{self, Write};

use super::vm::{VAL_CHAR_WIDTH, VAL_MAX, VAL_MIN};
use crate::printable::Printable;

pub const MAX_PUZZLE_VALUES: usize = 8;

pub type TestCaseSet = Vec<Puzzle>;

#[derive(Debug, Clone)]
pub struct Puzzle {
  processor_io: [ProcessorIO; 2],
}

#[derive(Debug, Clone)]
pub struct ProcessorIO {
  inputs: PuzzleIO,
  outputs: PuzzleIO,
}

impl Puzzle {
  pub fn new(processor_0: ProcessorIO, processor_1: ProcessorIO) -> Self {
    Self {
      processor_io: [processor_0, processor_1],
    }
  }

  pub fn into_processor_ios(self) -> (ProcessorIO, ProcessorIO) {
    let [p0, p1] = self.processor_io;
    (p0, p1)
  }
}

impl ProcessorIO {
  // Performs validation and returns a printable error string
  pub fn new(inputs: Vec<i16>, outputs: Vec<i16>) -> Result<Self, String> {
    if inputs.len() > MAX_PUZZLE_VALUES {
      return Err(format!(
        "Too many input values, maximum of {MAX_PUZZLE_VALUES} allowed, {} given",
        inputs.len()
      ));
    }
    if outputs.len() > MAX_PUZZLE_VALUES {
      return Err(format!(
        "Too many output values, maximum of {MAX_PUZZLE_VALUES} allowed, {} given",
        outputs.len()
      ));
    }

    for val in inputs.iter() {
      if !(VAL_MIN..=VAL_MAX).contains(val) {
        return Err(format!("Input {val} outside range [-999,999]"));
      }
    }
    for val in outputs.iter() {
      if !(VAL_MIN..=VAL_MAX).contains(val) {
        return Err(format!("Output {val} outside range [-999,999]"));
      }
    }

    Ok(Self {
      inputs: PuzzleIO(inputs.into()),
      outputs: PuzzleIO(outputs.into()),
    })
  }

  pub fn get_inputs(&self) -> &PuzzleIO {
    &self.inputs
  }

  pub fn get_outputs(&self) -> &PuzzleIO {
    &self.outputs
  }
}

impl Printable for Puzzle {
  fn print(&self) -> io::Result<()> {
    let mut stdout = io::stdout();

    const HEADER: &str = "Input  Output";
    write!(stdout, "{}", HEADER)?;
    stdout
      .queue(cursor::MoveLeft(HEADER.len() as u16))?
      .queue(cursor::MoveDown(1))?
      .queue(cursor::SavePosition)?;

    self.processor_io[0].inputs.print()?;

    stdout
      .queue(cursor::RestorePosition)?
      .queue(cursor::MoveDown(MAX_PUZZLE_VALUES as u16 + 1))?;
    self.processor_io[1].inputs.print()?;

    stdout
      .queue(cursor::RestorePosition)?
      .queue(cursor::MoveRight(7))?
      .queue(cursor::SavePosition)?;

    self.processor_io[0].outputs.print()?;
    stdout
      .queue(cursor::RestorePosition)?
      .queue(cursor::MoveDown(MAX_PUZZLE_VALUES as u16 + 1))?;

    self.processor_io[1].outputs.print()?;

    stdout
      .queue(cursor::RestorePosition)?
      .queue(cursor::MoveLeft(7))?
      .queue(cursor::MoveDown(MAX_PUZZLE_VALUES as u16 + 1))?;

    write!(stdout, "├{0}┤ ├{0}┤", "─".repeat(VAL_CHAR_WIDTH))?;

    Ok(())
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PuzzleIO(VecDeque<i16>);

#[allow(unused)]
impl PuzzleIO {
  pub fn new() -> Self {
    Self(VecDeque::new())
  }

  pub fn new_random() -> Self {
    let mut rng = rand::thread_rng();
    Self((0..rng.gen_range(0..=10)).map(|_| rng.gen_range(-999..=999)).collect())
  }

  pub fn len(&self) -> usize {
    self.0.len()
  }

  pub fn can_read(&self) -> bool {
    !self.0.is_empty()
  }

  pub fn read(&mut self) -> Option<i16> {
    self.0.pop_front()
  }

  // Returns false if the stack overflows
  pub fn write(&mut self, val: i16) -> bool {
    if self.0.len() < MAX_PUZZLE_VALUES {
      self.0.push_back(val);
      true
    } else {
      false
    }
  }

  pub(crate) fn print_with_expected_outputs(&self, expected_outputs: &PuzzleIO) -> io::Result<()> {
    let mut stdout = io::stdout();
    // ┌─┐
    // │ │
    // └─┘
    let top_bottom_lines: String = "─".repeat(VAL_CHAR_WIDTH);
    write!(stdout, "┌{}┐", top_bottom_lines)?;
    stdout
      .queue(cursor::MoveLeft(VAL_CHAR_WIDTH as u16 + 2))?
      .queue(cursor::MoveDown(1))?;

    for (i, value) in self.0.iter().enumerate() {
      let text = format!("{:-4}", value);
      match expected_outputs.0.get(i) {
        Some(x) if x == value => write!(stdout, "│{}│", text),
        Some(_) | None => write!(stdout, "│{}│", text.red()),
      }?;

      stdout
        .queue(cursor::MoveLeft(VAL_CHAR_WIDTH as u16 + 2))?
        .queue(cursor::MoveDown(1))?;
    }

    for _ in self.0.len()..MAX_PUZZLE_VALUES {
      write!(stdout, "│    │")?;
      stdout
        .queue(cursor::MoveLeft(VAL_CHAR_WIDTH as u16 + 2))?
        .queue(cursor::MoveDown(1))?;
    }

    write!(stdout, "└{}┘", top_bottom_lines)?;

    Ok(())
  }
}

impl Printable for PuzzleIO {
  fn print(&self) -> io::Result<()> {
    self.print_with_expected_outputs(self)
  }
}

///
/// Load and run the Lua code to generate the puzzles
///
pub fn generate_test_cases(lua_file: &str, seed: u32, n: usize) -> Result<TestCaseSet, Box<dyn Error>> {
  // Try to load the Lua code file into memory
  let lua_code = fs::read_to_string(format!("levels/{}", lua_file))?;

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
        let (p0_inputs, p0_outputs, p1_inputs, p1_outputs): (Vec<i16>, Vec<i16>, Vec<i16>, Vec<i16>) =
          generate_test_case.call(())?;

        let p0 = ProcessorIO::new(p0_inputs, p0_outputs).map_err(|e| LuaError::RuntimeError(e))?;
        let p1 = ProcessorIO::new(p1_inputs, p1_outputs).map_err(|e| LuaError::RuntimeError(e))?;
        Ok(Puzzle::new(p0, p1))
      })
      .collect::<Result<_, _>>()?;

    Ok(test_cases)
  })?;

  Ok(test_cases)
}
