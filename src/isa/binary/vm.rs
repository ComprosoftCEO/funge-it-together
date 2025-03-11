use crossterm::style::{self, Color, Stylize};
use crossterm::{cursor, QueueableCommand};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::io::{self, Write};

use super::puzzle::{Puzzle, PuzzleIO};
use super::solution::{Memory, Solution};
use crate::grid::Grid;
use crate::printable::Printable;

pub const VAL_CHAR_WIDTH: usize = 4; // 3 numbers and negative sign
const MAX_STACK_ENTRIES: usize = 16;

#[derive(Debug, Clone)]
pub struct VirtualMachine {
  grid: Grid<Command>,
  memory: Memory,

  cycle: u32,
  row: i16,
  col: i16,
  direction: Direction,
  skip_next_instruction: bool,
  last_was_number: bool,
  is_carry: bool,
  is_overflow: bool,
  stack: Stack,

  inputs: PuzzleIO,
  outputs: PuzzleIO,
  test_case: usize,
  expected_outputs: PuzzleIO,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
  Up,
  Down,
  Left,
  Right,
}

impl Direction {
  pub fn get_arrow(&self) -> char {
    match self {
      Self::Up => '▲',
      Self::Down => '▼',
      Self::Left => '◄',
      Self::Right => '►',
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
  #[serde(rename = " ", alias = "empty")]
  Empty,
  #[serde(rename = "↑", alias = "up")]
  Up,
  #[serde(rename = "↓", alias = "down")]
  Down,
  #[serde(rename = "←", alias = "left")]
  Left,
  #[serde(rename = "→", alias = "right")]
  Right,
  #[serde(rename = "/", alias = "forwardSlash")]
  ForwardSlash,
  #[serde(rename = "\\", alias = "backSlash")]
  BackSlash,
  #[serde(rename = "♂", alias = "skip")]
  Skip,

  #[serde(rename = "0", alias = "zero")]
  Zero,
  #[serde(rename = "1", alias = "one")]
  One,
  #[serde(rename = "2", alias = "two")]
  Two,
  #[serde(rename = "3", alias = "three")]
  Three,
  #[serde(rename = "4", alias = "four")]
  Four,
  #[serde(rename = "5", alias = "five")]
  Five,
  #[serde(rename = "6", alias = "six")]
  Six,
  #[serde(rename = "7", alias = "seven")]
  Seven,
  #[serde(rename = "8", alias = "eight")]
  Eight,
  #[serde(rename = "9", alias = "nine")]
  Nine,
  #[serde(rename = "A", alias = "a")]
  A,
  #[serde(rename = "B", alias = "b")]
  B,
  #[serde(rename = "C", alias = "c")]
  C,
  #[serde(rename = "D", alias = "d")]
  D,
  #[serde(rename = "E", alias = "e")]
  E,
  #[serde(rename = "F", alias = "f")]
  F,

  #[serde(rename = "☼", alias = "pop")]
  Pop,
  #[serde(rename = "©", alias = "copy")]
  Copy,
  #[serde(rename = "∫", alias = "swapTop2")]
  SwapTop2,
  #[serde(rename = "u", alias = "rotateDown")]
  RotateDown,
  #[serde(rename = "∩", alias = "rotateUp")]
  RotateUp,

  #[serde(rename = "+", alias = "add")]
  Add,
  #[serde(rename = "-", alias = "subtract")]
  Subtract,

  #[serde(rename = "&", alias = "bitwiseAnd")]
  BitwiseAnd,
  #[serde(rename = "¦", alias = "bitwiseOr")]
  BitwiseOr,
  #[serde(rename = "Λ", alias = "bitwiseXor")]
  BitwiseXor,
  #[serde(rename = "¬", alias = "bitwiseNot")]
  BitwiseNot,
  #[serde(rename = "»", alias = "logicalShiftRight")]
  LogicalShiftRight,

  #[serde(rename = "<", alias = "ifLess")]
  IfLess,
  #[serde(rename = "=", alias = "ifEqual")]
  IfEqual,
  #[serde(rename = ">", alias = "ifGreater")]
  IfGreater,
  #[serde(rename = "‰", alias = "ifCarry")]
  IfCarry,
  #[serde(rename = "±", alias = "ifOverflow")]
  IfOverflow,

  #[serde(rename = "Ї", alias = "in")]
  In,
  #[serde(rename = "?", alias = "hasInput")]
  HasInput,
  #[serde(rename = "Θ", alias = "out")]
  Out,

  #[serde(rename = "@", alias = "load")]
  Load,
  #[serde(rename = "$", alias = "store")]
  Store,
}

impl Command {
  pub fn get_char(&self) -> char {
    match self {
      Self::Empty => ' ',
      Self::Up => '↑',
      Self::Down => '↓',
      Self::Left => '←',
      Self::Right => '→',
      Self::ForwardSlash => '/',
      Self::BackSlash => '\\',
      Self::Skip => '♂', // Jump over
      Self::Zero => '0',
      Self::One => '1',
      Self::Two => '2',
      Self::Three => '3',
      Self::Four => '4',
      Self::Five => '5',
      Self::Six => '6',
      Self::Seven => '7',
      Self::Eight => '8',
      Self::Nine => '9',
      Self::A => 'A',
      Self::B => 'B',
      Self::C => 'C',
      Self::D => 'D',
      Self::E => 'E',
      Self::F => 'F',
      Self::Pop => '☼',  // Tiny explosion
      Self::Copy => '©', // "Copy"right
      Self::SwapTop2 => '∫',
      Self::RotateDown => 'u',
      Self::RotateUp => '∩',
      Self::Add => '+',
      Self::Subtract => '-',
      Self::BitwiseAnd => '&',
      Self::BitwiseOr => '¦',
      Self::BitwiseXor => 'Λ',
      Self::BitwiseNot => '¬',
      Self::LogicalShiftRight => '»',
      Self::IfLess => '<',
      Self::IfEqual => '=',
      Self::IfGreater => '>',
      Self::IfCarry => '‰',
      Self::IfOverflow => '±',
      Self::In => 'Ї',
      Self::HasInput => '?',
      Self::Out => 'Θ',
      Self::Load => '@',
      Self::Store => '$',
    }
  }
}

impl Default for Command {
  fn default() -> Self {
    Self::Empty
  }
}

pub enum VMError {
  StackOverflow,
  StackUnderflow,
  NoInputs,
}

#[allow(unused)]
impl VirtualMachine {
  pub fn new(solution: Solution, test_case: usize, puzzle: &Puzzle) -> Self {
    let row = solution.start_row() as i16;
    let col = solution.start_col() as i16;
    let (grid, memory) = solution.into_parts();

    // TODO: puzzle can write to memory too

    Self {
      grid,
      memory,
      cycle: 0,
      row,
      col,
      direction: Direction::Right, // Always starts facing right
      skip_next_instruction: false,
      last_was_number: false,
      is_carry: false,
      is_overflow: false,
      stack: Stack::new(),
      inputs: puzzle.get_inputs().clone(),
      outputs: PuzzleIO::new(),
      test_case,
      expected_outputs: puzzle.get_outputs().clone(),
    }
  }

  pub fn rows(&self) -> usize {
    self.grid.rows()
  }

  pub fn cols(&self) -> usize {
    self.grid.cols()
  }

  pub fn count_symbols(&self) -> usize {
    self.grid.count_symbols()
  }

  pub fn get_cycle(&self) -> u32 {
    self.cycle
  }

  // Returns Ok(true) when the puzzle is solved
  pub fn step(&mut self) -> Result<bool, VMError> {
    if self.inputs.len() == 0 && self.outputs == self.expected_outputs {
      return Ok(true);
    }

    self.cycle = self.cycle.wrapping_add(1);

    let mut is_number = false;
    if !self.skip_next_instruction {
      match self.grid.get_value(self.row as usize, self.col as usize) {
        Command::Empty => {},
        Command::Up => {
          self.direction = Direction::Up;
        },
        Command::Down => self.direction = Direction::Down,
        Command::Left => self.direction = Direction::Left,
        Command::Right => self.direction = Direction::Right,
        Command::ForwardSlash => {
          self.direction = match self.direction {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Up,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Down,
          };
        },
        Command::BackSlash => {
          self.direction = match self.direction {
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Up,
            Direction::Down => Direction::Right,
            Direction::Right => Direction::Down,
          };
        },
        Command::Skip => {
          self.skip_next_instruction = true;
        },

        Command::Zero => {
          is_number = true;
          self.handle_number(0)?;
        },
        Command::One => {
          is_number = true;
          self.handle_number(1)?;
        },
        Command::Two => {
          is_number = true;
          self.handle_number(2)?;
        },
        Command::Three => {
          is_number = true;
          self.handle_number(3)?;
        },
        Command::Four => {
          is_number = true;
          self.handle_number(4)?;
        },
        Command::Five => {
          is_number = true;
          self.handle_number(5)?;
        },
        Command::Six => {
          is_number = true;
          self.handle_number(6)?;
        },
        Command::Seven => {
          is_number = true;
          self.handle_number(7)?;
        },
        Command::Eight => {
          is_number = true;
          self.handle_number(8)?;
        },
        Command::Nine => {
          is_number = true;
          self.handle_number(9)?;
        },
        Command::A => {
          is_number = true;
          self.handle_number(10)?;
        },
        Command::B => {
          is_number = true;
          self.handle_number(11)?;
        },
        Command::C => {
          is_number = true;
          self.handle_number(12)?;
        },
        Command::D => {
          is_number = true;
          self.handle_number(13)?;
        },
        Command::E => {
          is_number = true;
          self.handle_number(14)?;
        },
        Command::F => {
          is_number = true;
          self.handle_number(15)?;
        },

        Command::Pop => {
          self.pop()?;
        },
        Command::Copy => {
          let val = self.peek()?;
          self.push(val)?;
        },
        Command::SwapTop2 => {
          let v1 = self.pop()?;
          let v2 = self.pop()?;
          self.push(v1)?;
          self.push(v2)?;
        },
        Command::RotateDown => {
          self.stack.rotate_down();
        },
        Command::RotateUp => {
          self.stack.rotate_up();
        },

        Command::Add => {
          let v2 = self.pop()?;
          let v1 = self.pop()?;
          let result = self.handle_add(v1, v2);
          self.push(result)?;
        },
        Command::Subtract => {
          let v2 = self.pop()?;
          let v1 = self.pop()?;
          let result = self.handle_sub(v1, v2);
          self.push(result)?;
        },

        Command::BitwiseAnd => {
          let v2 = self.pop()?;
          let v1 = self.pop()?;
          self.push(v1 & v2)?;
        },
        Command::BitwiseOr => {
          let v2 = self.pop()?;
          let v1 = self.pop()?;
          self.push(v1 | v2)?;
        },
        Command::BitwiseXor => {
          let v2 = self.pop()?;
          let v1 = self.pop()?;
          self.push(v1 ^ v2)?;
        },
        Command::BitwiseNot => {
          let v1 = self.pop()?;
          self.push(!v1)?;
        },
        Command::LogicalShiftRight => {
          let v1 = self.pop()?;
          self.push(v1 >> 1)?;
        },

        Command::IfLess => {
          let val = self.peek()?;
          self.skip_next_instruction = !((val as i8) < 0);
        },
        Command::IfEqual => {
          let val = self.peek()?;
          self.skip_next_instruction = !(val == 0);
        },
        Command::IfGreater => {
          let val = self.peek()?;
          self.skip_next_instruction = !((val as i8) > 0);
        },
        Command::IfCarry => {
          self.skip_next_instruction = self.is_carry;
        },
        Command::IfOverflow => {
          self.skip_next_instruction = self.is_overflow;
        },

        Command::In => {
          let val = self.inputs.read().ok_or(VMError::NoInputs)?;
          self.push(val)?;
        },
        Command::HasInput => {
          self.skip_next_instruction = !self.inputs.can_read();
        },
        Command::Out => {
          let val = self.pop()?;
          self.outputs.write(val);
        },

        Command::Load => {
          let addr = self.pop()?;
          self.push(self.memory[addr as usize])?;
        },
        Command::Store => {
          let addr = self.pop()?;
          let val = self.pop()?;
          self.memory[addr as usize] = val;
        },
      }
    } else {
      self.skip_next_instruction = false;
    }

    // Now perform movement
    match self.direction {
      Direction::Up => {
        self.row = (self.row - 1).rem_euclid(self.grid.rows() as i16);
      },
      Direction::Down => {
        self.row = (self.row + 1).rem_euclid(self.grid.rows() as i16);
      },
      Direction::Left => {
        self.col = (self.col - 1).rem_euclid(self.grid.cols() as i16);
      },
      Direction::Right => {
        self.col = (self.col + 1).rem_euclid(self.grid.cols() as i16);
      },
    }

    self.last_was_number = is_number;

    Ok(false)
  }

  fn push(&mut self, val: u8) -> Result<(), VMError> {
    self.stack.push(val).then_some(()).ok_or(VMError::StackOverflow)
  }

  fn pop(&mut self) -> Result<u8, VMError> {
    self.stack.pop().ok_or(VMError::StackUnderflow)
  }

  fn peek(&self) -> Result<u8, VMError> {
    self.stack.peek().ok_or(VMError::StackUnderflow)
  }

  fn handle_number(&mut self, number: u8) -> Result<(), VMError> {
    if self.last_was_number {
      let val = self.pop()?;
      self.last_was_number = false; // To have consecutive numbers
      self.push(val * 16 + number)
    } else {
      self.push(number)
    }
  }

  // Updates the carry and overflow flags accordingly
  fn handle_add(&mut self, left: u8, right: u8) -> u8 {
    let (result, is_carry) = left.overflowing_add(right);
    let (_, is_overflow) = (left as i8).overflowing_add(right as i8);

    self.is_carry = is_carry;
    self.is_overflow = is_overflow;

    result
  }

  // Updates the carry and overflow flags accordingly
  fn handle_sub(&mut self, left: u8, right: u8) -> u8 {
    let (result, is_carry) = left.overflowing_sub(right);
    let (_, is_overflow) = (left as i8).overflowing_sub(right as i8);

    self.is_carry = is_carry;
    self.is_overflow = is_overflow;

    result
  }

  pub fn print_error_symbol_at(&self, row: u16, col: u16) -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.queue(cursor::MoveTo(col + self.col as u16 + 1, row + self.row as u16 + 1))?;
    write!(
      stdout,
      "{}",
      self
        .grid
        .get_value(self.row as usize, self.col as usize)
        .get_char()
        .red()
        .reverse()
    )?;

    Ok(())
  }

  pub fn row(&self) -> usize {
    self.row as usize
  }

  pub fn col(&self) -> usize {
    self.col as usize
  }

  pub fn toggle_breakpoint(&mut self, row: usize, col: usize) {
    self.grid.toggle_breakpoint(row, col)
  }

  pub fn is_at_breakpoint(&self) -> bool {
    self.grid.has_breakpoint(self.row as usize, self.col as usize) && !self.skip_next_instruction
  }
}

// impl Printable for VirtualMachine {
//   fn print(&self) -> io::Result<()> {
//     let mut stdout = io::stdout();
//     stdout.queue(cursor::SavePosition)?;
//     self.grid.print()?;

//     stdout
//       .queue(cursor::RestorePosition)?
//       .queue(cursor::MoveDown(self.grid.rows() as u16 + 2 + 1))?;
//     write!(stdout, "{} {}", "Cycle:".dark_cyan(), self.cycle)?;

//     stdout
//       .queue(cursor::RestorePosition)?
//       .queue(cursor::MoveDown(self.row as u16 + 1))?
//       .queue(cursor::MoveRight(self.col as u16 + 1))?;
//     if self.skip_next_instruction {
//       write!(stdout, "{}", self.direction.get_arrow().red())?;
//     } else if self.is_at_breakpoint() {
//       stdout.queue(style::SetBackgroundColor(Color::DarkCyan))?;
//       write!(stdout, "{}", self.direction.get_arrow().black())?;
//       stdout.queue(style::ResetColor)?;
//     } else {
//       write!(stdout, "{}", self.direction.get_arrow().green())?;
//     }

//     stdout
//       .queue(cursor::RestorePosition)?
//       .queue(cursor::MoveRight(self.grid.cols() as u16 + 2 + 8))?
//       .queue(cursor::SavePosition)?;

//     write!(stdout, "{}", format!("Test Case {}", self.test_case).dark_yellow())?;
//     stdout
//       .queue(cursor::RestorePosition)?
//       .queue(cursor::MoveDown(2))?
//       .queue(cursor::SavePosition)?;
//     write!(stdout, "Stack    Input  Output Expected")?;

//     stdout
//       .queue(cursor::RestorePosition)?
//       .queue(cursor::MoveDown(1))?
//       .queue(cursor::SavePosition)?;
//     self.stack.print()?;
//     stdout.queue(cursor::RestorePosition)?.queue(cursor::MoveRight(9))?;
//     self.inputs.print()?;
//     stdout.queue(cursor::RestorePosition)?.queue(cursor::MoveRight(16))?;
//     self.outputs.print_with_expected_outputs(&self.expected_outputs)?;
//     stdout.queue(cursor::RestorePosition)?.queue(cursor::MoveRight(23))?;
//     self.expected_outputs.print()?;

//     Ok(())
//   }
// }

impl Printable for Command {
  fn print(&self) -> io::Result<()> {
    let mut stdout = io::stdout();
    write!(stdout, "{}", self.get_char())
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stack {
  values: VecDeque<u8>,
}

#[allow(unused)]
impl Stack {
  pub fn new() -> Self {
    Self {
      values: VecDeque::new(),
    }
  }

  pub fn len(&self) -> usize {
    self.values.len()
  }

  pub fn is_empty(&self) -> bool {
    self.values.len() == 0
  }

  // Returns false if the stack overflows
  pub fn push(&mut self, val: u8) -> bool {
    if self.values.len() < MAX_STACK_ENTRIES {
      self.values.push_back(val);
      true
    } else {
      false
    }
  }

  // Returns None if the stack underflows
  pub fn pop(&mut self) -> Option<u8> {
    self.values.pop_back()
  }

  pub fn peek(&self) -> Option<u8> {
    self.values.back().cloned()
  }

  pub fn rotate_down(&mut self) {
    match self.pop() {
      None => (),
      Some(v) => {
        self.values.push_front(v);
      },
    }
  }

  pub fn rotate_up(&mut self) {
    match self.values.pop_front() {
      None => (),
      Some(v) => self.values.push_back(v),
    }
  }
}

impl Printable for Stack {
  fn print(&self) -> io::Result<()> {
    let mut stdout = io::stdout();
    // ┌─┐
    // │ │
    // └─┘
    let top_bottom_lines: String = "─".repeat(VAL_CHAR_WIDTH);
    write!(stdout, "┌{}┐", top_bottom_lines)?;
    stdout
      .queue(cursor::MoveLeft(VAL_CHAR_WIDTH as u16 + 2))?
      .queue(cursor::MoveDown(1))?;

    for _ in self.values.len()..MAX_STACK_ENTRIES {
      write!(stdout, "│    │")?;
      stdout
        .queue(cursor::MoveLeft(VAL_CHAR_WIDTH as u16 + 2))?
        .queue(cursor::MoveDown(1))?;
    }

    for value in self.values.iter() {
      write!(stdout, "│{:-4}│", value)?;
      stdout
        .queue(cursor::MoveLeft(VAL_CHAR_WIDTH as u16 + 2))?
        .queue(cursor::MoveDown(1))?;
    }

    write!(stdout, "└{}┘", top_bottom_lines)?;

    Ok(())
  }
}

impl VMError {
  pub fn get_msg(&self) -> &'static str {
    match self {
      Self::StackOverflow => "Stack overflow",
      Self::StackUnderflow => "Stack underflow",
      Self::NoInputs => "No inputs left",
    }
  }
}

impl Printable for VMError {
  fn print(&self) -> io::Result<()> {
    write!(io::stdout(), "{}", self.get_msg().red())
  }
}
