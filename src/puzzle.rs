use crossterm::{cursor, QueueableCommand};
use rand::Rng;
use std::collections::VecDeque;
use std::{
  io::{self, Write},
  iter,
};

use crate::vm::{VAL_MAX, VAL_MIN};
use crate::{printable::Printable, vm::VAL_CHAR_WIDTH};

pub const MAX_PUZZLE_VALUES: usize = 15;

pub type TestCaseSet = Vec<Puzzle>;

#[derive(Debug, Clone)]
pub struct Puzzle {
  inputs: PuzzleIO,
  outputs: PuzzleIO,
}

impl Puzzle {
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

    for val in inputs.iter().cloned() {
      if val < VAL_MIN || val > VAL_MAX {
        return Err(format!("Input {val} outside range [-999,999]"));
      }
    }
    for val in outputs.iter().cloned() {
      if val < VAL_MIN || val > VAL_MAX {
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
    self.inputs.print()?;

    stdout.queue(cursor::RestorePosition)?.queue(cursor::MoveRight(7))?;
    self.outputs.print()?;

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
    Self(
      (0..rng.gen_range(0..=10))
        .into_iter()
        .map(|_| rng.gen_range(-999..=999))
        .collect(),
    )
  }

  pub fn len(&self) -> usize {
    self.0.len()
  }

  pub fn can_read(&self) -> bool {
    self.0.len() > 0
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
}

impl Printable for PuzzleIO {
  fn print(&self) -> io::Result<()> {
    let mut stdout = io::stdout();
    // ┌─┐
    // │ │
    // └─┘
    let top_bottom_lines: String = iter::repeat("─").take(VAL_CHAR_WIDTH).collect();
    write!(stdout, "┌{}┐", top_bottom_lines)?;
    stdout
      .queue(cursor::MoveLeft(VAL_CHAR_WIDTH as u16 + 2))?
      .queue(cursor::MoveDown(1))?;

    for value in self.0.iter() {
      write!(stdout, "│{:-4}│", value)?;
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
