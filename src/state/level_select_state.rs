use std::io::{self, Write};
use std::iter;

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::style::{self, Color, Stylize};
use crossterm::{cursor, event, QueueableCommand};

use super::{print_string, ShowHelpState, State, MIN_TERMINAL_WIDTH};
use crate::global_state::GlobalState;
use crate::isa::{self, InstructionSetArchitecture};
use crate::level::{Level, LevelIndex, LevelType};
use crate::statistics::Statistics;

const SEED: u32 = 0xdeadbeef;
const NUM_TEST_CASES: usize = 25;
const LEVELS_PER_PAGE: usize = 12;

static TITLE: &str = r#"  ___            ___  ___    _ ___    ___  __   ___  ___ ___       ___  ___ 
 |___ |  | |\ | | __ |___    |  |      |  |  | | __ |___  |  |__| |___ |__/ 
 |    |__| | \| |__] |___    |  |      |  |__| |__] |___  |  |  | |___ |  \

"#;

pub struct LevelSelectState {
  selected_level_pack_index: usize,
  selected_level_indexes: Vec<usize>,
  last_error: Option<String>,
  saved: bool,
  page_offset: usize,
}

enum LevelListEntry<'l> {
  NormalLevel {
    level: &'l Level,
    statistics: Option<Statistics>,
  },
  ChallengeLevel {
    level: &'l Level,
    statistics: Option<Statistics>,
  },
  LockedChallenge,
}

macro_rules! level_select_list {
  (($self:ident, $level_index:ident, $level:expr), [ $(($match_type:pat, $isa_type:ty),)+ ]) => {
    match $level.level_type() { $(
      $match_type => {
        let test_cases = match <$isa_type as InstructionSetArchitecture>::generate_test_cases($level.lua_file(), SEED, NUM_TEST_CASES) {
          Ok(t) => t,
          Err(e) => {
            $self.last_error = Some(format!("Failed to generate test cases: {e}"));
            return Ok(Some($self));
          },
        };

        return Ok(Some(Box::new(ShowHelpState::<$isa_type>::new(*$level_index, 0, test_cases))));
      }
    )+ }
  };
}

impl LevelSelectState {
  pub fn new(level_index: LevelIndex, global_state: &GlobalState) -> Self {
    let selected_level_pack_index = level_index.get_level_pack_index();
    let selected_level_index = global_state
      .get_level_pack(selected_level_pack_index)
      .get_absolute_index(level_index);

    let mut selected_level_indexes = vec![0; global_state.num_level_packs()];
    selected_level_indexes[selected_level_pack_index] = selected_level_index;

    let mut state = Self {
      selected_level_pack_index,
      selected_level_indexes,
      last_error: None,
      saved: false,
      page_offset: 0,
    };
    state.fix_page_offset();
    state
  }

  fn fix_page_offset(&mut self) {
    loop {
      let selected_level_index = self.selected_level_indexes[self.selected_level_pack_index];
      match (selected_level_index as isize) - (self.page_offset as isize) {
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

  fn get_flattened_level_list<'l>(&self, global_state: &'l GlobalState) -> Vec<(LevelListEntry<'l>, LevelIndex)> {
    use LevelListEntry::*;

    let mut take_next_group = true; // We always take the first group
    let unlocked_groups: Vec<_> = global_state
      .get_level_pack(self.selected_level_pack_index)
      .level_groups()
      .iter()
      .take_while(|lg| {
        let ret = take_next_group;
        take_next_group = lg.is_complete(global_state);
        ret
      })
      .collect();

    unlocked_groups
      .into_iter()
      .enumerate()
      .flat_map(|(group, lg)| {
        lg.main_levels()
          .iter()
          .enumerate()
          .flat_map(move |(level_in_group, level)| {
            let mut prev_level = level.level();

            // Challenges require the previous level to be unlocked
            let challenges_list = level.challenge_levels().iter().enumerate().map(move |(challenge, cl)| {
              let ret_val = if global_state.is_level_complete(prev_level.id()) {
                ChallengeLevel {
                  level: cl,
                  statistics: global_state.get_statistics(cl.id()),
                }
              } else {
                LockedChallenge
              };
              prev_level = cl;
              (
                ret_val,
                LevelIndex::new_challenge(self.selected_level_pack_index, group, level_in_group, challenge),
              )
            });

            // The main level and any challenge levels
            iter::once((
              NormalLevel {
                level: level.level(),
                statistics: global_state.get_statistics(level.level().id()),
              },
              LevelIndex::new(self.selected_level_pack_index, group, level_in_group),
            ))
            .chain(challenges_list)
          })
      })
      .collect()
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

    // Level pack selector
    let left_arrow = if self.selected_level_pack_index > 0 { '←' } else { ' ' };
    let right_arrow = if self.selected_level_pack_index < global_state.num_level_packs() - 1 {
      '→'
    } else {
      ' '
    };

    let level_pack_str = format!(
      "{left_arrow} {:^40} {right_arrow}",
      global_state.get_level_pack(self.selected_level_pack_index).name(),
    );

    write!(stdout, "{:^len$}", level_pack_str, len = MIN_TERMINAL_WIDTH as usize)?;
    stdout.queue(cursor::MoveToNextLine(1))?;

    if self.page_offset > 0 {
      write!(stdout, "↑")?;
    }
    stdout.queue(cursor::MoveToNextLine(1))?;

    let level_list = self.get_flattened_level_list(global_state);

    let selected_level_index = self.selected_level_indexes[self.selected_level_pack_index];
    for ((level_entry, level_index), absolute_index) in level_list
      .iter()
      .skip(self.page_offset)
      .take(LEVELS_PER_PAGE)
      .zip((self.page_offset + 1)..)
    {
      if (selected_level_index + 1) == absolute_index {
        write!(stdout, "{} ", "►".green())?;
      } else {
        write!(stdout, "  ")?;
      }

      level_entry.print_entry(*level_index)?;

      stdout.queue(style::ResetColor)?.queue(cursor::MoveToNextLine(1))?;
    }

    if (self.page_offset as isize) < (level_list.len() as isize - LEVELS_PER_PAGE as isize) {
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
    let num_level_packs = global_state.num_level_packs();
    let level_list = self.get_flattened_level_list(global_state);

    let num_options = level_list.len();
    let selected_level_index = self.selected_level_indexes[self.selected_level_pack_index];
    let (selected_level, level_index) = &level_list[selected_level_index];

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

          // Level Movement
          KeyCode::Up | KeyCode::Char('k') => {
            self.last_error = None;
            self.selected_level_indexes[self.selected_level_pack_index] =
              (selected_level_index as isize - 1).rem_euclid(num_options as isize) as usize;
            self.fix_page_offset();

            return Ok(Some(self));
          },
          KeyCode::Down | KeyCode::Char('j') => {
            self.last_error = None;
            self.selected_level_indexes[self.selected_level_pack_index] =
              (selected_level_index as isize + 1).rem_euclid(num_options as isize) as usize;
            self.fix_page_offset();

            return Ok(Some(self));
          },

          // Level pack movement
          KeyCode::Left | KeyCode::Char('h') => {
            self.last_error = None;
            self.selected_level_pack_index =
              (self.selected_level_pack_index as isize - 1).rem_euclid(num_level_packs as isize) as usize;
            self.fix_page_offset();

            return Ok(Some(self));
          },
          KeyCode::Right | KeyCode::Char('l') => {
            self.last_error = None;
            self.selected_level_pack_index =
              (self.selected_level_pack_index as isize + 1).rem_euclid(num_level_packs as isize) as usize;
            self.fix_page_offset();

            return Ok(Some(self));
          },

          // Select Level
          KeyCode::Enter if selected_level.is_unlocked() => {
            let level = global_state.level(*level_index);
            level_select_list!(
              (self, level_index, level),
              [
                (LevelType::Standard, isa::Standard),
                (LevelType::Parallel, isa::Parallel),
              ]
            );
          },

          _ => {},
        },
        _ => {},
      }
    }
  }
}

impl<'l> LevelListEntry<'l> {
  /// Can we actually play this level?
  pub fn is_unlocked(&self) -> bool {
    matches!(
      self,
      LevelListEntry::NormalLevel { .. } | LevelListEntry::ChallengeLevel { .. }
    )
  }

  /// Challenge levels are tabbed
  pub fn is_challenge(&self) -> bool {
    matches!(
      self,
      LevelListEntry::ChallengeLevel { .. } | LevelListEntry::LockedChallenge
    )
  }

  pub fn print_entry(&self, level_index: LevelIndex) -> io::Result<()> {
    use LevelListEntry::*;

    let mut stdout = io::stdout();
    let challenge_index = level_index.get_challenge().unwrap_or(0) + 1;

    let (level, statistics) = match self {
      NormalLevel { level, statistics } => (level, statistics),
      ChallengeLevel { level, statistics } => (level, statistics),
      LockedChallenge => {
        return write!(stdout, "  Challenge {}: Locked", challenge_index);
      },
    };

    // Challenge levels get an indent
    let level_text = if self.is_challenge() {
      format!("  {:38}", format!("Challenge {}: {}", challenge_index, level.name()))
    } else {
      format!("{:40}", level.get_title(level_index))
    };

    // Color text depending on if it is completed or not
    if let Some(statistics) = statistics {
      write!(
        stdout,
        "{}{} {: <10.2}   {} {}",
        level_text.dark_green(),
        "Cycles:".dark_yellow(),
        statistics.average_cycles(),
        "Symbols:".dark_cyan(),
        statistics.symbols_used()
      )?;
    } else {
      write!(stdout, "{}", level_text.yellow())?;
    }

    Ok(())
  }
}
