use std::io::{self, Write};

use crossterm::{
  cursor,
  event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
  style::Stylize,
  terminal::{self, ClearType},
  ExecutableCommand, QueueableCommand,
};

use crate::global_state::GlobalState;

const MIN_WIDTH: u16 = 80;
const MIN_HEIGHT: u16 = 24;

pub trait State {
  fn render(&mut self, global_state: &mut GlobalState) -> io::Result<()>;

  fn execute(self: Box<Self>, global_state: &mut GlobalState) -> io::Result<Option<Box<dyn State>>>;
}

// Run the main game loop
pub fn run(mut state: Box<dyn State>, global_state: &mut GlobalState) -> io::Result<()> {
  let mut stdout = io::stdout();
  stdout
    .queue(terminal::EnterAlternateScreen)?
    .execute(event::EnableMouseCapture)?;
  terminal::enable_raw_mode()?;

  loop {
    stdout
      .queue(terminal::Clear(ClearType::All))?
      .queue(cursor::MoveTo(0, 0))?
      .queue(cursor::Show)?;

    // Special case, show a warning if the terminal is too small
    let (cols, rows) = terminal::size()?;
    if rows < MIN_HEIGHT || cols < MIN_WIDTH {
      if wait_for_window_resize()? {
        continue;
      }
      break;
    }

    // Render the current state
    state.render(global_state)?;

    // Then handle any user interaction
    match state.execute(global_state)? {
      Some(next_state) => {
        state = next_state;
      },
      None => break,
    }
  }

  terminal::disable_raw_mode()?;
  stdout
    .queue(cursor::Show)?
    .queue(cursor::EnableBlinking)?
    .queue(event::DisableMouseCapture)?
    .execute(terminal::LeaveAlternateScreen)?;
  Ok(())
}

// Loop that runs to check when the terminal window gets the right size again
//    Returns Ok(false) to kill the program
//    Returns Ok(true) to continue execution
fn wait_for_window_resize() -> io::Result<bool> {
  let mut stdout = io::stdout();
  loop {
    let (cols, rows) = terminal::size()?;
    if rows < MIN_HEIGHT || cols < MIN_WIDTH {
      loop {
        let (cols, rows) = terminal::size()?;
        stdout.queue(cursor::MoveTo(0, 0))?.queue(cursor::Hide)?;
        write!(stdout, "{}", "Terminal window is too small:".red())?;
        stdout.queue(cursor::MoveToNextLine(1))?;
        write!(stdout, "  ∙ Current Size: {}x{}", cols, rows)?;
        stdout.queue(cursor::MoveToNextLine(1))?;
        write!(stdout, "  ∙ Required Size: {}x{}", MIN_WIDTH, MIN_HEIGHT)?;
        stdout.flush()?;

        let event = match event::read() {
          Ok(e) => e,
          Err(_) => return Ok(false),
        };

        match event {
          Event::Resize(cols, rows) if rows >= MIN_HEIGHT && cols >= MIN_WIDTH => {
            return Ok(true);
          },

          Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
              return Ok(false);
            },

            _ => {},
          },

          _ => {},
        }
      }
    }
  }
}
