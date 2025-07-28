use std::{collections::HashMap, sync::{Arc, Mutex}};

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

    // TODO: Add methods to get, set, and delete key-value pairs.
}