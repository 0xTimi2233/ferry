//! 领域层

pub mod aggregates;
pub mod canonical;
pub mod errors;
pub mod events;
pub mod values;

#[cfg(test)]
mod tests;

pub use aggregates::*;
pub use canonical::*;
pub use errors::*;
pub use events::*;
pub use values::*;
