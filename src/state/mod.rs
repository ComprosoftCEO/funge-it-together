use crossterm::{cursor, QueueableCommand};
use std::io::{self, Write};

mod editor_state;
mod execute_state;
mod level_select_state;
mod show_help_state;
mod state;
mod success_state;
mod title_state;

pub use editor_state::EditorState;
pub use execute_state::ExecuteState;
pub use level_select_state::LevelSelectState;
pub use show_help_state::ShowHelpState;
pub use state::{run, State};
pub use success_state::SuccessState;
pub use title_state::TitleState;

// Print string in the same column, uses save/restore
pub(crate) fn print_string(s: &str) -> io::Result<()> {
  let mut stdout = io::stdout();
  stdout.queue(cursor::SavePosition)?;

  for (line, i) in s.lines().zip(0..) {
    stdout.queue(cursor::RestorePosition)?.queue(cursor::MoveDown(i))?;
    write!(stdout, "{}", line)?;
  }

  Ok(())
}
