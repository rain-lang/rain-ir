/*!
Graph-theoretic utilities for `rain`
*/

pub mod bfs;
pub mod ndfs;
pub mod dfs;

/// A wrapper for a reference
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Borrowed<'a, V>(pub &'a V);