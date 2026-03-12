//! Query layer: logical plan types, SQL planner, and JSON query planner.
//!
//! Boundary rules:
//! - This module does not touch storage or index structures directly.
//! - It produces a `LogicalPlan` consumed by the `engine` module.
//! - Both SQL and JSON surfaces map onto the same `LogicalPlan` type.

pub mod json;
pub mod plan;
pub mod sql;
