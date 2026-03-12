//! Logical plan types shared by the SQL and JSON query planners.
//!
//! A `LogicalPlan` is an abstract representation of a query before it is
//! bound to any physical segment or index. The `engine` module takes a
//! `LogicalPlan` and executes it against the current set of segments.

// TODO (Week 19): define LogicalPlan, Predicate, Projection, OrderBy, Limit.
