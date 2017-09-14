use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::hash::{BuildHasherDefault, Hasher};
use std::ptr;

use unsafe_any::{UnsafeAny, UnsafeAnyExt};

#[derive(Default)]
pub struct TypeIdHasher {
    value: u64,
}

impl Hasher for TypeIdHasher {
    fn finish(&self) -> u64 {
        self.value
    }

    fn write(&mut self, bytes: &[u8]) {
        // This expects to receive one and exactly one 64-bit value
        debug_assert!(bytes.len() == 8);
        unsafe {
            ptr::copy_nonoverlapping(&bytes[0] as *const u8 as *const u64, &mut self.value, 1)
        }
    }
}

#[doc(hidden)]
pub unsafe trait Implements<A: ?Sized + UnsafeAnyExt> {
    fn into_object(self) -> Box<A>;
}

unsafe impl<T: UnsafeAny> Implements<UnsafeAny> for T {
    fn into_object(self) -> Box<UnsafeAny> {
        Box::new(self)
    }
}

unsafe impl<T: UnsafeAny + Send + Sync> Implements<(UnsafeAny + Send + Sync)> for T {
    fn into_object(self) -> Box<UnsafeAny + Send + Sync> {
        Box::new(self)
    }
}

#[derive(Default, Debug)]
pub struct TypeMap<A: ?Sized = UnsafeAny>
where
    A: UnsafeAnyExt,
{
    data: HashMap<TypeId, Box<A>, BuildHasherDefault<TypeIdHasher>>,
}

/// This trait defines the relationship between keys and values in a `TypeMap`.
///
/// It is implemented for Keys, with a phantom associated type for the values.
pub trait Key: Any {
    /// The value type associated with this key type.
    type Value: Any;
}

impl TypeMap {
    /// Create a new, empty TypeMap.
    pub fn new() -> TypeMap {
        TypeMap::custom()
    }
}

impl<A: UnsafeAnyExt + ?Sized> TypeMap<A> {
    /// Create a new, empty TypeMap.
    ///
    /// Can be used with any `A` parameter; `new` is specialized to get around
    /// the required type annotations when using this function.
    pub fn custom() -> TypeMap<A> {
        TypeMap {
            data: HashMap::default(),
        }
    }

    /// Insert a value into the map with a specified key type.
    pub fn put<K: Key>(&mut self, val: K::Value) -> Option<K::Value>
    where
        K::Value: Any + Implements<A>,
    {
        self.data
            .insert(TypeId::of::<K>(), val.into_object())
            .map(|v| unsafe { *v.downcast_unchecked::<K::Value>() })
    }

    /// Gets a value from the type map.
    ///
    /// # Panics
    ///
    /// If there is no value in the request state of the given type.
    pub fn get<K: Key>(&self) -> &K::Value
    where
        K::Value: Any + Implements<A>,
    {
        // FIXME: Don't use unwrap and explain the error
        self.try_get::<K>().unwrap()
    }

    /// Attempt to get a value from the type map.
    pub fn try_get<K: Key>(&self) -> Option<&K::Value>
    where
        K::Value: Any + Implements<A>,
    {
        self.data
            .get(&TypeId::of::<K>())
            .map(|v| unsafe { v.downcast_ref_unchecked::<K::Value>() })
    }

    /// Check if a key has an associated value stored in the map.
    pub fn has<K: Key>(&self) -> bool {
        self.data.contains_key(&TypeId::of::<K>())
    }
}

pub type ShareMap = TypeMap<UnsafeAny + Sync + Send>;

#[cfg(test)]
mod tests {
    use std::mem;
    use std::hash::{Hash, Hasher};

    use super::{Key, TypeId, TypeIdHasher, TypeMap};

    #[derive(Debug, PartialEq)]
    struct KeyType;

    #[derive(Clone, Debug, PartialEq)]
    struct ValueType(u8);

    impl Key for KeyType {
        type Value = ValueType;
    }

    #[test]
    fn test_pair_key_to_value() {
        let mut map = TypeMap::new();
        map.put::<KeyType>(ValueType(32));

        assert_eq!(map.get::<KeyType>(), &ValueType(32));
        assert!(map.has::<KeyType>());
    }

    #[test]
    fn test_type_id_hasher() {
        fn verify_hashing_with(type_id: TypeId) {
            let mut hasher = TypeIdHasher::default();
            type_id.hash(&mut hasher);

            assert_eq!(hasher.finish(), unsafe {
                mem::transmute::<TypeId, u64>(type_id)
            });
        }

        // Pick a variety of types, just to demonstrate it’s all sane.
        // Normal, zero-sized, unsized, &c.
        verify_hashing_with(TypeId::of::<usize>());
        verify_hashing_with(TypeId::of::<()>());
        verify_hashing_with(TypeId::of::<str>());
        verify_hashing_with(TypeId::of::<&str>());
        verify_hashing_with(TypeId::of::<Vec<u8>>());
    }
}
