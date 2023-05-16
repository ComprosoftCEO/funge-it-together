use std::io::{self, Write};
use std::iter;

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::style::{self, Color, Stylize};
use crossterm::{cursor, event, QueueableCommand};

use super::{print_string, EditorState, ShowTextState, State};
use crate::global_state::GlobalState;

const SEED: u32 = 0xdeadbeef;
const NUM_TEST_CASES: usize = 25;

pub struct LevelSelectState {
  selected_level_index: isize,
  last_error: Option<String>,
  saved: bool,
}

impl LevelSelectState {
  pub fn new(selected_level_index: usize) -> Self {
    Self {
      selected_level_index: selected_level_index as isize,
      last_error: None,
      saved: false,
    }
  }
}

impl State for LevelSelectState {
  fn render(&mut self, global_state: &mut GlobalState) -> io::Result<()> {
    // Try to save the game on the first render
    if !self.saved {
      self.saved = true;
      if let Err(e) = global_state.save() {
        self.last_error = Some(format!("Failed to save game: {e}"));
      }
    }

    let mut stdout = io::stdout();
    stdout.queue(cursor::Hide)?.queue(cursor::MoveTo(0, 0))?;

    write!(stdout, "Level Select:")?;
    stdout.queue(cursor::MoveToNextLine(2))?;

    let completed_levels = global_state.completed_levels();
    let all_levels_completed = completed_levels.len() == global_state.levels().len();

    let unlocked_levels = if all_levels_completed {
      completed_levels
    } else {
      let next_level = global_state.level(completed_levels.len()).clone();
      completed_levels.into_iter().chain(iter::once(next_level)).collect()
    };

    for (level, level_number) in unlocked_levels.iter().zip(1..) {
      if (self.selected_level_index + 1) == level_number {
        write!(stdout, "{}", "►".green())?;
      } else {
        write!(stdout, " ")?;
      }

      let text = format!(" Level {:2<} - {}", level_number, level.name());
      if level_number as usize == unlocked_levels.len() && !all_levels_completed {
        write!(stdout, "{:40}", text)?;
      } else {
        stdout.queue(style::SetForegroundColor(Color::DarkGreen))?;
        write!(stdout, "{:40}", text)?;
      }

      if let Some(statistics) = global_state.get_statistics(level.id()) {
        stdout.queue(style::SetForegroundColor(Color::DarkYellow))?;
        write!(
          stdout,
          "{: <20}   {}",
          format!("Cycles: {:.2}", statistics.average_cycles()),
          format!("Symbols: {}", statistics.symbols_used())
        )?;
      }

      stdout.queue(style::ResetColor)?.queue(cursor::MoveToNextLine(1))?;
    }

    // Special case: end message
    if all_levels_completed {
      stdout.queue(cursor::MoveToNextLine(1))?;
      if self.selected_level_index as usize == unlocked_levels.len() {
        write!(stdout, "{}", "►".green())?;
      } else {
        write!(stdout, " ")?;
      }

      write!(stdout, "{}", " Win Message".cyan())?;
    }

    if let Some(ref err) = self.last_error {
      stdout
        .queue(cursor::MoveToNextLine(2))?
        .queue(style::SetForegroundColor(Color::Red))?;
      print_string(err)?;
      stdout.queue(style::ResetColor)?;
    }

    stdout.flush()?;

    Ok(())
  }

  fn execute(mut self: Box<Self>, global_state: &mut GlobalState) -> io::Result<Option<Box<dyn State>>> {
    let completed_levels = global_state.completed_levels();
    let num_options = completed_levels.len() + 1;
    let all_levels_completed = completed_levels.len() == global_state.levels().len();

    loop {
      // `read()` blocks until an `Event` is available
      let event = match event::read() {
        Ok(e) => e,
        Err(_) => return Ok(None),
      };

      match event {
        Event::Resize(_, _) => {
          return Ok(Some(self));
        },

        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
          KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Ok(None);
          },

          // Close the game
          KeyCode::Esc => return Ok(None),

          // Movement
          KeyCode::Up => {
            self.last_error = None;
            self.selected_level_index = (self.selected_level_index - 1).rem_euclid(num_options as isize);
            return Ok(Some(self));
          },
          KeyCode::Down => {
            self.last_error = None;
            self.selected_level_index = (self.selected_level_index + 1).rem_euclid(num_options as isize);
            return Ok(Some(self));
          },

          KeyCode::Enter if all_levels_completed && self.selected_level_index as usize == completed_levels.len() => {
            return Ok(Some(Box::new(ShowTextState::new(global_state.message(), self))))
          },

          KeyCode::Enter => {
            let level = global_state.level(self.selected_level_index as usize);
            let solution = global_state.get_solution(level.id());
            let test_cases = match level.generate_test_cases(SEED, NUM_TEST_CASES) {
              Ok(t) => t,
              Err(e) => {
                self.last_error = Some(format!("Failed to generate test cases: {e}"));
                return Ok(Some(self));
              },
            };

            let editor = Box::new(EditorState::new(
              self.selected_level_index as usize,
              solution,
              test_cases,
              0,
            ));

            return Ok(Some(Box::new(ShowTextState::new(
              level.get_full_text(self.selected_level_index as usize),
              editor,
            ))));
          },

          _ => {},
        },
        _ => {},
      }
    }
  }
}
