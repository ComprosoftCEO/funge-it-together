use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufReader, ErrorKind};
use std::iter;
use std::path::Path;
use uuid::Uuid;

use crate::global_state::GlobalState;

const LEVELS_FOLDER: &str = "levels";
const PACK_FILE: &str = "pack.json";

/// Stores all details about the levels
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelPack {
  name: String,
  #[serde(rename = "levels")]
  groups: Vec<LevelGroup>,

  #[serde(skip)]
  folder: String,
}

/// Stores a group of levels that all unlock at once
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelGroup(Vec<MainLevel>);

/// Top-level object for a level, may have optional "challenge" levels
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MainLevel {
  #[serde(flatten)]
  level: Level,

  #[serde(default)]
  challenge_levels: Vec<Level>,
}

/// Single entry in the levels.json file
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Level {
  id: Uuid,
  name: String,
  description: String,
  #[serde(default)]
  r#type: LevelType,
  lua_file: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LevelType {
  Standard,
  Parallel,
}

impl Default for LevelType {
  fn default() -> Self {
    LevelType::Standard
  }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LevelIndex {
  pack_index: usize,
  group: usize,
  level_in_group: usize,
  challenge: Option<usize>,
}

impl LevelPack {
  pub fn name(&self) -> &String {
    &self.name
  }

  pub fn folder(&self) -> &String {
    &self.folder
  }

  pub fn level_groups(&self) -> &Vec<LevelGroup> {
    &self.groups
  }

  pub fn level_group(&self, index: usize) -> &LevelGroup {
    &self.groups[index]
  }

  pub fn get_absolute_index(&self, level_index: LevelIndex) -> usize {
    (0..level_index.group)
      .map(|group_index| self.groups[group_index].len())
      .sum::<usize>()
      + (0..level_index.level_in_group)
        .map(|level_in_group| self.groups[level_index.group].0[level_in_group].len())
        .sum::<usize>()
      + level_index.challenge.map(|x| x + 1).unwrap_or(0)
  }

  ///
  /// Load a level pack from a folder
  ///   Returns an error if there are no levels inside the pack file
  ///
  fn from_file<P: AsRef<Path>>(json_pack_file: P) -> io::Result<Self> {
    let parent_folder = json_pack_file
      .as_ref()
      .parent()
      .map(Path::to_str)
      .flatten()
      .unwrap_or("")
      .to_string();

    // Parse the level as a JSON file
    let file = File::open(json_pack_file)?;
    let reader = BufReader::new(file);
    let mut me: Self = serde_json::from_reader(reader)?;

    // Set the parent folder from the path
    me.folder = parent_folder;

    // Remove any groups that have no levels
    me.groups.retain(|g| !g.0.is_empty());

    // Make sure there is at least one level in one group
    if me.groups.is_empty() {
      Err(io::Error::new(
        ErrorKind::InvalidData,
        "No levels provided in pack file".to_string(),
      ))?;
    }

    Ok(me)
  }
}

impl LevelGroup {
  pub fn main_levels(&self) -> &Vec<MainLevel> {
    &self.0
  }

  pub fn main_level(&self, index: usize) -> &MainLevel {
    &self.0[index]
  }

  pub fn is_complete(&self, global_state: &GlobalState) -> bool {
    self.0.iter().all(|l| global_state.is_level_complete(l.level.id))
  }

  pub fn len(&self) -> usize {
    self.0.iter().map(MainLevel::len).sum()
  }
}

impl MainLevel {
  pub fn level(&self) -> &Level {
    &self.level
  }

  pub fn challenge_levels(&self) -> &Vec<Level> {
    &self.challenge_levels
  }

  pub fn challenge_level(&self, index: usize) -> &Level {
    &self.challenge_levels[index]
  }

  pub fn len(&self) -> usize {
    1 + self.challenge_levels.len()
  }
}

impl Level {
  pub fn id(&self) -> Uuid {
    self.id
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn description(&self) -> &str {
    &self.description
  }

  pub fn level_type(&self) -> LevelType {
    self.r#type
  }

  #[allow(unused)]
  pub fn lua_file(&self) -> &str {
    &self.lua_file
  }

  pub fn get_title(&self, level_index: LevelIndex) -> String {
    if level_index.challenge.is_some() {
      format!("Challenge {} - {}", level_index, self.name())
    } else {
      format!("Level {} - {}", level_index, self.name())
    }
  }
}

impl LevelIndex {
  pub fn new(pack_index: usize, group: usize, level_in_group: usize) -> Self {
    Self {
      pack_index,
      group,
      level_in_group,
      challenge: None,
    }
  }

  pub fn new_challenge(pack_index: usize, group: usize, level_in_group: usize, challenge: usize) -> Self {
    Self {
      pack_index,
      group,
      level_in_group,
      challenge: Some(challenge),
    }
  }

  pub fn get_level_pack_index(&self) -> usize {
    self.pack_index
  }

  pub fn get_group(&self) -> usize {
    self.group
  }

  pub fn get_level_in_group(&self) -> usize {
    self.level_in_group
  }

  pub fn get_challenge(&self) -> Option<usize> {
    self.challenge
  }
}

impl fmt::Display for LevelIndex {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut counter = self.level_in_group;
    let level_index: String = iter::once(counter)
      .chain(iter::from_fn(|| {
        (counter >= 26).then(|| {
          counter /= 26;
          counter
        })
      }))
      .map(|c| ((c % 26) as u8 + b'A') as char)
      .collect::<String>()
      .chars()
      .rev()
      .collect();

    if let Some(challenge_index) = self.challenge {
      write!(f, "{}{}-{}", self.group + 1, level_index, challenge_index + 1)
    } else {
      write!(f, "{}{}", self.group + 1, level_index)
    }
  }
}

pub fn load_all_level_packs() -> io::Result<Vec<LevelPack>> {
  let level_packs: Vec<LevelPack> = fs::read_dir(LEVELS_FOLDER)?
    // Skip any errors from traversing the directory
    .filter_map(|entry| match entry {
      Ok(entry) => Some(entry),
      Err(e) => {
        println!("Failed to traverse \"{LEVELS_FOLDER}\" directory: {e}");
        None
      },
    })
    // Search all directories in the levels folder
    .filter(|entry| entry.file_type().map(|t| t.is_dir()).unwrap_or(false))
    // Make sure the directory has a pack JSON file
    .filter_map(|entry| {
      let mut pack_file_path = entry.path();
      pack_file_path.push(PACK_FILE);

      pack_file_path.exists().then_some(pack_file_path)
    })
    // Skip level packs that fail to load
    .filter_map(|pack_file_path| match LevelPack::from_file(&pack_file_path) {
      Ok(level_pack) => {
        println!("Loaded level pack: {}", pack_file_path.to_string_lossy());
        Some(level_pack)
      },
      Err(e) => {
        println!(
          "Failed to load level pack: {}\n  - Error: {e}",
          pack_file_path.to_string_lossy()
        );
        None
      },
    })
    .collect();

  // Make sure we loaded at least one level pack
  if level_packs.is_empty() {
    Err(io::Error::new(
      ErrorKind::InvalidData,
      format!("No valid level packs found in the \"{LEVELS_FOLDER}\" folder"),
    ))?;
  }

  Ok(level_packs)
}
