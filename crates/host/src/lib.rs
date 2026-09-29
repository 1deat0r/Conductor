//! Internal local command persistence foundation.
//!
//! The store is deliberately not part of the crate's public API and is not
//! connected to an S0 route. It does not produce host receipts or execute work.

#[allow(dead_code)]
mod journal;
