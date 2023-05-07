//! Extension sets
//!
//! These things allow you to put custom data inside internal markdown-it structures.
//!
use downcast_rs::{impl_downcast, Downcast};
use std::fmt::Debug;

/// Extension set member for the entire parser (only writable at init).
pub trait MarkdownItExt : Debug + Downcast + Send + Sync {}
impl_downcast!(MarkdownItExt);
extension_set!(MarkdownItExtSet, MarkdownItExt);

/// Extension set member for an arbitrary AST node.
pub trait NodeExt : Debug + Downcast + Send + Sync {}
impl_downcast!(NodeExt);
extension_set!(NodeExtSet, NodeExt);

/// Extension set member for an inline context.
pub trait InlineRootExt : Debug + Downcast + Send + Sync {}
impl_downcast!(InlineRootExt);
extension_set!(InlineRootExtSet, InlineRootExt);

/// Extension set member for a block context.
pub trait RootExt : Debug + Downcast + Send + Sync {}
impl_downcast!(RootExt);
extension_set!(RootExtSet, RootExt);

/// Extension set member for a renderer context.
pub trait RenderExt : Debug + Downcast + Send + Sync {}
impl_downcast!(RenderExt);
extension_set!(RenderExtSet, RenderExt);

// see https://github.com/malobre/erased_set for inspiration and API
// see https://lucumr.pocoo.org/2022/1/7/as-any-hack/ for additional impl details
macro_rules! extension_set {
    ($name: ident, $trait: ident) => {
        #[derive(Debug, Default)]
        pub struct $name(::std::collections::HashMap<crate::common::TypeKey, Box<dyn $trait>>);

        impl $name {
            #[must_use]
            pub fn new() -> Self {
                Self::default()
            }

            #[must_use]
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            #[must_use]
            pub fn len(&self) -> usize {
                self.0.len()
            }

            pub fn clear(&mut self) {
                self.0.clear();
            }

            #[must_use]
            pub fn contains<T: 'static>(&self) -> bool {
                let key = crate::common::TypeKey::of::<T>();
                self.0.contains_key(&key)
            }

            #[must_use]
            pub fn get<T: $trait>(&self) -> Option<&T> {
                let key = crate::common::TypeKey::of::<T>();
                let result = self.0.get(&key)?;
                result.downcast_ref::<T>()
            }

            #[must_use]
            pub fn get_mut<T: $trait>(&mut self) -> Option<&mut T> {
                let key = crate::common::TypeKey::of::<T>();
                let result = self.0.get_mut(&key)?;
                result.downcast_mut::<T>()
            }

            pub fn get_or_insert<T: $trait>(&mut self, value: T) -> &mut T {
                let key = crate::common::TypeKey::of::<T>();
                let result = self.0.entry(key).or_insert_with(|| Box::new(value));
                result.downcast_mut::<T>().unwrap()
            }

            pub fn get_or_insert_with<T: $trait>(&mut self, f: impl FnOnce() -> T) -> &mut T {
                let key = crate::common::TypeKey::of::<T>();
                let result = self.0.entry(key).or_insert_with(|| Box::new(f()));
                result.downcast_mut::<T>().unwrap()
            }

            pub fn get_or_insert_default<T: $trait + Default>(&mut self) -> &mut T {
                let key = crate::common::TypeKey::of::<T>();
                let result = self.0.entry(key).or_insert_with(|| Box::<T>::default());
                result.downcast_mut::<T>().unwrap()
            }

            pub fn insert<T: $trait>(&mut self, value: T) -> Option<T> {
                let key = crate::common::TypeKey::of::<T>();
                let result = self.0.insert(key, Box::new(value))?;
                Some(*result.downcast::<T>().unwrap())
            }

            pub fn remove<T: $trait>(&mut self) -> Option<T> {
                let key = crate::common::TypeKey::of::<T>();
                let result = self.0.remove(&key)?;
                Some(*result.downcast::<T>().unwrap())
            }
        }
    }
}

pub(crate) use extension_set;

#[cfg(test)]
mod tests {
    use super::extension_set;
    use downcast_rs::{Downcast, impl_downcast};
    use std::fmt::Debug;

    pub trait TestExt : Debug + Downcast + Send + Sync {}
    impl_downcast!(TestExt);

    extension_set!(TestExtSet, TestExt);

    impl<T: Debug + Downcast + Send + Sync> TestExt for T {}

    #[test]
    fn empty_set() {
        let set = TestExtSet::new();
        assert_eq!(set.len(), 0);
        assert!(set.is_empty());
    }

    #[test]
    fn insert_elements() {
        let mut set = TestExtSet::new();
        set.insert(42u8);
        assert_eq!(set.len(), 1);
        assert!(!set.is_empty());
        set.insert(42u16);
        assert_eq!(set.len(), 2);
        assert!(!set.is_empty());
    }

    #[test]
    fn contains() {
        let mut set = TestExtSet::new();
        set.insert(42u8);
        assert!(!set.contains::<u16>());
        set.insert(42u16);
        assert!(set.contains::<u16>());
        set.remove::<u16>();
        assert!(!set.contains::<u16>());
    }

    #[test]
    fn get() {
        let mut set = TestExtSet::new();
        set.insert(42u8);
        assert_eq!(set.get::<u16>(), None);
        set.insert(42u16);
        set.insert(123u16);
        assert_eq!(set.get::<u16>(), Some(&123u16));
    }

    #[test]
    fn get_mut() {
        let mut set = TestExtSet::new();
        set.insert(42u16);
        *set.get_mut::<u16>().unwrap() = 123u16;
        assert_eq!(set.get::<u16>(), Some(&123u16));
    }

    #[test]
    fn or_insert() {
        let mut set = TestExtSet::new();
        set.insert(123u8);
        assert_eq!(set.get_or_insert(0u8), &mut 123u8);
        assert_eq!(set.get_or_insert_default::<u8>(), &mut 123u8);
        assert_eq!(set.get_or_insert_with(|| 0u8), &mut 123u8);
        set.clear();
        assert_eq!(set.get_or_insert(10u8), &mut 10u8);
        set.clear();
        assert_eq!(set.get_or_insert_with(|| 20u8), &mut 20u8);
        set.clear();
        assert_eq!(set.get_or_insert_default::<u8>(), &mut 0u8);
    }

    #[test]
    fn different_types_stored_once() {
        let mut set = TestExtSet::new();
        set.insert("foo");
        set.insert("bar");
        set.insert("quux");
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn zero_sized_types() {
        #[derive(Debug, PartialEq, Eq)]
        struct A;
        #[derive(Debug, PartialEq, Eq)]
        struct B;
        let mut set = TestExtSet::new();
        set.insert(A);
        set.insert(B);
        assert_eq!(set.len(), 2);
        assert_eq!(set.get::<A>(), Some(&A));
    }

    #[test]
    fn clear() {
        let mut set = TestExtSet::new();
        set.insert(42u8);
        set.insert(42u16);
        assert_eq!(set.len(), 2);
        set.clear();
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn debug() {
        let mut set = TestExtSet::new();
        set.insert(42);
        set.insert("test");
        let str = format!("{:?}", set);
        // there are no guarantees about field order, so check both
        assert!(str == "TestExtSet({i32: 42, &str: \"test\"})" ||
                str == "TestExtSet({&str: \"test\", i32: 42})");
    }
}
