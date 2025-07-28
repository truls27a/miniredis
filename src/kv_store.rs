use std::{collections::HashMap, sync::{Arc, Mutex, MutexGuard}};

/// A key-value store that can be shared between threads.
/// 
/// KVStore is a thread-safe key-value store that can be used to store and retrieve data between threads.
/// It includes a set of methods to get, set, and delete key-value pairs.
/// 
/// # Examples
/// 
/// ```rust
/// use kv_store::KVStore;
/// 
/// let store = KVStore::new();
/// store.set("key", "value");
/// let value = store.get("key");
/// assert_eq!(value, Some("value"));
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
    /// use kv_store::KVStore;
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
    /// # Panics
    /// 
    /// Panics if the store is already locked.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use kv_store::KVStore;
    /// 
    /// let store = KVStore::new();
    /// store.set("key", "value");
    /// let value = store.get("key");
    /// assert_eq!(value, Some("value"));
    /// ```
    pub fn get(&self, key: &str) -> Option<String> {
        let store = self.get_store();
        store.get(key).cloned()
    }

    /// Sets a value in the store.
    /// 
    /// # Arguments
    /// 
    /// * `key` - The key to set the value for.
    /// * `value` - The value to set.
    /// 
    /// # Panics
    /// 
    /// Panics if the store is already locked
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use kv_store::KVStore;
    /// 
    /// let store = KVStore::new();
    /// store.set("key", "value");
    /// let value = store.get("key");
    /// assert_eq!(value, Some("value"));
    /// ```
    pub fn set(&self, key: &str, value: &str) {
        let mut store = self.get_store();
        store.insert(key.to_string(), value.to_string());
    }

    /// Deletes a value from the store.
    /// 
    /// # Arguments
    /// 
    /// * `key` - The key to delete the value for.
    /// 
    /// # Panics
    /// 
    /// Panics if the store is already locked.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use kv_store::KVStore;
    /// 
    /// let store = KVStore::new();
    /// store.set("key", "value");
    /// store.delete("key");
    /// let value = store.get("key");
    /// assert_eq!(value, None);
    /// ```
    pub fn delete(&self, key: &str) {
        let mut store = self.get_store();
        store.remove(key);
    }

    /// Gets a mutable reference to the store.
    /// 
    /// # Returns
    /// 
    /// A mutable reference to the store.
    /// 
    /// # Panics
    /// 
    /// Panics if the store is already locked.
    fn get_store(&self) -> MutexGuard<HashMap<String, String>> {
        self.store.lock().unwrap()
    }
}