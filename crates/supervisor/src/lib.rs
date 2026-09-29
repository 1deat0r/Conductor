//! Internal durable supervisor session-state foundation.
//!
//! This crate exposes no session API to the S0 binary and does not spawn or
//! own operating-system processes.

#[allow(dead_code)]
mod session_store;
