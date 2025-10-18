use core::borrow::Borrow;

const CAPACITY: usize = 32;

/// A simple array-based map implementation.
#[derive(Clone, Copy)]
pub struct ArrayMap<K, V> {
    data: [(K, V); CAPACITY],
    size: usize,
}

impl<K, V> ArrayMap<K, V>
where
    K: PartialEq + Copy + Default,
    V: Copy + Default,
{
    /// Creates a new empty ArrayMap.
    pub fn new() -> Self {
        ArrayMap {
            data: [(K::default(), V::default()); CAPACITY],
            size: 0,
        }
    }

    fn find_index<Q: ?Sized>(&self, key: &Q) -> Option<usize>
    where
        K: Borrow<Q>,
        Q: PartialEq,
    {
        for i in 0..self.size {
            if key == self.data[i].0.borrow() {
                return Some(i);
            }
        }
        None
    }

    /// Inserts a new key-value pair into the map.
    ///
    /// If the key already exists in the map, the old value is returned.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if let Some(index) = self.find_index(&key) {
            let old_value = self.data[index].1;
            self.data[index].1 = value;
            Some(old_value)
        } else {
            if self.size < CAPACITY {
                self.data[self.size] = (key, value);
                self.size += 1;
                None
            } else {
                None
            }
        }
    }

    /// Retrieves a value from the map by key.
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: PartialEq,
    {
        self.find_index(key).map(|i| &self.data[i].1)
    }

    /// Retrieves a mutable value from the map by key.
    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: PartialEq,
    {
        self.find_index(key).map(|i| &mut self.data[i].1)
    }
}
