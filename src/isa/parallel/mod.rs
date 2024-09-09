use crate::isa::InstructionSetArchitecture;

mod solution;

pub use solution::Solution;

/// Parallel level type
pub struct Parallel;

impl InstructionSetArchitecture for Parallel {
  type Solution = Solution;
}
