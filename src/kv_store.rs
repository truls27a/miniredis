use std::{collections::HashMap, sync::{Arc, Mutex, MutexGuard}};

use crate::error::MiniRedisError;

/// A key-value store that can be shared between threads.
/// 
/// KVStore is a thread-safe key-value store that can be used to store and retrieve data between threads.
/// It includes a set of methods to get, set, and delete key-value pairs.
/// 
/// # Examples
/// 
/// ```rust
/// use miniredis::kv_store::KVStore;
/// 
/// let store = KVStore::new();
/// store.set("key", "value");
/// let value = store.get("key");
/// assert_eq!(value, Some("value".to_string()));
/// ```
pub struct KVStore {
    store: Arc<Mutex<HashMap<String, String>>>,
}

impl KVStore {
    /// Creates a new KVStore.
    /// 
    /// # Returns
    /// 
    /// A new KVStore.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use miniredis::kv_store::KVStore;
    /// 
    /// let store = KVStore::new();
    /// ```
    pub fn new() -> Self {
        Self { store: Arc::new(Mutex::new(HashMap::new())) }
    }

    /// Gets a value from the store.
    /// 
    /// # Arguments
    /// 
    /// * `key` - The key to get the value for.
    /// 
    /// # Returns
    /// 
    /// The value associated with the key, or None if the key is not found.
    /// 
    /// # Errors
    /// 
    /// If the store is already locked, it will return an error.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use miniredis::kv_store::KVStore;
    /// 
    /// let store = KVStore::new();
    /// store.set("key", "value");
    /// let value = store.get("key");
    /// assert_eq!(value, Some("value".to_string()));
    /// ```
    pub fn get(&self, key: &str) -> Result<Option<String>, MiniRedisError> {
        let store = self.get_store()?;
        Ok(store.get(key).cloned())
    }

    /// Sets a value in the store.
    /// 
    /// # Arguments
    /// 
    /// * `key` - The key to set the value for.
    /// * `value` - The value to set.
    /// 
    /// # Errors
    /// 
    /// If the store is already locked, it will return an error.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use miniredis::kv_store::KVStore;
    /// 
    /// let store = KVStore::new();
    /// store.set("key", "value");
    /// let value = store.get("key");
    /// assert_eq!(value, Some("value".to_string()));
    /// ```
    pub fn set(&self, key: &str, value: &str) -> Result<(), MiniRedisError> {
        let mut store = self.get_store()?;
        store.insert(key.to_string(), value.to_string());
        Ok(())
    }

    /// Deletes a value from the store.
    /// 
    /// # Arguments
    /// 
    /// * `key` - The key to delete the value for.
    /// 
    /// # Errors
    /// 
    /// If the store is already locked, it will return an error.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use miniredis::kv_store::KVStore;
    /// 
    /// let store = KVStore::new();
    /// store.set("key", "value");
    /// store.del("key");
    /// let value = store.get("key");
    /// assert_eq!(value, None);
    /// ```
    pub fn del(&self, key: &str) -> Result<(), MiniRedisError> {
        let mut store = self.get_store()?;
        store.remove(key);
        Ok(())
    }

    /// Gets a mutable reference to the store.
    /// 
    /// # Returns
    /// 
    /// A mutable reference to the store.
    /// 
    /// # Errors
    /// 
    /// If the store is already locked, it will return an error.
    fn get_store(&self) -> Result<MutexGuard<HashMap<String, String>>, MiniRedisError> {
        self.store.lock().map_err(|_| MiniRedisError::StoreLocked)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_empty_store() {
        let store = KVStore::new();
        assert_eq!(None, store.get("key").unwrap());
    }

    #[test]
    fn get_returns_value_if_set() {
        let store = KVStore::new();
        store.set("key", "value").unwrap();
        assert_eq!(Some("value".to_string()), store.get("key").unwrap());
    }

    #[test]
    fn get_returns_none_if_not_set() {
        let store = KVStore::new();
        assert_eq!(None, store.get("key").unwrap());
    }

    #[test]
    fn get_returns_none_if_not_set_and_other_key_is_set() {
        let store = KVStore::new();
        store.set("key", "value").unwrap();
        assert_eq!(None, store.get("other_key").unwrap());
    }

    #[test]
    fn get_returns_value_if_set_and_other_key_is_set() {
        let store = KVStore::new();
        store.set("key", "value").unwrap();
        store.set("other_key", "other_value").unwrap();
        assert_eq!(Some("value".to_string()), store.get("key").unwrap());
    }

    #[test]
    fn get_returns_none_if_deleted() {
        let store = KVStore::new();
        store.set("key", "value").unwrap();
        store.del("key").unwrap();
        assert_eq!(None, store.get("key").unwrap());
    }

    #[test]
    fn set_sets_value() {
        let store = KVStore::new();
        store.set("key", "value").unwrap();
        assert_eq!(Some("value".to_string()), store.get("key").unwrap());
    }

    #[test]
    fn delete_deletes_value() {
        let store = KVStore::new();
        store.set("key", "value").unwrap();
        store.del("key").unwrap();
        assert_eq!(None, store.get("key").unwrap());
    }

    #[test]
    fn delete_does_nothing_if_key_not_set() {
        let store = KVStore::new();
        store.del("key").unwrap();
        assert_eq!(None, store.get("key").unwrap());
    }

    #[test]
    fn delete_does_nothing_if_key_not_set_and_other_key_is_set() {
        let store = KVStore::new();
        store.set("other_key", "other_value").unwrap();
        store.del("key").unwrap();
        assert_eq!(None, store.get("key").unwrap());
    }
}