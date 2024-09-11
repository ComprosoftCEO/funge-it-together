use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;
use uuid::Uuid;

use crate::isa::{self, SolutionManager};
use crate::level::{Level, LevelIndex, LevelPack};
use crate::statistics::Statistics;

static SAVE_FILE: &str = "save.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalState {
  #[serde(default, deserialize_with = "deserialize_one_or_many_map")]
  solutions: HashMap<Uuid, Vec<isa::standard::Solution>>,
  #[serde(default)]
  parallel_solutions: HashMap<Uuid, Vec<isa::parallel::Solution>>,
  #[serde(default)]
  unlocked: HashMap<Uuid, Statistics>,

  #[serde(skip)]
  level_packs: Vec<LevelPack>,
}

impl GlobalState {
  pub fn load(level_packs: Vec<LevelPack>) -> Self {
    let mut state = Self::from_file(SAVE_FILE).unwrap_or_default();
    state.level_packs = level_packs;
    state
  }

  fn from_file<P: AsRef<Path>>(json_save_file: P) -> io::Result<Self> {
    // Parse the level as a JSON file
    let file = File::open(json_save_file)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
  }

  pub fn save(&self) -> io::Result<()> {
    let file = File::create(SAVE_FILE)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, self)?;
    Ok(())
  }

  #[inline]
  pub fn num_level_packs(&self) -> usize {
    self.level_packs.len()
  }

  #[inline]
  pub fn get_level_pack(&self, pack_index: usize) -> &LevelPack {
    &self.level_packs[pack_index]
  }

  pub fn level(&self, index: LevelIndex) -> &Level {
    let main_level = self.level_packs[index.get_level_pack_index()]
      .level_group(index.get_group())
      .main_level(index.get_level_in_group());
    if let Some(challenge) = index.get_challenge() {
      main_level.challenge_level(challenge)
    } else {
      main_level.level()
    }
  }

  pub fn is_level_complete(&self, level_id: Uuid) -> bool {
    self.unlocked.contains_key(&level_id)
  }

  pub fn get_statistics(&self, level_id: Uuid) -> Option<Statistics> {
    self.unlocked.get(&level_id).cloned()
  }

  // Returns the best statistics overall
  pub fn complete_level(&mut self, level_id: Uuid, statistics: Statistics) -> Statistics {
    let best = self
      .unlocked
      .entry(level_id)
      .and_modify(|s| s.set_to_best(&statistics))
      .or_insert(statistics)
      .clone();

    // A level complete is very important, so ALWAYS try to save right away!
    // Silently ignore any errors (hopefully won't happen in practice)
    self.save().ok();

    best
  }
}

impl SolutionManager<isa::Standard> for GlobalState {
  fn get_all_solutions(&self, level_id: Uuid) -> &Vec<isa::standard::Solution> {
    static EMPTY_LIST: Vec<isa::standard::Solution> = Vec::new();
    self.solutions.get(&level_id).unwrap_or(&EMPTY_LIST)
  }

  fn get_all_solutions_mut(&mut self, level_id: Uuid) -> &mut Vec<isa::standard::Solution> {
    self.solutions.entry(level_id).or_default()
  }
}

impl SolutionManager<isa::Parallel> for GlobalState {
  fn get_all_solutions(&self, level_id: Uuid) -> &Vec<isa::parallel::Solution> {
    static EMPTY_LIST: Vec<isa::parallel::Solution> = Vec::new();
    self.parallel_solutions.get(&level_id).unwrap_or(&EMPTY_LIST)
  }

  fn get_all_solutions_mut(&mut self, level_id: Uuid) -> &mut Vec<isa::parallel::Solution> {
    self.parallel_solutions.entry(level_id).or_default()
  }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
enum OneOrManyMap<K, V>
where
  K: Eq + Hash,
{
  One(HashMap<K, V>),
  Vec(HashMap<K, Vec<V>>),
}

impl<K, V> From<OneOrManyMap<K, V>> for HashMap<K, Vec<V>>
where
  K: Eq + Hash,
{
  fn from(from: OneOrManyMap<K, V>) -> Self {
    match from {
      OneOrManyMap::One(val) => val.into_iter().map(|(k, v)| (k, vec![v])).collect(),
      OneOrManyMap::Vec(vec) => vec,
    }
  }
}

fn deserialize_one_or_many_map<'de, D, K, V>(deserializer: D) -> Result<HashMap<K, Vec<V>>, D::Error>
where
  D: Deserializer<'de>,
  K: Eq + Hash + Deserialize<'de>,
  V: Deserialize<'de>,
{
  Ok(OneOrManyMap::<K, V>::deserialize(deserializer)?.into())
}
