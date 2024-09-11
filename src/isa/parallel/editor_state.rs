use crossterm::{
  cursor,
  event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind},
  style::{self, Color, Stylize},
  ExecutableCommand, QueueableCommand,
};
use std::io::{self, Write};

use super::execute_state::{ExecuteState, Speed};
use super::puzzle::TestCaseSet;
use super::solution::Solution;
use super::vm::{Command, VirtualMachine};
use crate::printable::Printable;
use crate::state::{print_string, ShowHelpState, State};
use crate::{global_state::GlobalState, isa::SolutionManager};
use crate::{isa, level::LevelIndex};

const GRID_ROW: u16 = 3;
const GRID_COL: u16 = 0;

macro_rules! current_program {
  ($input:expr) => {
    if $input.is_program_1 {
      $input.solution.program_1()
    } else {
      $input.solution.program_0()
    }
  };
}

static INSTRUCTIONS: &str = r#"
│Esc    = Main Menu
│Tab    = Step
│Space  = Start/Stop
│[  ]   = Test Case
│, .    = Breakpoint
│
│asdw  = ←↓→↑ (Move)
│/ \   = / \ (Bounce)
│$     = » (Skip)
│0-9   = 0-9
│p     = ☼ (Pop)
│c     = © (Copy)
│~     = ∫ (Swap)
│^ v   = ∩ u (Rotate)
│+ - * = (Add/Sub/Mul)
│< = > = (Compare to 0)
│i     = Ї (Input)
│o     = Θ (Output)
│?     = (Has input?)
│t     = τ (Transmit)
│r     = я (Receive)
|T R   = Ť Ř (Try T/R?)
│b     = Set start"#;

pub struct EditorState {
  level_index: LevelIndex,
  solution_index: usize,

  solution: Solution,

  cursor_row: isize,
  cursor_col: isize,
  is_program_1: bool,

  test_cases: TestCaseSet,
  test_case_index: isize,
}

impl EditorState {
  pub fn new(
    level_index: LevelIndex,
    solution_index: usize,
    solution: Solution,
    test_cases: TestCaseSet,
    test_case_index: usize,
  ) -> Self {
    Self {
      level_index,
      solution_index,
      solution,
      cursor_row: 0,
      cursor_col: 0,
      is_program_1: false,
      test_cases,
      test_case_index: test_case_index as isize,
    }
  }

  fn set_cell(&mut self, command: Command) {
    current_program!(self).set_grid_value(self.cursor_row as usize, self.cursor_col as usize, command);
  }

  pub(crate) fn level_index(&self) -> LevelIndex {
    self.level_index
  }

  pub(crate) fn vms(&self) -> Vec<VirtualMachine> {
    (0..self.test_cases.len())
      .map(|i| {
        let index = (self.test_case_index as usize + i).rem_euclid(self.test_cases.len());
        VirtualMachine::new(self.solution.clone(), index + 1, self.test_cases[index].clone())
      })
      .collect()
  }

  pub(crate) fn toggle_processor_0_breakpoint(&mut self, row: usize, col: usize) {
    self.solution.program_0().toggle_breakpoint(row, col);
  }

  pub(crate) fn toggle_processor_1_breakpoint(&mut self, row: usize, col: usize) {
    self.solution.program_1().toggle_breakpoint(row, col);
  }
}

impl State for EditorState {
  fn render(&mut self, global_state: &mut GlobalState) -> io::Result<()> {
    let mut stdout = io::stdout();

    let level = global_state.level(self.level_index);
    write!(stdout, "     {} - {}", self.level_index, level.name().yellow())?;

    self.solution.print_at(GRID_ROW, GRID_COL)?;

    stdout.queue(cursor::MoveTo(GRID_COL, 2))?.queue(cursor::SavePosition)?;

    write!(
      stdout,
      "{}",
      format!("Test Case {}", self.test_case_index + 1).dark_yellow()
    )?;
    stdout.queue(cursor::MoveTo(
      GRID_COL + self.solution.program_0().cols() as u16 + 2 + 8,
      2,
    ))?;

    self.test_cases[self.test_case_index as usize].print()?;

    stdout
      .queue(cursor::MoveTo(55, 0))?
      .queue(style::SetForegroundColor(Color::DarkCyan))?;
    print_string(INSTRUCTIONS)?;

    stdout
      .queue(style::ResetColor)?
      .queue(cursor::EnableBlinking)?
      .execute(cursor::MoveTo(
        GRID_COL + 1 + self.cursor_col as u16,
        GRID_ROW
          + 1
          + self.cursor_row as u16
          + if self.is_program_1 {
            self.solution.program_1().cols() as u16 + 1
          } else {
            0
          },
      ))?;

    Ok(())
  }

  fn execute(mut self: Box<Self>, global_state: &mut GlobalState) -> io::Result<Option<Box<dyn State>>> {
    loop {
      // `read()` blocks until an `Event` is available
      let event = match event::read() {
        Ok(e) => e,
        Err(e) => {
          println!("Error: {}", e);
          return Ok(None);
        },
      };

      match event {
        Event::Resize(_, _) => {
          return Ok(Some(self));
        },

        Event::Mouse(mouse) => {
          // Mouse only causes events inside the grid
          let mouse_col = (mouse.column as isize) - 1 - GRID_COL as isize;
          if mouse_col < 0 || mouse_col >= current_program!(self).cols() as isize {
            continue;
          }

          let mut mouse_row = (mouse.row as isize) - 1 - GRID_ROW as isize;
          if mouse_row < 0
            || mouse_row == self.solution.program_0().rows() as isize
            || mouse_row >= (self.solution.program_0().rows() + self.solution.program_1().rows() + 1) as isize
          {
            continue;
          }

          // Special case: handle program 0 vs 1
          let is_program_1 = if mouse_row > self.solution.program_0().rows() as isize {
            mouse_row -= (self.solution.program_0().rows() + 1) as isize;
            true
          } else {
            false
          };

          match mouse.kind {
            // Left button just selects the space
            MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Drag(MouseButton::Left) => {
              self.is_program_1 = is_program_1;
              self.cursor_row = mouse_row;
              self.cursor_col = mouse_col;
              return Ok(Some(self));
            },

            // Right button clears
            MouseEventKind::Down(MouseButton::Right) | MouseEventKind::Drag(MouseButton::Right) => {
              self.is_program_1 = is_program_1;
              self.cursor_row = mouse_row;
              self.cursor_col = mouse_col;
              self.set_cell(Command::Empty);
              return Ok(Some(self));
            },

            _ => {},
          }
        },

        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
          KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Ok(None);
          },

          KeyCode::Esc => {
            return Ok(Some(Box::new(ShowHelpState::<isa::Parallel>::new(
              self.level_index,
              self.solution_index,
              self.test_cases,
            ))))
          },

          // Start execution
          KeyCode::Tab => return Ok(Some(Box::new(ExecuteState::new(*self, Speed::None)))),
          KeyCode::Char(' ') => return Ok(Some(Box::new(ExecuteState::new(*self, Speed::Slow)))),

          // Movement
          KeyCode::Up | KeyCode::Char('k') => {
            if self.cursor_row == 0 {
              self.is_program_1 = !self.is_program_1;
            }
            self.cursor_row = (self.cursor_row - 1).rem_euclid(current_program!(self).rows() as isize); // TODO
            return Ok(Some(self));
          },
          KeyCode::Down | KeyCode::Char('j') => {
            if self.cursor_row + 1 == current_program!(self).rows() as isize {
              self.is_program_1 = !self.is_program_1;
            }
            self.cursor_row = (self.cursor_row + 1).rem_euclid(current_program!(self).rows() as isize); // TODO
            return Ok(Some(self));
          },
          KeyCode::Left | KeyCode::Char('h') => {
            self.cursor_col = (self.cursor_col - 1).rem_euclid(current_program!(self).cols() as isize);
            return Ok(Some(self));
          },
          KeyCode::Right | KeyCode::Char('l') => {
            self.cursor_col = (self.cursor_col + 1).rem_euclid(current_program!(self).cols() as isize);
            return Ok(Some(self));
          },

          // Select test case
          KeyCode::Char(']') => {
            self.test_case_index = (self.test_case_index + 1).rem_euclid(self.test_cases.len() as isize);
            return Ok(Some(self));
          },
          KeyCode::Char('[') => {
            self.test_case_index = (self.test_case_index - 1).rem_euclid(self.test_cases.len() as isize);
            return Ok(Some(self));
          },

          // Breakpoint
          KeyCode::Char(',') | KeyCode::Char('.') => {
            current_program!(self).toggle_breakpoint(self.cursor_row as usize, self.cursor_col as usize);
            break;
          },

          // Deletion
          KeyCode::Backspace | KeyCode::Delete | KeyCode::Char('x') => {
            self.set_cell(Command::Empty);
            break;
          },

          // Arrow commands
          KeyCode::Char('w') => {
            self.set_cell(Command::Up);
            break;
          },
          KeyCode::Char('s') => {
            self.set_cell(Command::Down);
            break;
          },
          KeyCode::Char('a') => {
            self.set_cell(Command::Left);
            break;
          },
          KeyCode::Char('d') => {
            self.set_cell(Command::Right);
            break;
          },
          KeyCode::Char('/') => {
            self.set_cell(Command::ForwardSlash);
            break;
          },
          KeyCode::Char('\\') => {
            self.set_cell(Command::BackSlash);
            break;
          },
          KeyCode::Char('$') => {
            self.set_cell(Command::Skip);
            break;
          },

          // Numbers
          KeyCode::Char('0') => {
            self.set_cell(Command::Zero);
            break;
          },
          KeyCode::Char('1') => {
            self.set_cell(Command::One);
            break;
          },
          KeyCode::Char('2') => {
            self.set_cell(Command::Two);
            break;
          },
          KeyCode::Char('3') => {
            self.set_cell(Command::Three);
            break;
          },
          KeyCode::Char('4') => {
            self.set_cell(Command::Four);
            break;
          },
          KeyCode::Char('5') => {
            self.set_cell(Command::Five);
            break;
          },
          KeyCode::Char('6') => {
            self.set_cell(Command::Six);
            break;
          },
          KeyCode::Char('7') => {
            self.set_cell(Command::Seven);
            break;
          },
          KeyCode::Char('8') => {
            self.set_cell(Command::Eight);
            break;
          },
          KeyCode::Char('9') => {
            self.set_cell(Command::Nine);
            break;
          },

          // Stack operations
          KeyCode::Char('p') => {
            self.set_cell(Command::Pop);
            break;
          },
          KeyCode::Char('c') => {
            self.set_cell(Command::Copy);
            break;
          },
          KeyCode::Char('~') => {
            self.set_cell(Command::SwapTop2);
            break;
          },
          KeyCode::Char('v') => {
            self.set_cell(Command::RotateDown);
            break;
          },
          KeyCode::Char('^') => {
            self.set_cell(Command::RotateUp);
            break;
          },

          // Math
          KeyCode::Char('+') => {
            self.set_cell(Command::Add);
            break;
          },
          KeyCode::Char('-') => {
            self.set_cell(Command::Subtract);
            break;
          },
          KeyCode::Char('*') if self.is_program_1 => {
            // Only processor 1 can multiply
            self.set_cell(Command::Multiply);
            break;
          },

          // Comparisons
          KeyCode::Char('<') => {
            self.set_cell(Command::IfLess);
            break;
          },
          KeyCode::Char('=') => {
            self.set_cell(Command::IfEqual);
            break;
          },
          KeyCode::Char('>') => {
            self.set_cell(Command::IfGreater);
            break;
          },

          // Input and output
          KeyCode::Char('i') => {
            self.set_cell(Command::In);
            break;
          },
          KeyCode::Char('?') => {
            self.set_cell(Command::HasInput);
            break;
          },
          KeyCode::Char('o') => {
            self.set_cell(Command::Out);
            break;
          },

          // Transmit / Receive
          KeyCode::Char('t') => {
            self.set_cell(Command::Transmit);
            break;
          },
          KeyCode::Char('r') => {
            self.set_cell(Command::Receive);
            break;
          },
          KeyCode::Char('T') => {
            self.set_cell(Command::TryTransmit);
            break;
          },
          KeyCode::Char('R') => {
            self.set_cell(Command::TryReceive);
            break;
          },

          // Starting location
          KeyCode::Char('b') => {
            current_program!(self).set_start(self.cursor_row as usize, self.cursor_col as usize);
            break;
          },

          _ => {},
        },
        _ => {},
      }
    }

    let level_id = global_state.level(self.level_index).id();
    <GlobalState as SolutionManager<isa::Parallel>>::save_solution(
      global_state,
      level_id,
      self.solution_index,
      self.solution.clone(),
    );

    Ok(Some(self))
  }
}
