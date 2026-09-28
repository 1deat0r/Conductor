//! Durable supervisor session receipts.
//!
//! This store does not spawn processes. A future process owner may build on
//! these receipts only after it has an independently enforced launch and
//! process-tree ownership boundary.

mod session_store;

pub use session_store::{SessionError, SessionReceipt, SessionState, SessionStore, StartReceipt};
