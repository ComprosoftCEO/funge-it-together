use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
  average_cycles: f64,
  symbols_used: usize,
}

impl Statistics {
  pub fn new(average_cycles: f64, symbols_used: usize) -> Self {
    Self {
      average_cycles,
      symbols_used,
    }
  }

  pub fn average_cycles(&self) -> f64 {
    self.average_cycles
  }

  pub fn symbols_used(&self) -> usize {
    self.symbols_used
  }

  pub fn set_to_best(&mut self, other: &Statistics) {
    if other.average_cycles < self.average_cycles {
      self.average_cycles = other.average_cycles;
    }
    if other.symbols_used < self.symbols_used {
      self.symbols_used = other.symbols_used;
    }
  }
}
