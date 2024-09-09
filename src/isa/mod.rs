pub mod parallel;
pub mod standard;

// Re-export the puzzle types
pub use parallel::Parallel;
pub use standard::Standard;

/// All level types need to implement this interface
pub trait InstructionSetArchitecture {
  type Solution;
}
