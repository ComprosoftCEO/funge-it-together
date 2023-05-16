use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use crossterm::{cursor, QueueableCommand};

use super::{LevelSelectState, State};
use crate::global_state::GlobalState;

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
   ╚═╝    ╚═════╝  ╚═════╝ ╚══════╝   ╚═╝   ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝

                Created by Bryan McClain
                 © Comprosoft 2023"#;

pub struct TitleState;

impl TitleState {
  pub fn new() -> Self {
    Self
  }
}

impl State for TitleState {
  fn render(&mut self, _global_state: &mut GlobalState) -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.queue(cursor::Hide)?.queue(cursor::MoveTo(0, 0))?;

    for line in TITLE.lines() {
      write!(stdout, "{}", line)?;
      stdout.queue(cursor::MoveToNextLine(1))?;
    }

    stdout.flush()?;

    Ok(())
  }

  fn execute(self: Box<Self>, _global_state: &mut GlobalState) -> io::Result<Option<Box<dyn State>>> {
    thread::sleep(Duration::from_secs(1));
    Ok(Some(Box::new(LevelSelectState::new(0))))
  }
}
