// Extracted code from https://github.com/reem/rust-typemap, and adapted for shio usage
use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::hash::{BuildHasherDefault, Hasher};
use std::ptr;

use unsafe_any::{UnsafeAny, UnsafeAnyExt};

#[derive(Default)]
pub struct TypeIdHasherValue {
    value: u64,
}

impl Hasher for TypeIdHasherValue {
    fn finish(&self) -> u64 {
        self.value
    }

    fn write(&mut self, bytes: &[u8]) {
        if bytes.len() != 8 {
            panic!("unexpected len for typeid hash");
        }

        let buffer = &mut self.value as *mut u64;
        let buffer = buffer as *mut u8;

        let orig = bytes.as_ptr();

        unsafe {
            ptr::copy_nonoverlapping(orig, buffer, 8);
        }
    }
}

// exported only for avoid "private in public" error
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
    data: HashMap<TypeId, Box<A>, BuildHasherDefault<TypeIdHasherValue>>,
}

/// This trait defines the relationship between keys and values in a `TypeMap`.
///
/// It is implemented for Keys, with a phantom associated type for the values.
pub trait Key: Any {
    /// The value type associated with this key type.
    type Value: Any;
}

#[cfg(feature = "nightly")]
default impl<T: 'static> Key for T {
    type Value = T;
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
    pub fn insert<K: Key>(&mut self, val: K::Value) -> Option<K::Value>
    where
        K::Value: Any + Implements<A>,
    {
        self.data
            .insert(TypeId::of::<K>(), val.into_object())
            .map(|v| unsafe { *v.downcast_unchecked::<K::Value>() })
    }

    /// Find a value in the map and get a reference to it.
    pub fn get<K: Key>(&self) -> Option<&K::Value>
    where
        K::Value: Any + Implements<A>,
    {
        self.data
            .get(&TypeId::of::<K>())
            .map(|v| unsafe { v.downcast_ref_unchecked::<K::Value>() })
    }

    /// Check if a key has an associated value stored in the map.
    pub fn contains<K: Key>(&self) -> bool {
        self.data.contains_key(&TypeId::of::<K>())
    }
}

pub type ShareMap = TypeMap<UnsafeAny + Sync + Send>;

#[cfg(test)]
mod tests {
    use std::mem;
    use std::hash::{Hash, Hasher};

    use super::{Key, TypeMap, TypeId, TypeIdHasherValue};

    #[derive(Debug, PartialEq)]
    struct KeyType;

    #[derive(Clone, Debug, PartialEq)]
    struct ValueType(u8);

    impl Key for KeyType {
        type Value = ValueType;
    }

    #[test]
    fn test_key_value() {
        let mut map = TypeMap::new();
        map.insert::<KeyType>(ValueType(32));

        assert_eq!(*map.get::<KeyType>().unwrap(), ValueType(32));
        assert!(map.contains::<KeyType>());
    }

    #[test]
    fn test_type_id_hasher() {
        fn verify_hashing_with(type_id: TypeId) {
            let mut hasher = TypeIdHasherValue::default();
            type_id.hash(&mut hasher);

            assert_eq!(hasher.finish(), unsafe { mem::transmute::<TypeId, u64>(type_id) });
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
