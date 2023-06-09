use std::io::{self, Write};
use std::iter;

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::style::{self, Color, Stylize};
use crossterm::{cursor, event, QueueableCommand};

use super::{print_string, ShowHelpState, State};
use crate::global_state::GlobalState;

const SEED: u32 = 0xdeadbeef;
const NUM_TEST_CASES: usize = 25;
const LEVELS_PER_PAGE: usize = 12;

static TITLE: &str = r#"  ___            ___  ___    _ ___    ___  __   ___  ___ ___       ___  ___ 
 |___ |  | |\ | | __ |___    |  |      |  |  | | __ |___  |  |__| |___ |__/ 
 |    |__| | \| |__] |___    |  |      |  |__| |__] |___  |  |  | |___ |  \

"#;

pub struct LevelSelectState {
  selected_level_index: isize,
  last_error: Option<String>,
  saved: bool,
  page_offset: usize,
}

impl LevelSelectState {
  pub fn new(selected_level_index: usize) -> Self {
    let mut state = Self {
      selected_level_index: selected_level_index as isize,
      last_error: None,
      saved: false,
      page_offset: 0,
    };
    state.fix_page_offset();
    state
  }

  fn fix_page_offset(&mut self) {
    loop {
      match self.selected_level_index - self.page_offset as isize {
        x if x >= (LEVELS_PER_PAGE as isize) => {
          self.page_offset += 1;
        },
        x if x < 0 => {
          self.page_offset -= 1;
        },
        _ => break,
      }
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

    for line in TITLE.lines() {
      write!(stdout, "{}", line.dark_cyan())?;
      stdout.queue(cursor::MoveToNextLine(1))?;
    }

    write!(stdout, "Level Select:")?;
    stdout.queue(cursor::MoveToNextLine(1))?;

    if self.page_offset > 0 {
      write!(stdout, "↑")?;
    }
    stdout.queue(cursor::MoveToNextLine(1))?;

    let completed_levels = global_state.completed_levels();
    let all_levels_completed = completed_levels.len() == global_state.levels().len();

    let unlocked_levels = if all_levels_completed {
      completed_levels
    } else {
      let next_level = global_state.level(completed_levels.len()).clone();
      completed_levels.into_iter().chain(iter::once(next_level)).collect()
    };

    for (level, level_number) in unlocked_levels
      .iter()
      .skip(self.page_offset)
      .take(LEVELS_PER_PAGE)
      .zip((self.page_offset + 1)..)
    {
      if (self.selected_level_index + 1) == (level_number as isize) {
        write!(stdout, "{}", "►".green())?;
      } else {
        write!(stdout, " ")?;
      }

      // Color text depending on if it is completed or not
      let text = format!(" Level {:2<} - {}", level_number, level.name());
      if level_number as usize == unlocked_levels.len() && !all_levels_completed {
        stdout.queue(style::SetForegroundColor(Color::Yellow))?;
        write!(stdout, "{:40}", text)?;
      } else {
        stdout.queue(style::SetForegroundColor(Color::DarkGreen))?;
        write!(stdout, "{:40}", text)?;
      }

      if let Some(statistics) = global_state.get_statistics(level.id()) {
        write!(
          stdout,
          "{} {: <10.2}",
          "Cycles:".dark_yellow(),
          statistics.average_cycles()
        )?;
        write!(stdout, "   {} {}", "Symbols:".dark_cyan(), statistics.symbols_used())?;
      }

      stdout.queue(style::ResetColor)?.queue(cursor::MoveToNextLine(1))?;
    }

    if (self.page_offset as isize) < (unlocked_levels.len() as isize - LEVELS_PER_PAGE as isize) {
      write!(stdout, "↓")?;
    }

    if let Some(ref err) = self.last_error {
      stdout
        .queue(cursor::MoveToNextLine(1))?
        .queue(style::SetForegroundColor(Color::Red))?;
      print_string(err)?;
      stdout.queue(style::ResetColor)?;
    }

    stdout.flush()?;

    Ok(())
  }

  fn execute(mut self: Box<Self>, global_state: &mut GlobalState) -> io::Result<Option<Box<dyn State>>> {
    let completed_levels = global_state.completed_levels();
    let all_levels_completed = completed_levels.len() == global_state.levels().len();

    let num_options = if all_levels_completed {
      completed_levels.len() // No next level to unlock
    } else {
      completed_levels.len() + 1 // There is a next level to unlock
    };

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
          KeyCode::Up | KeyCode::Char('k') => {
            self.last_error = None;
            self.selected_level_index = (self.selected_level_index - 1).rem_euclid(num_options as isize);
            self.fix_page_offset();

            return Ok(Some(self));
          },
          KeyCode::Down | KeyCode::Char('j') => {
            self.last_error = None;
            self.selected_level_index = (self.selected_level_index + 1).rem_euclid(num_options as isize);
            self.fix_page_offset();

            return Ok(Some(self));
          },

          // Select Level
          KeyCode::Enter => {
            let level = global_state.level(self.selected_level_index as usize);
            let test_cases = match level.generate_test_cases(SEED, NUM_TEST_CASES) {
              Ok(t) => t,
              Err(e) => {
                self.last_error = Some(format!("Failed to generate test cases: {e}"));
                return Ok(Some(self));
              },
            };

            return Ok(Some(Box::new(ShowHelpState::new(
              self.selected_level_index as usize,
              0,
              test_cases,
            ))));
          },

          _ => {},
        },
        _ => {},
      }
    }
  }
}
