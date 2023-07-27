use std::collections::{HashMap, LinkedList};
use std::hash::Hash;

pub mod killer;
pub mod transposition;

struct HashTable<K: Eq + Hash + Copy, V> {
    capacity: usize,
    entry_list: LinkedList<K>,
    entry_map: HashMap<K, V>,
}

impl<K: Eq + Hash + Copy, V> HashTable<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self { capacity, entry_list: LinkedList::new(), entry_map: HashMap::with_capacity(capacity) }
    }

    fn clear(&mut self) {
        self.entry_list.clear();
        self.entry_map.clear();
    }

    fn put(&mut self, key: K, value: V) {
        if self.entry_map.insert(key, value).is_none() {
            self.entry_list.push_back(key);
        }
        if self.entry_map.len() > self.capacity {
            let remove_key = self.entry_list.pop_front().unwrap();
            self.entry_map.remove(&remove_key);
        }
    }

    fn get(&self, key: K) -> Option<&V> {
        self.entry_map.get(&key)
    }

    fn len(&self) -> usize {
        self.entry_map.len()
    }

    fn load_factor(&self) -> f32 {
        self.len() as f32 / self.capacity as f32
    }
}