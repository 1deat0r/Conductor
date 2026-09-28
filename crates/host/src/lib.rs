//! Local host-side durable command journal.
//!
//! This library is not exposed by the S0 control API or renderer. A persisted
//! receipt records storage progress; it does not authorize an external effect.

mod journal;

pub use journal::{
    CommandEnvelope, CommandJournal, CommandOutboxEvent, CommandReceipt, CommandState,
    HostRejection, JournalError,
};
