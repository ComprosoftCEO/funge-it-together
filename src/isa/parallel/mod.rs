use crate::isa::InstructionSetArchitecture;

mod editor_state;
mod execute_state;
mod puzzle;
mod solution;
mod vm;

pub use solution::Solution;

/// Parallel level type
pub struct Parallel;

impl InstructionSetArchitecture for Parallel {
  type Solution = Solution;
  type Puzzle = puzzle::Puzzle;
}
