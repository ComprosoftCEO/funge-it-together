use crossterm::style::{self, Color, Stylize};
use crossterm::{cursor, QueueableCommand};
use serde::{Deserialize, Serialize};
use std::cell::{RefCell, RefMut};
use std::collections::VecDeque;
use std::io::{self, Write};
use std::rc::{Rc, Weak};

// use crate::global_state::Solution;
use super::puzzle::{ProcessorIO, Puzzle, PuzzleIO};
use super::solution::{Program, Solution};
use crate::grid::Grid;
use crate::printable::Printable;

pub const VAL_MIN: i16 = -999;
pub const VAL_MAX: i16 = 999;
pub const VAL_CHAR_WIDTH: usize = 4; // 3 numbers and negative sign
const MAX_STACK_ENTRIES: usize = 8;

#[derive(Debug, Clone)]
pub struct VirtualMachine {
  processors: [Rc<RefCell<Processor>>; 2],
  cycle: u32,
  test_case: usize,
}

#[derive(Debug, Clone)]
pub struct Processor {
  grid: Grid<Command>,

  row: i16,
  col: i16,
  direction: Direction,
  skip_next_instruction: bool,
  last_was_number: bool,
  stack: Stack,

  inputs: PuzzleIO,
  outputs: PuzzleIO,
  expected_outputs: PuzzleIO,

  other_processor: Option<Weak<RefCell<Processor>>>,
  sending_status: SendStatus,
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
  #[serde(rename = "х", alias = "multiply")]
  Multiply,
  #[serde(rename = "<", alias = "ifLess")]
  IfLess,
  #[serde(rename = "=", alias = "ifEqual")]
  IfEqual,
  #[serde(rename = ">", alias = "ifGreater")]
  IfGreater,
  #[serde(rename = "»", alias = "skip")]
  Skip,
  #[serde(rename = "Ї", alias = "in")]
  In,
  #[serde(rename = "?", alias = "hasInput")]
  HasInput,
  #[serde(rename = "Θ", alias = "out")]
  Out,
  #[serde(rename = "τ", alias = "transmit")]
  Transmit,
  #[serde(rename = "я", alias = "receive")]
  Receive,
  #[serde(rename = "Ť", alias = "tryTransmit")]
  TryTransmit,
  #[serde(rename = "Ř", alias = "tryReceive")]
  TryReceive,
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
      Self::Pop => '☼',  // Tiny explosion
      Self::Copy => '©', // "Copy"right
      Self::SwapTop2 => '∫',
      Self::RotateDown => 'u',
      Self::RotateUp => '∩',
      Self::Add => '+',
      Self::Subtract => '-',
      Self::Multiply => 'х',
      Self::IfLess => '<',
      Self::IfEqual => '=',
      Self::IfGreater => '>',
      Self::Skip => '»',
      Self::In => 'Ї',
      Self::HasInput => '?',
      Self::Out => 'Θ',
      Self::Transmit => 'τ',
      Self::Receive => 'я',
      Self::TryTransmit => 'Ť',
      Self::TryReceive => 'Ř',
    }
  }
}

impl Default for Command {
  fn default() -> Self {
    Self::Empty
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SendStatus {
  None,
  Transmitting,
  TryTransmitting,
  Receiving,
  TryReceiving,
  Completed,
}

impl SendStatus {
  pub fn is_blocking(self) -> bool {
    matches!(self, SendStatus::Transmitting | SendStatus::Receiving)
  }
}

pub enum VMError {
  NumericOverflow,
  StackOverflow,
  StackUnderflow,
  NoInputs,
  TooManyOutputs,
  Deadlock,
}

#[allow(unused)]
impl VirtualMachine {
  pub fn new(solution: Solution, test_case: usize, io: Puzzle) -> Self {
    let (p0, p1) = solution.into_programs();
    let (p0_io, p1_io) = io.into_processor_ios();

    let mut processor_0 = Rc::new(RefCell::new(Processor::new(p0, p0_io)));
    let mut processor_1 = Rc::new(RefCell::new(Processor::new(p1, p1_io)));

    (*processor_0).borrow_mut().other_processor = Some(Rc::downgrade(&processor_1));
    (*processor_1).borrow_mut().other_processor = Some(Rc::downgrade(&processor_0));

    Self {
      processors: [processor_0, processor_1],
      cycle: 0,
      test_case,
    }
  }

  pub fn get_cycle(&self) -> u32 {
    self.cycle
  }

  pub fn count_symbols(&self) -> usize {
    self
      .processors
      .iter()
      .map(|processor| (**processor).borrow().count_symbols())
      .sum()
  }

  pub fn is_at_breakpoint(&self) -> bool {
    self
      .processors
      .iter()
      .any(|processor| (**processor).borrow().is_at_breakpoint())
  }

  pub fn processor_0(&mut self) -> RefMut<Processor> {
    (*self.processors[0]).borrow_mut()
  }

  pub fn processor_1(&mut self) -> RefMut<Processor> {
    (*self.processors[1]).borrow_mut()
  }

  // Returns Ok(true) when the puzzle is solved
  pub fn step(&mut self) -> Result<bool, (VMError, usize)> {
    // Have we solved the puzzle?
    if self.processors.iter().all(|processor| {
      let processor = (**processor).borrow();
      processor.inputs.len() == 0 && processor.outputs == processor.expected_outputs
    }) {
      return Ok(true);
    }

    for processor in self.processors.iter_mut() {
      (**processor).borrow_mut().compute_send_status();
    }

    let mut result = None;

    for (index, processor) in self.processors.iter_mut().enumerate() {
      // Nope, so step the next processor
      if let Err(e) = (**processor).borrow_mut().step() {
        if result.is_none() {
          result = Some((e, index)) // Only save the first error
        }
      }
    }

    self.cycle = self.cycle.wrapping_add(1);

    if let Some(e) = result {
      Err(e)
    } else {
      Ok(false)
    }
  }

  fn print_processor_program(&self, processor: &Processor, line: u16) -> io::Result<()> {
    let mut stdout = io::stdout();

    if line > 0 {
      stdout.queue(cursor::MoveDown(line))?;
    }

    processor.grid.print()?;

    stdout
      .queue(cursor::RestorePosition)?
      .queue(cursor::MoveDown(processor.row as u16 + 2 + line))?
      .queue(cursor::MoveRight(processor.col as u16 + 1))?;

    if processor.skip_next_instruction {
      write!(stdout, "{}", processor.direction.get_arrow().red())?;
    } else if self.is_at_breakpoint() && processor.is_at_breakpoint() {
      stdout.queue(style::SetBackgroundColor(Color::DarkCyan))?;
      write!(stdout, "{}", processor.direction.get_arrow().black())?;
      stdout.queue(style::ResetColor)?;
    } else {
      write!(stdout, "{}", processor.direction.get_arrow().green())?;
    }

    Ok(())
  }

  pub fn height(&self) -> u16 {
    self
      .processors
      .iter()
      .map(|processor| (**processor).borrow().rows() + 1)
      .sum::<usize>() as u16
      + 1
  }
}

#[allow(unused)]
impl Processor {
  pub fn new(program: Program, io: ProcessorIO) -> Self {
    let row = program.start_row() as i16;
    let col = program.start_col() as i16;

    Self {
      grid: program.into_grid(),
      row,
      col,
      direction: Direction::Right, // Always starts facing right
      skip_next_instruction: false,
      last_was_number: false,
      stack: Stack::new(),
      inputs: io.get_inputs().clone(),
      outputs: PuzzleIO::new(),
      expected_outputs: io.get_outputs().clone(),
      other_processor: None,
      sending_status: SendStatus::None,
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

  pub fn compute_send_status(&mut self) {
    self.sending_status = match self.grid.get_value(self.row as usize, self.col as usize) {
      Command::Transmit => SendStatus::Transmitting,
      Command::TryTransmit => SendStatus::TryTransmitting,
      Command::Receive => SendStatus::Receiving,
      Command::TryReceive => SendStatus::TryReceiving,
      _ => SendStatus::None,
    };
  }

  // Returns Ok(true) when the puzzle is solved
  pub fn step(&mut self) -> Result<(), VMError> {
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
          self.push(v1 + v2)?;
        },
        Command::Subtract => {
          let v2 = self.pop()?;
          let v1 = self.pop()?;
          self.push(v1 - v2)?;
        },
        Command::Multiply => {
          let v2 = self.pop()?;
          let v1 = self.pop()?;
          self.push(v1 * v2)?;
        },
        Command::IfLess => {
          let val = self.peek()?;
          self.skip_next_instruction = !(val < 0);
        },
        Command::IfEqual => {
          let val = self.peek()?;
          self.skip_next_instruction = !(val == 0);
        },
        Command::IfGreater => {
          let val = self.peek()?;
          self.skip_next_instruction = !(val > 0);
        },
        Command::Skip => {
          self.skip_next_instruction = true;
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
          if !self.outputs.write(val) {
            return Err(VMError::TooManyOutputs);
          }
        },
        Command::Transmit => {
          self.handle_transmit()?;
        },
        Command::Receive => {
          self.handle_receive()?;
        },
        Command::TryTransmit => {
          self.handle_try_transmit()?;
        },
        Command::TryReceive => {
          self.handle_try_receive()?;
        },
      }
    } else {
      self.skip_next_instruction = false;
    }

    // Now perform movement (if not waiting for the send status)
    if !self.sending_status.is_blocking() {
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
    }

    self.last_was_number = is_number;

    Ok(())
  }

  fn push(&mut self, val: i16) -> Result<(), VMError> {
    if val != val.clamp(VAL_MIN, VAL_MAX) {
      return Err(VMError::NumericOverflow);
    }

    self.stack.push(val).then_some(()).ok_or(VMError::StackOverflow)
  }

  fn pop(&mut self) -> Result<i16, VMError> {
    self.stack.pop().ok_or(VMError::StackUnderflow)
  }

  fn peek(&self) -> Result<i16, VMError> {
    self.stack.peek().ok_or(VMError::StackUnderflow)
  }

  fn handle_number(&mut self, number: i16) -> Result<(), VMError> {
    if self.last_was_number {
      let val = self.pop()?;
      self.push(val * 10 + number)
    } else {
      self.push(number)
    }
  }

  fn handle_transmit(&mut self) -> Result<(), VMError> {
    let rc_other_processor = Weak::upgrade(self.other_processor.as_ref().unwrap()).unwrap();
    let mut other_processor = (*rc_other_processor).borrow_mut();

    match other_processor.sending_status {
      SendStatus::None | SendStatus::TryTransmitting => { /* Block */ },
      SendStatus::Transmitting => return Err(VMError::Deadlock),
      SendStatus::Receiving | SendStatus::TryReceiving => {
        other_processor.push(self.pop()?)?;
        self.sending_status = SendStatus::Completed;
      },
      SendStatus::Completed => {
        self.sending_status = SendStatus::Completed;
      },
    }

    Ok(())
  }

  fn handle_receive(&mut self) -> Result<(), VMError> {
    let rc_other_processor = Weak::upgrade(self.other_processor.as_ref().unwrap()).unwrap();
    let mut other_processor = (*rc_other_processor).borrow_mut();

    match other_processor.sending_status {
      SendStatus::None | SendStatus::TryReceiving => { /* Block */ },
      SendStatus::Receiving => return Err(VMError::Deadlock),
      SendStatus::Transmitting | SendStatus::TryTransmitting => {
        self.push(other_processor.pop()?)?;
        self.sending_status = SendStatus::Completed;
      },
      SendStatus::Completed => {
        self.sending_status = SendStatus::Completed;
      },
    }

    Ok(())
  }

  fn handle_try_transmit(&mut self) -> Result<(), VMError> {
    let rc_other_processor = Weak::upgrade(self.other_processor.as_ref().unwrap()).unwrap();
    let mut other_processor = (*rc_other_processor).borrow_mut();

    match other_processor.sending_status {
      SendStatus::None | SendStatus::Transmitting | SendStatus::TryTransmitting => {
        // Transmission failed, so skip the next instruction
        self.skip_next_instruction = true;
      },
      SendStatus::Receiving | SendStatus::TryReceiving => {
        other_processor.push(self.pop()?)?;
      },
      SendStatus::Completed => {},
    }

    self.sending_status = SendStatus::Completed;
    Ok(())
  }

  fn handle_try_receive(&mut self) -> Result<(), VMError> {
    let rc_other_processor = Weak::upgrade(self.other_processor.as_ref().unwrap()).unwrap();
    let mut other_processor = (*rc_other_processor).borrow_mut();

    match other_processor.sending_status {
      SendStatus::None | SendStatus::Receiving | SendStatus::TryReceiving => {
        // Receiving failed, so skip the next instruction
        self.skip_next_instruction = true;
      },
      SendStatus::Transmitting | SendStatus::TryTransmitting => {
        self.push(other_processor.pop()?)?;
      },
      SendStatus::Completed => {},
    }

    self.sending_status = SendStatus::Completed;
    Ok(())
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

impl Printable for VirtualMachine {
  fn print(&self) -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.queue(cursor::SavePosition)?.queue(cursor::MoveDown(1))?;

    let p0 = (*self.processors[0]).borrow();
    let p1 = (*self.processors[1]).borrow();

    self.print_processor_program(&p0, 0)?;
    stdout.queue(cursor::RestorePosition)?.queue(cursor::MoveDown(1))?;
    self.print_processor_program(&p1, p0.rows() as u16 + 1)?;

    stdout
      .queue(cursor::RestorePosition)?
      .queue(cursor::MoveDown(p0.rows() as u16 + 2))?;
    write!(stdout, "├{}┤", "─".repeat(p1.cols()))?;

    stdout
      .queue(cursor::RestorePosition)?
      .queue(cursor::MoveDown(self.height() + 1))?;
    write!(stdout, "{} {}", "Cycle:".dark_cyan(), self.cycle)?;

    stdout.queue(cursor::RestorePosition)?;
    write!(stdout, "{}", format!("Test Case {}", self.test_case).dark_yellow())?;

    stdout
      .queue(cursor::RestorePosition)?
      .queue(cursor::MoveRight(p0.cols() as u16 + 11))?
      .queue(cursor::SavePosition)?;

    write!(stdout, "Stack    Input  Output Expected")?;

    stdout
      .queue(cursor::RestorePosition)?
      .queue(cursor::MoveDown(1))?
      .queue(cursor::SavePosition)?;
    p0.stack.print()?;
    stdout.queue(cursor::RestorePosition)?.queue(cursor::MoveRight(9))?;
    p0.inputs.print()?;
    stdout.queue(cursor::RestorePosition)?.queue(cursor::MoveRight(16))?;
    p0.outputs.print_with_expected_outputs(&p0.expected_outputs)?;
    stdout.queue(cursor::RestorePosition)?.queue(cursor::MoveRight(23))?;
    p0.expected_outputs.print()?;

    stdout
      .queue(cursor::RestorePosition)?
      .queue(cursor::MoveDown(MAX_STACK_ENTRIES as u16 + 1))?
      .queue(cursor::SavePosition)?;

    p1.stack.print()?;
    stdout.queue(cursor::RestorePosition)?.queue(cursor::MoveRight(9))?;
    p1.inputs.print()?;
    stdout.queue(cursor::RestorePosition)?.queue(cursor::MoveRight(16))?;
    p1.outputs.print_with_expected_outputs(&p1.expected_outputs)?;
    stdout.queue(cursor::RestorePosition)?.queue(cursor::MoveRight(23))?;
    p1.expected_outputs.print()?;

    stdout.queue(cursor::RestorePosition)?;
    write!(stdout, "├{0}┤   ├{0}┤ ├{0}┤ ├{0}┤", "─".repeat(VAL_CHAR_WIDTH))?;

    Ok(())
  }
}

impl Printable for Command {
  fn print(&self) -> io::Result<()> {
    let mut stdout = io::stdout();
    write!(stdout, "{}", self.get_char())
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stack {
  values: VecDeque<i16>,
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
  pub fn push(&mut self, val: i16) -> bool {
    if self.values.len() < MAX_STACK_ENTRIES {
      self.values.push_back(val.clamp(VAL_MIN, VAL_MAX));
      true
    } else {
      false
    }
  }

  // Returns None if the stack underflows
  pub fn pop(&mut self) -> Option<i16> {
    self.values.pop_back()
  }

  pub fn peek(&self) -> Option<i16> {
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
      Self::NumericOverflow => "Numeric overflow",
      Self::StackOverflow => "Stack overflow",
      Self::StackUnderflow => "Stack underflow",
      Self::NoInputs => "No inputs left",
      Self::TooManyOutputs => "Too many outputs",
      Self::Deadlock => "Deadlock",
    }
  }
}

impl Printable for VMError {
  fn print(&self) -> io::Result<()> {
    write!(io::stdout(), "{}", self.get_msg().red())
  }
}
