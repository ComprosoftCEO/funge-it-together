use crossterm::{
  cursor,
  event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
  style::Stylize,
  QueueableCommand,
};
use std::io::{self, Write};

use super::{EditorState, LevelSelectState, State};
use crate::{global_state::GlobalState, level::LevelIndex, puzzle::Puzzle};

const SOLUTIONS_PER_PAGE: usize = 3;

const MAX_NAME_LEN: usize = 30;
static COPY_STR: &str = " (Copy)";

static SELECT_INSTRUCTIONS: &str = r#"
───────────────────────────────────────────────────────┬────────────────────────
                                                       │c   = Make a Copy
                                                       │r   = Rename
                                                       │^ v = Rearrange Up/Down
                                                       │x   = Delete
                                                       │"#;

pub struct ShowHelpState {
  level_index: LevelIndex,
  selected_solution_index: isize,
  page_offset: usize,
  test_cases: Vec<Puzzle>,

  in_rename: Option<String>,
}

impl ShowHelpState {
  pub fn new(level_index: LevelIndex, selected_solution_index: usize, test_cases: Vec<Puzzle>) -> Self {
    let mut state = Self {
      level_index,
      selected_solution_index: selected_solution_index as isize,
      page_offset: 0,
      test_cases,
      in_rename: None,
    };
    state.fix_page_offset();
    state
  }

  fn fix_page_offset(&mut self) {
    loop {
      match self.selected_solution_index - self.page_offset as isize {
        x if x >= (SOLUTIONS_PER_PAGE as isize) => {
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

impl State for ShowHelpState {
  fn render(&mut self, global_state: &mut GlobalState) -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.queue(cursor::Hide)?;

    let level = global_state.level(self.level_index);

    write!(stdout, "{}", level.get_title(self.level_index).as_str().yellow())?;
    stdout.queue(cursor::MoveToNextLine(2))?;

    for line in level.description().lines() {
      write!(stdout, "{}", line)?;
      stdout.queue(cursor::MoveToNextLine(1))?;
    }

    stdout.queue(cursor::MoveTo(0, 17))?;
    for line in SELECT_INSTRUCTIONS.lines() {
      write!(stdout, "{}", line.dark_cyan())?;
      stdout.queue(cursor::MoveToNextLine(1))?;
    }
    stdout.queue(cursor::MoveTo(0, 19))?;

    if self.page_offset > 0 {
      write!(stdout, "↑")?;
    }
    stdout.queue(cursor::MoveToNextLine(1))?;

    let level_id = global_state.level(self.level_index).id();
    let solutions = global_state.get_solutions_mut(level_id);

    for (solution, solution_number) in solutions
      .iter()
      .skip(self.page_offset)
      .take(SOLUTIONS_PER_PAGE)
      .zip((self.page_offset + 1)..)
    {
      if (self.selected_solution_index + 1) == (solution_number as isize) {
        match self.in_rename {
          None => write!(stdout, "{}", "►".green())?,
          Some(ref cur_name) => {
            let color_fn = |s: &'static str| match cur_name.len() {
              0 => s.red(),
              _ => s.stylize(),
            };
            write!(stdout, "{} {}{}", color_fn("◊"), cur_name.as_str(), color_fn("_"))?;
          },
        }
      } else {
        write!(stdout, " ")?;
      }

      if self.in_rename.is_none() || (self.selected_solution_index + 1) != (solution_number as isize) {
        write!(stdout, " {}", solution.name())?;
      }
      stdout.queue(cursor::MoveToColumn(41))?;
      write!(stdout, "{} {}", "Symbols:".dark_cyan(), solution.symbols_used(),)?;

      stdout.queue(cursor::MoveToNextLine(1))?;
    }

    if (self.page_offset as isize) > (solutions.len() as isize - SOLUTIONS_PER_PAGE as isize) {
      if (self.selected_solution_index) == (solutions.len() as isize) {
        write!(stdout, "{}", "►".green())?;
      } else {
        write!(stdout, " ")?;
      }

      write!(stdout, " {}", "‹New Solution›".dark_yellow())?;
      stdout.queue(cursor::MoveToNextLine(1))?;
    }

    if (self.page_offset as isize) < ((solutions.len() + 1) as isize - SOLUTIONS_PER_PAGE as isize) {
      write!(stdout, "↓")?;
    }

    stdout.flush()?;

    Ok(())
  }

  fn execute(mut self: Box<Self>, global_state: &mut GlobalState) -> io::Result<Option<Box<dyn State>>> {
    let level_id = global_state.level(self.level_index).id();
    let num_options = global_state.get_solutions_mut(level_id).len() + 1;

    if let Some(cur_name) = self.in_rename.as_mut() {
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

            // Cancel rename
            KeyCode::Esc => {
              self.in_rename = None;
              return Ok(Some(self));
            },

            // Save rename
            KeyCode::Enter if !cur_name.is_empty() => {
              let solutions = global_state.get_solutions_mut(level_id);
              solutions[self.selected_solution_index as usize].rename(self.in_rename.take().unwrap());
              return Ok(Some(self));
            },

            KeyCode::Backspace => {
              cur_name.pop();
              return Ok(Some(self));
            },

            KeyCode::Char(c) if cur_name.len() < MAX_NAME_LEN => {
              cur_name.push(c);
              return Ok(Some(self));
            },

            _ => {},
          },
          _ => {},
        }
      }
    }

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

          // Go back
          KeyCode::Esc => {
            return Ok(Some(Box::new(LevelSelectState::new(self.level_index, global_state))));
          },

          // Movement
          KeyCode::Up | KeyCode::Char('k') => {
            self.selected_solution_index = (self.selected_solution_index - 1).rem_euclid(num_options as isize);
            self.fix_page_offset();

            return Ok(Some(self));
          },

          KeyCode::Down | KeyCode::Char('j') => {
            self.selected_solution_index = (self.selected_solution_index + 1).rem_euclid(num_options as isize);
            self.fix_page_offset();

            return Ok(Some(self));
          },

          // Select Solution
          KeyCode::Enter => {
            if self.selected_solution_index == (num_options - 1) as isize {
              global_state.new_solution(level_id);
              return Ok(Some(self));
            } else {
              let solution = global_state.get_solutions_mut(level_id)[self.selected_solution_index as usize].clone();
              return Ok(Some(Box::new(EditorState::new(
                self.level_index,
                self.selected_solution_index as usize,
                solution,
                self.test_cases,
                0,
              ))));
            }
          },

          // Copy Solution
          KeyCode::Char('c') if self.selected_solution_index < (num_options - 1) as isize => {
            let solutions = global_state.get_solutions_mut(level_id);
            let mut solution = solutions[self.selected_solution_index as usize].clone();
            solution.rename(format!("{}{}", remove_copy_suffix(solution.name()), COPY_STR));
            solutions.insert(self.selected_solution_index as usize + 1, solution);

            self.selected_solution_index += 1;
            self.fix_page_offset();

            return Ok(Some(self));
          },

          // Rename Solution
          KeyCode::Char('r') if self.selected_solution_index < (num_options - 1) as isize => {
            self.in_rename = Some(String::new());
            return Ok(Some(self));
          },

          // Move Solution
          KeyCode::Char('^')
            if self.selected_solution_index > 0 && self.selected_solution_index < (num_options - 1) as isize =>
          {
            let solutions = global_state.get_solutions_mut(level_id);
            solutions.swap(
              self.selected_solution_index as usize - 1,
              self.selected_solution_index as usize,
            );

            self.selected_solution_index -= 1;
            self.fix_page_offset();

            return Ok(Some(self));
          },
          KeyCode::Char('v') if self.selected_solution_index < (num_options as isize - 2) => {
            let solutions = global_state.get_solutions_mut(level_id);
            solutions.swap(
              self.selected_solution_index as usize + 1,
              self.selected_solution_index as usize,
            );

            self.selected_solution_index += 1;
            self.fix_page_offset();

            return Ok(Some(self));
          },

          // Delete Solution
          KeyCode::Char('x') if self.selected_solution_index < (num_options - 1) as isize => {
            let solutions = global_state.get_solutions_mut(level_id);
            solutions.remove(self.selected_solution_index as usize);
            return Ok(Some(self));
          },

          _ => {},
        },
        _ => {},
      }
    }
  }
}

fn remove_copy_suffix(s: &str) -> &str {
  if s.len() <= MAX_NAME_LEN {
    return s;
  }

  match s.strip_suffix(COPY_STR) {
    Some(s) => s,
    None => s,
  }
}
