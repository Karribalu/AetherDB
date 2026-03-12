//! SQL query planner.
//!
//! Uses `sqlparser` to parse SQL into an AST, then maps the AST onto a
//! `LogicalPlan`. The `sqlparser` AST is never used as the execution model —
//! it is only ever an intermediate parse result.
//!
//! Supported initially:
//! - `SELECT ... FROM <namespace> WHERE ... ORDER BY ... LIMIT ...`
//! - Full-text predicate: `MATCH(field, 'query text')`
//! - Vector predicate: `ORDER BY vector_distance(field, [1.0, 0.5, ...]) LIMIT k`

// TODO (Week 23-24): implement SqlPlanner.
