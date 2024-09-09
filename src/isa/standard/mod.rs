use crate::isa::InstructionSetArchitecture;

mod puzzle;
mod solution;
mod vm;

pub use solution::Solution;

/// Standard level type
pub struct Standard;

impl InstructionSetArchitecture for Standard {
  type Solution = Solution;
}
