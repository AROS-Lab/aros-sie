//! Persistence layer — state store abstraction and event sourcing.
//!
//! Provides a trait-based state store that the kernel can implement
//! with SQLite/WAL, while the SIE ships with an in-memory implementation.

pub mod events;
pub mod store;

use crate::error::SieError;

/// Trait for the SIE state store.
///
/// Keys are namespaced strings (e.g., "self_model/snapshot", "policy/head").
/// Values are opaque byte vectors (typically JSON-serialized).
pub trait StateStore: Send + Sync {
    /// Store a value under the given key.
    fn put(&mut self, key: &str, value: Vec<u8>) -> Result<(), SieError>;

    /// Retrieve a value by key.
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, SieError>;

    /// Delete a value by key.
    fn delete(&mut self, key: &str) -> Result<(), SieError>;

    /// List all keys matching the given prefix.
    fn list_keys(&self, prefix: &str) -> Result<Vec<String>, SieError>;

    /// Check if a key exists.
    fn exists(&self, key: &str) -> Result<bool, SieError>;
}
