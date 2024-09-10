use crate::{level::LevelIndex, state::State};
use std::error::Error;
use uuid::Uuid;

pub mod parallel;
pub mod standard;

// Re-export the puzzle types
pub use parallel::Parallel;
pub use standard::Standard;

pub const MAX_SOLUTION_NAME_LEN: usize = 30;
static COPY_STR: &str = " (Copy)";

/// All level types need to implement this interface
pub trait InstructionSetArchitecture {
  type Solution: Solution;
  type Puzzle;

  fn generate_test_cases(
    folder: &str,
    lua_file: &str,
    seed: u32,
    n: usize,
  ) -> Result<Vec<Self::Puzzle>, Box<dyn Error>>;

  fn open_editor(
    level_index: LevelIndex,
    solution_index: usize,
    solution: Self::Solution,
    test_cases: Vec<Self::Puzzle>,
    test_case_index: usize,
  ) -> impl State;
}

/// Any solution type should implement this interface
pub trait Solution: Clone {
  fn new(name: impl Into<String>) -> Self;

  fn name(&self) -> &str;
  fn rename(&mut self, new_name: impl Into<String>);

  fn symbols_used(&self) -> usize;
}

/// Manages solutions for a level type
pub trait SolutionManager<ISA: InstructionSetArchitecture> {
  fn get_all_solutions(&self, level_id: Uuid) -> &Vec<ISA::Solution>;
  fn get_all_solutions_mut(&mut self, level_id: Uuid) -> &mut Vec<ISA::Solution>;

  // -------- Default Implementation: --------

  /// Return the index of the new solution
  fn new_solution(&mut self, level_id: Uuid) -> usize {
    let all_solutions = self.get_all_solutions_mut(level_id);
    let new_solution_name = format!("Solution {}", all_solutions.len() + 1);
    all_solutions.push(ISA::Solution::new(new_solution_name));
    all_solutions.len() - 1
  }

  fn save_solution(&mut self, level_id: Uuid, solution_index: usize, solution: ISA::Solution) {
    let all_solutions = self.get_all_solutions_mut(level_id);
    all_solutions[solution_index] = solution;
  }

  fn rename_solution(&mut self, level_id: Uuid, solution_index: usize, new_name: String) {
    let all_solutions = self.get_all_solutions_mut(level_id);
    all_solutions[solution_index].rename(new_name);
  }

  /// Returns the index of the new solution
  fn copy_solution(&mut self, level_id: Uuid, solution_index: usize) -> usize {
    let all_solutions = self.get_all_solutions_mut(level_id);

    let mut new_solution = all_solutions[solution_index].clone();

    // Handle making a solution copy. Some examples:
    //  - "Solution Name"        => "Solution Name (Copy)"
    //  - "Solution Name (Copy)" => "Solution Name (Copy) (Copy)"
    //
    // If the solution name + (Copy) is too long, then remove the trailing copy
    //  - "Too ... Long (Copy)" => "Too ... Long (Copy)"
    fn remove_copy_suffix(s: &str) -> &str {
      if s.len() <= MAX_SOLUTION_NAME_LEN {
        return s;
      }

      match s.strip_suffix(COPY_STR) {
        Some(s) => s,
        None => s,
      }
    }

    let new_solution_name = format!("{}{}", remove_copy_suffix(new_solution.name()), COPY_STR);
    new_solution.rename(new_solution_name);

    all_solutions.insert(solution_index + 1, new_solution);
    solution_index + 1
  }

  fn swap_solutions_order(&mut self, level_id: Uuid, index_one: usize, index_two: usize) {
    let all_solutions = self.get_all_solutions_mut(level_id);
    all_solutions.swap(index_one, index_two);
  }

  fn delete_solution(&mut self, level_id: Uuid, solution_index: usize) {
    let all_solutions = self.get_all_solutions_mut(level_id);
    all_solutions.remove(solution_index);
  }
}
