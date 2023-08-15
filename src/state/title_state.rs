use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::style::Stylize;
use crossterm::{cursor, event, QueueableCommand};

use super::{LevelSelectState, State};
use crate::global_state::GlobalState;
use crate::level::LevelIndex;

static TITLE: &str = r#"
    ███████╗██╗   ██╗███╗   ██╗ ██████╗ ███████╗    ██╗████████╗        
    ██╔════╝██║   ██║████╗  ██║██╔════╝ ██╔════╝    ██║╚══██╔══╝        
    █████╗  ██║   ██║██╔██╗ ██║██║  ███╗█████╗      ██║   ██║           
    ██╔══╝  ██║   ██║██║╚██╗██║██║   ██║██╔══╝      ██║   ██║           
    ██║     ╚██████╔╝██║ ╚████║╚██████╔╝███████╗    ██║   ██║           
    ╚═╝      ╚═════╝ ╚═╝  ╚═══╝ ╚═════╝ ╚══════╝    ╚═╝   ╚═╝           
████████╗ ██████╗  ██████╗ ███████╗████████╗██╗  ██╗███████╗██████╗ 
╚══██╔══╝██╔═══██╗██╔════╝ ██╔════╝╚══██╔══╝██║  ██║██╔════╝██╔══██╗
   ██║   ██║   ██║██║  ███╗█████╗     ██║   ███████║█████╗  ██████╔╝
   ██║   ██║   ██║██║   ██║██╔══╝     ██║   ██╔══██║██╔══╝  ██╔══██╗
   ██║   ╚██████╔╝╚██████╔╝███████╗   ██║   ██║  ██║███████╗██║  ██║
   ╚═╝    ╚═════╝  ╚═════╝ ╚══════╝   ╚═╝   ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝"#;

static VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));
static CREATOR: &str = "Created by Bryan McClain";
static COPYRIGHT: &str = "© Comprosoft 2023";

pub struct TitleState {
  now: Instant,
}

impl TitleState {
  pub fn new() -> Self {
    Self { now: Instant::now() }
  }
}

impl State for TitleState {
  fn render(&mut self, _global_state: &mut GlobalState) -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.queue(cursor::Hide)?.queue(cursor::MoveTo(0, 0))?;

    for line in TITLE.lines() {
      write!(stdout, "{}", line.dark_cyan())?;
      stdout.queue(cursor::MoveToNextLine(1))?;
    }

    write!(stdout, "                            {}", VERSION)?;
    stdout.queue(cursor::MoveToNextLine(2))?;
    write!(stdout, "                    {}", CREATOR.dark_yellow())?;
    stdout.queue(cursor::MoveToNextLine(1))?;
    write!(stdout, "                       {}", COPYRIGHT.dark_green())?;

    stdout.flush()?;

    Ok(())
  }

  fn execute(self: Box<Self>, global_state: &mut GlobalState) -> io::Result<Option<Box<dyn State>>> {
    let elapsed = self.now.elapsed();
    if elapsed > Duration::from_secs(2) {
      return Ok(Some(Box::new(LevelSelectState::new(
        LevelIndex::default(),
        global_state,
      ))));
    }

    if event::poll(Duration::from_millis(100))? {
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

          KeyCode::Enter => {
            return Ok(Some(Box::new(LevelSelectState::new(
              LevelIndex::default(),
              global_state,
            ))))
          },
          KeyCode::Esc => return Ok(None),

          _ => {},
        },

        _ => {},
      }

      // Then clear all pending events
      while event::poll(Duration::from_secs(0))? {
        event::read()?;
      }
    }

    Ok(Some(self))
  }
}
