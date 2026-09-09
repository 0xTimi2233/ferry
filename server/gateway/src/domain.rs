//! 领域层

pub mod aggregates;
pub mod errors;
pub mod events;
pub mod values;

#[cfg(test)]
mod tests;

pub use aggregates::*;
pub use errors::*;
pub use events::*;
pub use values::*;
