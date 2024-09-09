use crate::isa::InstructionSetArchitecture;

mod solution;

pub use solution::Solution;

/// Standard level type
pub struct Standard;

impl InstructionSetArchitecture for Standard {
  type Solution = Solution;
}
