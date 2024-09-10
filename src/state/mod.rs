use crossterm::{cursor, QueueableCommand};
use std::io::{self, Write};

mod level_select_state;
mod show_help_state;
mod state;
mod success_state;
mod title_state;

pub use level_select_state::LevelSelectState;
pub use show_help_state::ShowHelpState;
#[allow(unused)]
pub use state::{run, State, MIN_TERMINAL_HEIGHT, MIN_TERMINAL_WIDTH};
pub use success_state::SuccessState;
pub use title_state::TitleState;

// Print string in the same column, uses save/restore
pub fn print_string(s: &str) -> io::Result<()> {
  let mut stdout = io::stdout();
  stdout.queue(cursor::SavePosition)?;

  for (line, i) in s.lines().zip(0..) {
    stdout.queue(cursor::RestorePosition)?.queue(cursor::MoveDown(i))?;
    write!(stdout, "{}", line)?;
  }

  Ok(())
}
