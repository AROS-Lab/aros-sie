//! InMemoryStateStore — in-memory implementation of StateStore for testing and development.

use std::collections::HashMap;

use crate::error::SieError;

use super::StateStore;

/// In-memory implementation of the StateStore trait.
/// Used for testing and as a reference implementation.
/// Production use should be backed by SQLite/WAL in the kernel.
pub struct InMemoryStateStore {
    data: HashMap<String, Vec<u8>>,
}

impl InMemoryStateStore {
    /// Create a new empty store.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Get the number of entries in the store.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Default for InMemoryStateStore {
    fn default() -> Self {
        Self::new()
    }
}

impl StateStore for InMemoryStateStore {
    fn put(&mut self, key: &str, value: Vec<u8>) -> Result<(), SieError> {
        self.data.insert(key.to_string(), value);
        Ok(())
    }

    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, SieError> {
        Ok(self.data.get(key).cloned())
    }

    fn delete(&mut self, key: &str) -> Result<(), SieError> {
        self.data.remove(key);
        Ok(())
    }

    fn list_keys(&self, prefix: &str) -> Result<Vec<String>, SieError> {
        let keys: Vec<String> = self
            .data
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        Ok(keys)
    }

    fn exists(&self, key: &str) -> Result<bool, SieError> {
        Ok(self.data.contains_key(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_and_get() {
        let mut store = InMemoryStateStore::new();
        store.put("key1", b"value1".to_vec()).unwrap();
        let val = store.get("key1").unwrap().unwrap();
        assert_eq!(val, b"value1");
    }

    #[test]
    fn test_get_nonexistent() {
        let store = InMemoryStateStore::new();
        assert!(store.get("missing").unwrap().is_none());
    }

    #[test]
    fn test_delete() {
        let mut store = InMemoryStateStore::new();
        store.put("key1", b"value1".to_vec()).unwrap();
        store.delete("key1").unwrap();
        assert!(!store.exists("key1").unwrap());
    }

    #[test]
    fn test_list_keys_by_prefix() {
        let mut store = InMemoryStateStore::new();
        store.put("self_model/snapshot", b"a".to_vec()).unwrap();
        store.put("self_model/config", b"b".to_vec()).unwrap();
        store.put("policy/head", b"c".to_vec()).unwrap();

        let keys = store.list_keys("self_model/").unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.iter().all(|k| k.starts_with("self_model/")));
    }

    #[test]
    fn test_exists() {
        let mut store = InMemoryStateStore::new();
        assert!(!store.exists("key1").unwrap());
        store.put("key1", b"value1".to_vec()).unwrap();
        assert!(store.exists("key1").unwrap());
    }

    #[test]
    fn test_overwrite() {
        let mut store = InMemoryStateStore::new();
        store.put("key1", b"old".to_vec()).unwrap();
        store.put("key1", b"new".to_vec()).unwrap();
        let val = store.get("key1").unwrap().unwrap();
        assert_eq!(val, b"new");
    }
}
