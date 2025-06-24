//! Asset caching system

use indexmap::IndexMap;
use std::collections::HashMap;
use std::hash::Hash;

/// Generic asset cache with LRU eviction
pub struct AssetCache<T> {
    cache: IndexMap<String, T>,
    max_size: usize,
}

impl<T> AssetCache<T> {
    pub fn new() -> Self {
        Self::with_capacity(1000) // Default to 1000 items
    }

    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            cache: IndexMap::with_capacity(max_size),
            max_size,
        }
    }

    /// Insert an asset into the cache
    pub fn insert(&mut self, key: String, value: T) -> &T {
        // If we're at capacity, remove the least recently used item
        if self.cache.len() >= self.max_size && !self.cache.contains_key(&key) {
            if let Some((first_key, _)) = self.cache.first() {
                let first_key = first_key.clone();
                self.cache.shift_remove(&first_key);
            }
        }

        // Insert or update the item (moves it to the end)
        self.cache.insert(key.clone(), value);
        self.cache.get(&key).unwrap()
    }

    /// Get an asset from the cache
    pub fn get(&mut self, key: &str) -> Option<&T> {
        // Move the accessed item to the end (most recently used)
        if self.cache.contains_key(key) {
            let key = key.to_string();
            self.cache
                .move_index(self.cache.get_index_of(&key)?, self.cache.len() - 1);
            self.cache.get(&key)
        } else {
            None
        }
    }

    /// Get an immutable reference without updating LRU
    pub fn peek(&self, key: &str) -> Option<&T> {
        self.cache.get(key)
    }

    /// Check if the cache contains a key
    pub fn contains_key(&self, key: &str) -> bool {
        self.cache.contains_key(key)
    }

    /// Remove an item from the cache
    pub fn remove(&mut self, key: &str) -> Option<T> {
        self.cache.shift_remove(key)
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Get the current size of the cache
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Get all keys in the cache
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.cache.keys()
    }
}

impl<T> Default for AssetCache<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Specialized cache for assets with handles
pub struct HandleCache<K: Eq + Hash, V> {
    cache: HashMap<K, V>,
}

impl<K: Eq + Hash, V> HandleCache<K, V> {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.cache.insert(key, value);
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.cache.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.cache.get_mut(key)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.cache.remove(key)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.cache.contains_key(key)
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

impl<K: Eq + Hash, V> Default for HandleCache<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
