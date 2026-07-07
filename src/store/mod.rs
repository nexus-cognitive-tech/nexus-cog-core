//! Storage primitives: atomic writes, file locks, schema migrations,
//! and transactional batch updates.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod atomic;
pub mod batch;
pub mod error;
pub mod json_store;
pub mod lock;
pub mod migrations;

pub use atomic::atomic_write;
pub use batch::{BatchReport, BatchWriter};
pub use error::{StoreError, StoreResult};
pub use json_store::{JsonStore, StoreConfig};
pub use lock::FileLock;
pub use migrations::{Migration, MigrationEngine};
