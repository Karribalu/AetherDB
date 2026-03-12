//! Core primitives: error types, configuration, shared constants.
//!
//! Every other module may depend on `core`. `core` depends on nothing
//! inside AetherDB — only on the standard library and external crates.

pub mod config;
pub mod error;

pub use error::{AetherError, Result};
