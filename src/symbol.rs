type NameFn = fn() -> (std::any::TypeId, &'static str);

///
/// Symbol is a small value that's guaranteed to be unique within Rust program
/// (heavily inspired by Symbol primitive in JavaScript).
///
/// You should use [symbol!()](symbol) macro to create a new Symbol.
///
/// Internally, it's just a pointer to a function that returns its string
/// representation (for debugging).
///
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Symbol(NameFn);

impl Symbol {
    #[doc(hidden)]
    pub const fn __new__(namefn: NameFn) -> Self {
        Self(namefn)
    }

    #[inline]
    ///
    /// Get unique id associated with this Symbol.
    ///
    pub fn id(&self) -> usize {
        self.0 as *const () as usize
    }

    ///
    /// Get string representation of this Symbol
    /// (used primarily for debugging purposes).
    ///
    pub fn name(&self) -> &'static str {
        let res = (self.0)().1;
        if res.ends_with("::__Symbol__") {
            &res[0..res.len() - 12]
        } else {
            res
        }
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

///
/// Creates a new [Symbol](Symbol).
///
/// Usage:
/// ```
/// use markdown_it::{symbol, Symbol};
/// const A : Symbol = symbol!();
/// const B : Symbol = A;
/// const C : Symbol = symbol!();
/// assert_eq!(A, B);
/// assert_ne!(A, C);
/// println!("{}", A.to_string());
/// ```
///
#[macro_export]
macro_rules! symbol {
    () => {{
        struct __Symbol__;
        $crate::Symbol::__new__(
            || {
                // line and column are used to prevent rust from optimizing
                // symbols with same path into one in release mode
                (::std::any::TypeId::of::<__Symbol__>(), ::std::any::type_name::<__Symbol__>())
            }
        )
    }}
}

#[cfg(test)]
mod tests {
    use crate::symbol;
    use super::Symbol;

    #[test]
    fn symbol_format() {
        const SYM : Symbol = symbol!();
        assert!(format!("{}", SYM).ends_with("::SYM"));
        assert!(SYM.to_string().ends_with("::SYM"));
    }

    #[test]
    fn same_symbol_equal() {
        const A : Symbol = symbol!();
        assert_eq!(A, A);
    }

    #[test]
    fn same_name_not_equal() {
        let a : Symbol = symbol!();
        let b : Symbol = symbol!();
        assert_eq!(a.name(), b.name());
        assert_ne!(a, b);
    }

    #[test]
    fn can_use_in_hash() {
        let mut set = std::collections::HashSet::new();
        const A : Symbol = symbol!();
        const B : Symbol = symbol!();
        set.insert(&A);
        assert!(set.contains(&A));
        assert!(!set.contains(&B));
    }

    #[test]
    fn should_be_small_and_copyable() {
        const A : Symbol = symbol!();
        assert_eq!(std::mem::size_of_val(&A), 8);
        let a = A;
        let b = A;
        assert_eq!(a, b);
    }
}
