//! Nexus Cog core — shared types, storage abstractions and runtime configuration.
//!
//! Every Nexus Cog engine and interface builds on this crate. It carries no
//! business logic — only data shapes, serde-friendly representations, store
//! primitives, and runtime configuration.

pub mod antifragile;
pub mod architect;
pub mod causal;
pub mod cognitive;
pub mod common;
pub mod config;
pub mod diff;
pub mod graph;
pub mod hypothesis;
pub mod intent;
pub mod learner;
pub mod memory;
pub mod palace;
pub mod patterns;
pub mod predictor;
pub mod provenance;
pub mod risk;
pub mod search;
pub mod store;
pub mod thought;
pub mod verification;

pub use chrono;
pub use indexmap;
pub use serde;
pub use serde_json;
pub use uuid;
