use std::{
    any::{Any, TypeId},
    collections::HashMap,
    hash::Hash,
    marker::PhantomData,
};

/// A map which supports keys with different types.
/// Keys must be `'static` and implement [`Eq`] and [`Hash`].
/// Values must be `'static`.
pub struct MultiTypeMap<T> {
    maps: HashMap<TypeId, Box<dyn Any>>,
    length: usize,
    _marker: PhantomData<T>,
}

impl<T: 'static> MultiTypeMap<T> {
    /// Creates an empty [`MultiTypeMap`].
    pub fn new() -> Self {
        Self {
            maps: HashMap::new(),
            length: 0,
            _marker: PhantomData,
        }
    }

    /// Inserts a value into the map. If the map did not have this key present, `None` is returned.
    /// If the map did have this key present, the value is updated, and the old value is returned.
    pub fn insert<K: 'static + Eq + Hash>(&mut self, key: K, value: T) -> Option<T> {
        self.length += 1;
        self.map_mut().insert(key, value).map(|value| {
            self.length -= 1;
            value
        })
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in the map.
    pub fn remove<K: 'static + Eq + Hash>(&mut self, key: &K) -> Option<T> {
        self.map_mut::<K>().remove(key).map(|value| {
            self.length -= 1;
            value
        })
    }

    /// Gets an immutable reference to the value corresponding to the given key.
    pub fn get<K: 'static + Eq + Hash>(&self, key: &K) -> Option<&T> {
        self.map::<K>().and_then(|map| map.get(key))
    }

    /// Gets a mutable reference to the value corresponding to the given key.
    pub fn get_mut<K: 'static + Eq + Hash>(&mut self, key: &K) -> Option<&mut T> {
        self.map_mut::<K>().get_mut(key)
    }

    /// Gets an immutable reference to the map for the given key type.
    fn map<K: 'static>(&self) -> Option<&HashMap<K, T>> {
        self.maps.get(&TypeId::of::<K>()).map(|map| {
            map.downcast_ref::<HashMap<K, T>>()
                .expect("two different types should not have the same TypeId")
        })
    }

    /// Gets a mutable reference to the map corresponding to the given key.
    fn map_mut<K: 'static>(&mut self) -> &mut HashMap<K, T> {
        self.maps
            .entry(TypeId::of::<K>())
            .or_insert_with(|| Box::<HashMap<K, T>>::default())
            .downcast_mut()
            .expect("two different types should not have the same TypeId")
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}

impl<T: 'static> Default for MultiTypeMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_remove() {
        let mut map = MultiTypeMap::new();

        map.insert(false, 2);
        map.insert(1, 1);
        map.insert("hey", 3);

        assert_eq!(map.remove(&false), Some(2));
        assert_eq!(map.remove(&1), Some(1));
        assert_eq!(map.remove(&"hey"), Some(3));
    }

    #[test]
    fn test_get_mut() {
        let mut map = MultiTypeMap::new();

        map.insert(false, 0);
        map.insert("hey", 3);

        assert_eq!(map.get_mut(&false), Some(&mut 0));
        map.get_mut(&false).map(|v| *v = 1);
        assert_eq!(map.get_mut(&false), Some(&mut 1));

        assert_eq!(map.get_mut(&"hey"), Some(&mut 3));
        map.get_mut(&"hey").map(|v| *v = 4);
        assert_eq!(map.get_mut(&"hey"), Some(&mut 4));
    }

    #[test]
    fn test_strings() {
        let mut map = MultiTypeMap::new();

        // First we insert a `&str` key.
        assert_eq!(map.insert("foo", 1), None);
        assert_eq!(map.insert("foo", 2), Some(1));

        // When we insert a `String` key with the 'same' value, it's a different type, so it's a different entry.
        assert_eq!(map.insert("foo".to_owned(), 3), None);
        assert_eq!(map.insert("foo".to_owned(), 4), Some(3));

        // We can still get the `&str` key.
        assert_eq!(map.get(&"foo"), Some(&2));
        assert_eq!(map.get(&"foo".to_owned()), Some(&4));
    }
}
