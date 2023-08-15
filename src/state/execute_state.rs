use crossterm::{
  cursor,
  event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
  style::{self, Color, Stylize},
  QueueableCommand,
};
use rand::Rng;
use std::{
  io::{self, Write},
  thread,
  time::Duration,
};

use super::{print_string, EditorState, State, SuccessState};
use crate::{
  global_state::{GlobalState, Statistics},
  level::LevelIndex,
  printable::Printable,
  vm::{VMError, VirtualMachine},
};

static INSTRUCTIONS: &str = r#"
│Esc    = Editor
│Ctrl-C = Close Program
│
│Tab    = Step
│Space  = Start/Stop
│1-6    = Set Speed
│,      = Breakpoint
│
│
│
│
│
│
│
│
│
│
│
│
│
│
│
│"#;

pub struct ExecuteState {
  editor: EditorState,
  level_index: LevelIndex,

  vms: Vec<VirtualMachine>,
  test_case: usize,
  speed: Speed,

  last_error: Option<VMError>,

  total_cycles: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speed {
  None,
  Slow,
  Normal,
  Fast,
  ExtremelyFast,
  Turbo,
  SuperTurbo,
}

enum StepResult {
  Continue(Box<ExecuteState>),
  OtherState(Box<dyn State>),
}

impl ExecuteState {
  pub fn new(editor: EditorState, speed: Speed) -> Self {
    let level_index = editor.level_index();
    let vms = editor.vms();

    Self {
      editor,
      level_index,
      vms,
      test_case: 0,
      speed,
      last_error: None,
      total_cycles: 0.0,
    }
  }

  fn step_vm(mut self: Box<Self>, global_state: &mut GlobalState) -> StepResult {
    let current_vm = &mut self.vms[self.test_case];
    let step = current_vm.step();
    if current_vm.is_at_breakpoint() {
      self.speed = Speed::None;
    }

    match step {
      Ok(false) => StepResult::Continue(self),
      Ok(true) => {
        self.total_cycles += current_vm.get_cycle() as f64;
        let num_symbols = current_vm.count_symbols();
        let num_vms = self.vms.len() as f64;

        self.test_case += 1;
        if self.test_case == self.vms.len() {
          let level_id = global_state.level(self.level_index).id();
          let statistics = Statistics::new(self.total_cycles / num_vms, num_symbols);

          let best = global_state.complete_level(level_id, statistics.clone());
          return StepResult::OtherState(Box::new(SuccessState::new(
            self.level_index,
            statistics,
            best,
            self.editor,
          )));
        }

        StepResult::Continue(self)
      },
      Err(e) => {
        self.last_error = Some(e);
        StepResult::OtherState(Box::new(*self))
      },
    }
  }
}

impl State for ExecuteState {
  fn render(&mut self, global_state: &mut GlobalState) -> std::io::Result<()> {
    let mut stdout = io::stdout();

    let level = global_state.level(self.level_index);
    stdout.queue(cursor::Hide)?;
    write!(stdout, "     {}", level.get_title(self.level_index).yellow())?;

    self.vms[self.test_case].print_at(2, 0)?;

    if let Some(ref last_error) = self.last_error {
      self.vms[self.test_case].print_error_symbol_at(2, 0)?;
      last_error.print_at(self.vms[0].rows() as u16 + 2 + 2 + 4, 0)?;
    }

    stdout
      .queue(cursor::MoveTo(55, 0))?
      .queue(style::SetForegroundColor(Color::DarkCyan))?;
    print_string(INSTRUCTIONS)?;
    stdout.queue(style::ResetColor)?;

    stdout.flush()?;
    Ok(())
  }

  fn execute(mut self: Box<Self>, global_state: &mut GlobalState) -> io::Result<Option<Box<dyn State>>> {
    // If an error occured, then wait until they press escape to go back
    if self.last_error.is_some() {
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

            KeyCode::Esc => return Ok(Some(Box::new(self.editor))),

            _ => {},
          },

          _ => {},
        }
      }
    }

    for _ in 0..self.speed.num_steps() {
      self = match self.step_vm(global_state) {
        StepResult::Continue(s) => s,
        result @ StepResult::OtherState(_) => return Ok(result.into_box()),
      };

      if self.speed == Speed::None {
        return Ok(Some(self)); // Reached a breakpoint
      }
    }

    if self.speed != Speed::None {
      thread::sleep(Duration::from_millis(100));
    }

    if self.speed == Speed::None || event::poll(Duration::from_secs(0))? {
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

          // Single step
          KeyCode::Tab => {
            self.speed = Speed::None;
            return Ok(self.step_vm(global_state).into_box());
          },

          // Start/Stop
          KeyCode::Char(' ') => {
            if self.speed != Speed::None {
              self.speed = Speed::None;
            } else {
              self.speed = Speed::Slow;
            }
          },

          // Set Speed
          KeyCode::Char('1') => {
            self.speed = Speed::Slow;
          },
          KeyCode::Char('2') => {
            self.speed = Speed::Normal;
          },
          KeyCode::Char('3') => {
            self.speed = Speed::Fast;
          },
          KeyCode::Char('4') => {
            self.speed = Speed::ExtremelyFast;
          },
          KeyCode::Char('5') => {
            self.speed = Speed::Turbo;
          },
          KeyCode::Char('6') => {
            self.speed = Speed::SuperTurbo;
          },

          // Breakpoint
          KeyCode::Char(',') if self.speed == Speed::None => {
            let current_vm = &mut self.vms[self.test_case];
            let row = current_vm.row();
            let col = current_vm.col();
            for vm in self.vms.iter_mut() {
              vm.toggle_breakpoint(row, col);
            }
            self.editor.toggle_breakpoint(row, col);
            return Ok(Some(self));
          },

          // Go back
          KeyCode::Esc => {
            return Ok(Some(Box::new(self.editor)));
          },

          _ => {},
        },

        _ => {},
      }

      // Then clear all pending events
      if self.speed != Speed::None {
        while event::poll(Duration::from_secs(0))? {
          event::read()?;
        }
      }
    }

    Ok(Some(self))
  }
}

impl Speed {
  pub fn num_steps(self) -> usize {
    let mut rng = rand::thread_rng();
    match self {
      Self::None => 0,
      Self::Slow => 1,
      Self::Normal => rng.gen_range(4..=8),
      Self::Fast => rng.gen_range(20..=30),
      Self::ExtremelyFast => rng.gen_range(90..=110),
      Self::Turbo => rng.gen_range(900..=1100),
      Self::SuperTurbo => rng.gen_range(9000..=11000),
    }
  }
}

impl StepResult {
  pub fn into_box(self) -> Option<Box<dyn State>> {
    match self {
      Self::Continue(s) => Some(s),
      Self::OtherState(s) => Some(s),
    }
  }
}
