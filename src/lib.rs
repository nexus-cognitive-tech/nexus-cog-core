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

// Flat re-exports so downstream crates can write `nexus_cog_core::TypeName`.
pub use antifragile::*;
pub use architect::*;
pub use causal::*;
pub use cognitive::*;
pub use common::*;
pub use diff::*;
pub use graph::*;
pub use hypothesis::*;
pub use intent::*;
pub use learner::*;
pub use memory::*;
pub use palace::*;
pub use patterns::*;
pub use predictor::*;
pub use provenance::*;
pub use risk::*;
pub use search::*;
pub use thought::*;
pub use verification::*;
