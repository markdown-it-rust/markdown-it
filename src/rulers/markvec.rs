use std::fmt::Debug;
use std::hash::Hash;
use std::collections::HashSet;

///
/// MarkVec (marked vector) is a data structure used for chain-of-responsibility pattern.
///
/// It allows you to implement a plugin system with dependency management and ensure that
/// your dependencies are called in the correct order.
///
/// You can use it like this:
/// ```
/// use markdown_it::rulers::markvec::MarkVec;
///
/// // this example prints "[ hello, world! ]",
/// // where each token is printed by separate closure
/// let mut chain = MarkVec::<&str, fn () -> ()>::new();
///
/// // define rules printing "hello" and "world"
/// chain.push("hello", || print!("hello"));
/// chain.push("world", || print!("world"));
///
/// // open bracket should be before "hello", and closing one after "world"
/// chain.before("hello").push("open_bracket", || print!("[ "));
/// chain.after("world").push("close_bracket", || print!(" ]"));
///
/// // between "hello" and "world" we shall have a comma
/// chain.after("hello").before("world").push("comma", || print!(", "));
///
/// // after "world" we should have "!" as a first rule, but ensure "world" exists first
/// chain.after("world").require("world").unshift("bang", || print!("!"));
///
/// // add a newline after all rules
/// chain.push("newline", || print!("\n"));
///
/// // now we run this chain
/// for (_, f) in chain.iter() { f() }
///
/// // outputs "[ hello, world! ]"
/// ```
///
/// This data structure contains any number of elements (M, T), where T is any type and
/// M (mark) is its identifier.
///
///  - `M` is used for ordering and dependency checking, it must implement `Eq + Copy + Hash + Debug`
/// for this purpose. Common choices for `M` are `u32`, `&'static str`, or a special `Symbol` type
/// designed for this purpose.
///
///  - `T` is any user-defined type. It's usually a function or boxed trait.
///
pub struct MarkVec<M, T> {
    vec: Vec<(M, T)>,
    deps: HashSet<M>,
}

impl<M, T> MarkVec<M, T> {
    pub fn new() -> Self {
        Self {
            vec: Vec::new(),
            deps: HashSet::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn iter(&self) -> std::slice::Iter<(M, T)> {
        self.vec.iter()
    }

    pub fn remove(&mut self, index: usize) {
        self.vec.remove(index);
    }
}

impl<M: Eq + Copy + Hash + Debug, T> MarkVec<M, T> {
    /// If `mark` exists, following element inserted with `.push`/`.unshift` will be placed before it.
    pub fn before(&mut self, mark: M) -> Range<M, T> {
        Range::new(self).before(mark)
    }

    /// If `mark` exists, following element inserted with `.push`/`.unshift` will be placed after it.
    pub fn after(&mut self, mark: M) -> Range<M, T> {
        Range::new(self).after(mark)
    }

    /// Insert an element at the end of the chain.
    pub fn push(&mut self, mark: M, value: T) {
        self.insert(self.vec.len(), mark, value);
    }

    /// Insert an element at the start of the chain.
    pub fn unshift(&mut self, mark: M, value: T) {
        self.insert(0, mark, value);
    }

    /// Insert an element at an arbitrary position of the chain. It is not recommended, you should use `.before`/`.after`
    /// instead to declare positioning.
    pub fn insert(&mut self, index: usize, mark: M, value: T) {
        if self.deps.get(&mark).is_some() {
            panic!("cannot add {:?} because this mark was used as before/after earlier (wrong loading order)", mark)
        }
        self.vec.insert(index, (mark, value));
    }

    /// Require `mark` to exists in the chain, panic if it doesn't.
    pub fn require(&mut self, mark: M) -> Range<M, T> {
        Range::new(self).require(mark)
    }
}

impl<M: Eq, T> MarkVec<M, T> {
    /// Find a position of first `mark`.
    pub fn position(&mut self, mark: M) -> Option<usize> {
        self.vec.iter().position(|(m, _)| mark == *m)
    }

    /// Find a position of last `mark`.
    pub fn rposition(&mut self, mark: M) -> Option<usize> {
        self.vec.iter().rposition(|(m, _)| mark == *m)
    }
}

impl<M: Debug, T> Debug for MarkVec<M, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter().map(|(m, _)| m)).finish()
    }
}

///
/// This struct supports chained syntax for [.before()](MarkVec::before),
/// [.after()](MarkVec::after), etc.
///
/// It should not be used directly.
///
/// For documentation on each method, see [MarkVec](MarkVec).
///
pub struct Range<'vec, M: Eq + Copy + Hash + Debug, T> {
    mvec: &'vec mut MarkVec<M, T>,
    start: usize,
    end: usize,
}

impl<'vec, M: Eq + Copy + Hash + Debug, T> Range<'vec, M, T> {
    pub fn new(mvec: &'vec mut MarkVec<M, T>) -> Self {
        let len = mvec.len();
        Self {
            mvec,
            start: 0,
            end: len,
        }
    }

    pub fn before(mut self, mark: M) -> Self {
        self.mvec.deps.insert(mark);
        if let Some(pos) = self.mvec.position(mark) {
            if pos < self.end {
                self.end = pos;
                self.check_range();
            }
        }
        self
    }

    pub fn after(mut self, mark: M) -> Self {
        self.mvec.deps.insert(mark);
        if let Some(pos) = self.mvec.rposition(mark) {
            if pos >= self.start {
                self.start = pos + 1;
                self.check_range();
            }
        }
        self
    }

    fn check_range(&self) {
        if self.start > self.end {
            let (start_mark, _) = self.mvec.iter().skip(self.start - 1).next().unwrap();
            let (end_mark, _)   = self.mvec.iter().skip(self.end).next().unwrap();
            panic!(".before({:?}).after({:?}): before mark >= after mark", end_mark, start_mark);
        }
    }

    pub fn require(self, mark: M) -> Self {
        if self.mvec.rposition(mark).is_none() {
            panic!(".require({:?}): required mark doesn't exist", mark);
        }
        self
    }

    pub fn push(self, mark: M, value: T) {
        self.mvec.insert(self.end, mark, value);
    }

    pub fn unshift(self, mark: M, value: T) {
        self.mvec.insert(self.start, mark, value);
    }
}


#[cfg(test)]
mod tests {
    use std::fmt::Display;
    use super::MarkVec;

    fn vec2str<M: Display, T>(mvec: &MarkVec<M, T>) -> String {
        mvec.iter().map(|(m, _)| m.to_string()).collect::<Vec<String>>().join(",")
    }

    #[test]
    fn title_example() {
        let mut chain = MarkVec::<&str, fn () -> &'static str>::new();

        chain.push("hello", || "hello");
        chain.push("world", || "world");
        chain.before("hello").push("open_bracket", || "[ ");
        chain.after("world").push("close_bracket", || " ]");
        chain.after("hello").before("world").push("comma", || ", ");
        chain.after("world").require("world").unshift("!", || "!");

        let mut res = String::new();
        for (_, f) in chain.iter() { res += f(); }

        assert_eq!(res, "[ hello, world! ]");
    }

    #[test]
    fn markvec_find() {
        let mut vec = MarkVec::new();
        vec.push("A", ());
        vec.push("X", ());
        vec.push("B", ());
        vec.push("X", ());
        vec.push("X", ());
        assert_eq!(vec.position("X"), Some(1));
        assert_eq!(vec.position("Z"), None);
        assert_eq!(vec.rposition("X"), Some(4));
    }

    #[test]
    fn markvec_remove() {
        let mut vec = MarkVec::new();
        vec.push(1, ());
        vec.push(2, ());
        vec.push(3, ());
        vec.push(4, ());
        vec.remove(1);
        vec.remove(1);
        assert_eq!(vec2str(&vec), "1,4");
    }

    #[test]
    fn push_before_or_after_nonexistant() {
        let mut vec = MarkVec::new();
        vec.push(1, ());
        vec.push(2, ());
        vec.before(123).push(7, ());
        vec.after(123).push(8, ());
        assert_eq!(vec2str(&vec), "1,2,7,8");
    }

    #[test]
    fn require_exists() {
        let mut vec = MarkVec::new();
        vec.push(1, ());
        vec.before(1).before(1).before(1).require(1).push(2, ());
        assert_eq!(vec2str(&vec), "2,1");
    }

    #[test]
    #[should_panic(expected=".require(123): required mark doesn't exist")]
    fn require_nonexistant() {
        let mut vec = MarkVec::new();
        vec.push(1, ());
        vec.after(123).require(123).push(8, ());
    }

    #[test]
    #[should_panic(expected=".before(1).after(1): before mark >= after mark")]
    fn push_before_and_after_same_mark() {
        let mut vec = MarkVec::new();
        vec.push(1, ());
        vec.before(1).after(1).push(9, ());
    }

    #[test]
    #[should_panic(expected=".before(2).after(3): before mark >= after mark")]
    fn push_before_and_after_wrong_order() {
        let mut vec = MarkVec::new();
        vec.push(2, ());
        vec.push(3, ());
        vec.push(2, ());
        vec.before(2).after(3).push(9, ());
    }

    #[test]
    fn markvec_debug() {
        let mut vec = MarkVec::new();
        vec.push(1, ());
        vec.push(2, ());
        vec.push(3, ());
        assert_eq!(format!("{:?}", vec), "[1, 2, 3]");
    }

    #[test]
    #[should_panic(expected="cannot add \"A\" because this mark was used as before/after earlier (wrong loading order)")]
    fn insert_dependency_after_it_is_referenced() {
        let mut vec = MarkVec::new();
        vec.unshift("A", ());
        vec.before("A").push("beforeA", ());
        vec.unshift("A", ()); // fails because beforeA depended on it
        //assert_eq!(vec2str(&vec), "beforeA,A,A"); // this is what user would expect
    }
}
