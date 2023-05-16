use crossterm::{
  cursor,
  event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind},
  style::Stylize,
  ExecutableCommand, QueueableCommand,
};
use std::io::{self, Write};

use super::{execute_state::Speed, print_string, ExecuteState, LevelSelectState, ShowHelpState};
use crate::{
  global_state::{GlobalState, Solution},
  printable::Printable,
  puzzle::TestCaseSet,
  state::State,
  vm::{Command, VirtualMachine},
};

static INSTRUCTIONS: &str = r#"
│Esc    = Main Menu
│Ctrl-C = Close Program
│
│Tab    = Step
│Space  = Start/Stop
│h      = Level Help
│[  ]   = Test Case
│
│Arrow  = Move Cursor
│Delete = Clear
│asdw   = ←↓→↑ (Move)
│/ \    = / \ (Bounce)
│0-9    = 0-9
│x      = ☼ (Pop)
│c      = © (Copy)
│~      = ∫ (Swap)
│^ v    = ∩ u (Rotate)
│+ -    = (Add, Sub)
│< = >  = (Compare to 0)
│i      = Ї (Input)
│o      = Θ (Output)
│?      = (Has input?)
│b      = (Set start)"#;

pub struct EditorState {
  level_index: usize,

  solution: Solution,

  cursor_row: isize,
  cursor_col: isize,

  test_cases: TestCaseSet,
  test_case_index: isize,
}

impl EditorState {
  pub fn new(level_index: usize, solution: Solution, test_cases: TestCaseSet, test_case_index: usize) -> Self {
    Self {
      level_index,
      solution,
      cursor_row: 0,
      cursor_col: 0,
      test_cases,
      test_case_index: test_case_index as isize,
    }
  }

  fn set_cell(&mut self, command: Command) {
    self
      .solution
      .set_grid_value(self.cursor_row as usize, self.cursor_col as usize, command);
  }

  pub(crate) fn level_index(&self) -> usize {
    self.level_index
  }

  pub(crate) fn vms(&self) -> Vec<VirtualMachine> {
    (0..self.test_cases.len())
      .into_iter()
      .map(|i| {
        let index = (self.test_case_index as usize + i).rem_euclid(self.test_cases.len());
        VirtualMachine::new(self.solution.clone(), index + 1, &self.test_cases[index])
      })
      .collect()
  }
}

impl State for EditorState {
  fn render(&mut self, global_state: &mut GlobalState) -> io::Result<()> {
    let mut stdout = io::stdout();

    let level = &global_state.levels()[self.level_index];
    write!(
      stdout,
      "     {}",
      format!("Level {} - {}", self.level_index + 1, level.name()).yellow()
    )?;

    self.solution.print_at(2, 0)?;

    stdout
      .queue(cursor::MoveTo(self.solution.cols() as u16 + 2 + 8, 2))?
      .queue(cursor::SavePosition)?;

    write!(
      stdout,
      "{}",
      format!("Test Case {}", self.test_case_index + 1).dark_yellow()
    )?;
    stdout.queue(cursor::RestorePosition)?.queue(cursor::MoveDown(2))?;

    self.test_cases[self.test_case_index as usize].print()?;

    stdout.queue(cursor::MoveTo(55, 0))?;
    print_string(INSTRUCTIONS)?;

    stdout.queue(cursor::EnableBlinking)?.execute(cursor::MoveTo(
      self.cursor_col as u16 + 1,
      self.cursor_row as u16 + 1 + 2,
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
          let mouse_row = (mouse.row as isize) - 1 - 2;
          let mouse_col = (mouse.column as isize) - 1;
          if mouse_row < 0
            || mouse_row >= self.solution.rows() as isize
            || mouse_col < 0
            || mouse_col >= self.solution.cols() as isize
          {
            continue;
          }

          match mouse.kind {
            // Left button just selects the space
            MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Drag(MouseButton::Left) => {
              self.cursor_row = mouse_row;
              self.cursor_col = mouse_col;
              return Ok(Some(self));
            },

            // Right button clears
            MouseEventKind::Down(MouseButton::Right) | MouseEventKind::Drag(MouseButton::Right) => {
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

          KeyCode::Esc => return Ok(Some(Box::new(LevelSelectState::new(self.level_index)))),
          KeyCode::Char('h') => {
            let level = global_state.level(self.level_index);
            return Ok(Some(Box::new(ShowHelpState::new(
              level.get_title(self.level_index as usize),
              level.description(),
              self,
              None,
            ))));
          },

          // Start execution
          KeyCode::Tab => return Ok(Some(Box::new(ExecuteState::new(*self, Speed::None)))),
          KeyCode::Char(' ') => return Ok(Some(Box::new(ExecuteState::new(*self, Speed::Slow)))),

          // Movement
          KeyCode::Up => {
            self.cursor_row = (self.cursor_row - 1).rem_euclid(self.solution.rows() as isize);
            return Ok(Some(self));
          },
          KeyCode::Down => {
            self.cursor_row = (self.cursor_row + 1).rem_euclid(self.solution.rows() as isize);
            return Ok(Some(self));
          },
          KeyCode::Left => {
            self.cursor_col = (self.cursor_col - 1).rem_euclid(self.solution.cols() as isize);
            return Ok(Some(self));
          },
          KeyCode::Right => {
            self.cursor_col = (self.cursor_col + 1).rem_euclid(self.solution.cols() as isize);
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

          // Deletion
          KeyCode::Backspace => {
            self.set_cell(Command::Empty);
            break;
          },
          KeyCode::Delete => {
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
          KeyCode::Char('x') => {
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

          // Starting location
          KeyCode::Char('b') => {
            self
              .solution
              .set_start(self.cursor_row as usize, self.cursor_col as usize);
            break;
          },

          _ => {},
        },
        _ => {},
      }
    }

    let level_id = global_state.level(self.level_index).id();
    global_state.save_solution(level_id, &self.solution);

    return Ok(Some(self));
  }
}
