use std::any::{self, TypeId};
use std::fmt::{self, Debug};
use std::hash::{Hash, Hasher};

#[readonly::make]
#[derive(Clone, Copy)]
pub struct TypeKey {
    pub id:   TypeId,
    pub name: &'static str,
}

impl TypeKey {
    #[must_use]
    pub fn of<T: 'static>() -> Self {
        Self { id: TypeId::of::<T>(), name: any::type_name::<T>() }
    }
}

impl Hash for TypeKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for TypeKey {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for TypeKey {}

impl Debug for TypeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::TypeKey;

    #[test]
    fn typekey_eq() {
        struct A;
        struct B;
        assert_eq!(TypeKey { id: std::any::TypeId::of::<A>(), name: "foo" },
                   TypeKey { id: std::any::TypeId::of::<A>(), name: "bar" });
        assert_ne!(TypeKey { id: std::any::TypeId::of::<A>(), name: "foo" },
                   TypeKey { id: std::any::TypeId::of::<B>(), name: "foo" });
    }

    #[test]
    fn typekey_of() {
        struct A;
        struct B;
        assert_eq!(TypeKey::of::<A>(), TypeKey::of::<A>());
        assert_ne!(TypeKey::of::<A>(), TypeKey::of::<B>());
    }
}
