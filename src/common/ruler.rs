use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::slice::Iter;
use once_cell::sync::OnceCell;
use derivative::Derivative;

///
/// Ruler allows you to implement a plugin system with dependency management and ensure that
/// your dependencies are called in the correct order.
///
/// You can use it like this:
/// ```
/// use markdown_it::common::ruler::Ruler;
///
/// // this example prints "[ hello, world! ]",
/// // where each token is printed by separate closure
/// let mut chain = Ruler::<&str, fn (&mut String)>::new();
///
/// // define rules printing "hello" and "world"
/// chain.add("hello", |s| s.push_str("hello"));
/// chain.add("world", |s| s.push_str("world"));
///
/// // open bracket should be before "hello", and closing one after "world"
/// chain.add("open_bracket", |s| s.push_str("[ ")).before("hello");
/// chain.add("close_bracket", |s| s.push_str(" ]")).after("world");
///
/// // between "hello" and "world" we shall have a comma
/// chain.add("comma", |s| s.push_str(", ")).after("hello").before("world");
///
/// // after "world" we should have "!" as a first rule, but ensure "world" exists first
/// chain.add("bang", |s| s.push_str("!")).require("world").after("world").before_all();
///
/// // now we run this chain
/// let mut result = String::new();
/// for f in chain.iter() { f(&mut result); }
/// assert_eq!(result, "[ hello, world! ]");
/// ```
///
/// This data structure contains any number of elements (M, T), where T is any type and
/// M (mark) is its identifier.
///
///  - `M` is used for ordering and dependency checking, it must implement `Eq + Copy + Hash + Debug`
/// . Common choices for `M` are `u32`, `&'static str`, or a special `Symbol` type
/// designed for this purpose.
///
///  - `T` is any user-defined type. It's usually a function or boxed trait.
///
pub struct Ruler<M, T> {
    deps: Vec<RuleItem<M, T>>,
    compiled: OnceCell<(Vec<usize>, Vec<T>)>,
}

impl<M, T> Ruler<M, T> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<M: Eq + Hash + Copy + Debug, T: Clone> Ruler<M, T> {
    /// Add a new rule identified by `mark` with payload `value`.
    pub fn add(&mut self, mark: M, value: T) -> &mut RuleItem<M, T> {
        self.compiled = OnceCell::new();
        let dep = RuleItem::new(mark, value);
        self.deps.push(dep);
        self.deps.last_mut().unwrap()
    }

    /// Remove all rules identified by `mark`.
    pub fn remove(&mut self, mark: M) {
        self.deps.retain(|dep| !dep.marks.contains(&mark));
    }

    /// Check if there are any rules identified by `mark`.
    pub fn contains(&mut self, mark: M) -> bool {
        self.deps.iter().any(|dep| dep.marks.contains(&mark))
    }

    /// Ordered iteration through rules.
    #[inline]
    pub fn iter(&self) -> Iter<T> {
        self.compiled.get_or_init(|| self.compile()).1.iter()
    }

    fn compile(&self) -> (Vec<usize>, Vec<T>) {
        // ID -> [RuleItem index]
        let mut idhash = HashMap::<M, Vec<usize>>::new();

        // RuleItem index -> [RuleItem index] - dependency graph, None if already inserted
        let mut deps_graph = vec![HashSet::new(); self.deps.len()];

        // additional level of indirection that takes into account item priority
        let mut deps_order = vec![];
        let mut beforeall_len = 0;
        let mut afterall_len = 0;

        // compiled result
        let mut result = vec![];
        let mut result_idx = vec![];

        // track which rules have been added already
        let mut deps_inserted = vec![false; self.deps.len()];
        let mut deps_remaining = self.deps.len();

        for (idx, dep) in self.deps.iter().enumerate() {
            match dep.prio {
                RuleItemPriority::Normal => {
                    deps_order.insert(deps_order.len() - afterall_len, idx);
                }
                RuleItemPriority::BeforeAll => {
                    deps_order.insert(beforeall_len, idx);
                    beforeall_len += 1;
                }
                RuleItemPriority::AfterAll => {
                    deps_order.insert(deps_order.len(), idx);
                    afterall_len += 1;
                }
            }
            for mark in &dep.marks {
                idhash.entry(*mark).or_default().push(idx);
            }
        }

        // build dependency graph, replacing all after's with before's,
        // i.e. B.after(A) -> A.before(B)
        for idx in deps_order.iter().copied() {
            let dep = self.deps.get(idx).unwrap();
            for constraint in &dep.cons {
                match constraint {
                    RuleItemConstraint::Before(v) => {
                        for depidx in idhash.entry(*v).or_default().iter() {
                            deps_graph.get_mut(*depidx).unwrap().insert(idx);
                        }
                    }
                    RuleItemConstraint::After(v) => {
                        for depidx in idhash.entry(*v).or_default().iter() {
                            deps_graph.get_mut(idx).unwrap().insert(*depidx);
                        }
                    }
                    RuleItemConstraint::Require(v) => {
                        assert!(
                            idhash.contains_key(v),
                            "missing dependency: {:?} requires {:?}", dep.marks.get(0).unwrap(), v
                        );
                    }
                }
            }
        }

        // now go through the deps and push whatever doesn't have any
        'outer: while deps_remaining > 0 {
            for idx in deps_order.iter().copied() {
                let inserted = deps_inserted.get_mut(idx).unwrap();
                if *inserted { continue; }

                let dlist = deps_graph.get(idx).unwrap();
                if dlist.is_empty() {
                    let dep = self.deps.get(idx).unwrap();
                    result.push(dep.value.clone());
                    result_idx.push(idx);
                    *inserted = true;
                    deps_remaining -= 1;
                    for d in deps_graph.iter_mut() {
                        d.remove(&idx);
                    }
                    continue 'outer;
                }
            }

            #[cfg(debug_assertions)] {
                // check cycles in dependency graph;
                // this is very suboptimal, but only used to generate a nice panic message.
                // in release mode we'll just simply panic
                for idx in deps_order.iter().copied() {
                    let mut seen = HashMap::new();
                    let mut vec = vec![idx];
                    while let Some(didx) = vec.pop() {
                        let dlist = deps_graph.get(didx).unwrap();
                        for x in dlist.iter() {
                            if seen.get(x).is_some() { continue; }
                            vec.push(*x);
                            seen.insert(*x, didx);
                            if *x == idx {
                                let mut backtrack = vec![];
                                let mut curr = idx;
                                while !backtrack.contains(&curr) {
                                    backtrack.push(curr);
                                    curr = *seen.get(&curr).unwrap();
                                }
                                backtrack.push(curr);
                                let path = backtrack.iter()
                                    .rev()
                                    .map(|x| format!("{:?}", self.deps.get(*x).unwrap().marks.get(0).unwrap()))
                                    .collect::<Vec<String>>()
                                    .join(" < ");
                                panic!("cyclic dependency: {}", path);
                            }
                        }
                    }
                }
            }

            // if you see this in debug mode, report it as a bug
            panic!("cyclic dependency: (use debug mode for more details)");
        }

        (result_idx, result)
    }
}

impl<M: Eq + Hash + Copy + Debug, T: Clone> Debug for Ruler<M, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vec: Vec<(usize, M)> = self.compiled.get_or_init(|| self.compile()).0
                                    .iter()
                                    .map(|idx| (*idx, *self.deps.get(*idx).unwrap().marks.get(0).unwrap()))
                                    .collect();

        f.debug_struct("Ruler")
            .field("deps", &self.deps)
            .field("compiled", &vec)
            .finish()
    }
}

impl<M, T> Default for Ruler<M, T> {
    fn default() -> Self {
        Self {
            deps: Vec::new(),
            compiled: OnceCell::new(),
        }
    }
}

///
/// Result of [Ruler::add](Ruler::add), allows to customize position of each rule.
///
#[derive(Derivative)]
#[derivative(Debug)]
pub struct RuleItem<M, T> {
    marks: Vec<M>,
    #[derivative(Debug="ignore")]
    value: T,
    prio: RuleItemPriority,
    cons: Vec<RuleItemConstraint<M>>,
}

impl<M, T> RuleItem<M, T> {
    fn new(mark: M, value: T) -> Self {
        Self {
            marks: vec![mark],
            value,
            prio: RuleItemPriority::Normal,
            cons: vec![],
        }
    }
}

impl<M: Copy, T> RuleItem<M, T> {
    /// Make sure this rule will be inserted before any rule defined by `mark` (if such rule exists).
    /// ```
    /// use markdown_it::common::ruler::Ruler;
    /// let mut chain = Ruler::<&str, fn (&mut String)>::new();
    ///
    /// chain.add("a", |s| s.push_str("bar"));
    /// chain.add("b", |s| s.push_str("foo")).before("a");
    ///
    /// let mut result = String::new();
    /// for f in chain.iter() { f(&mut result); }
    /// assert_eq!(result, "foobar");
    /// ```
    pub fn before(&mut self, mark: M) -> &mut Self {
        self.cons.push(RuleItemConstraint::Before(mark));
        self
    }

    /// Make sure this rule will be inserted after any rule defined by `mark` (if such rule exists).
    /// Similar to [RuleItem::before](RuleItem::before).
    pub fn after(&mut self, mark: M) -> &mut Self {
        self.cons.push(RuleItemConstraint::After(mark));
        self
    }

    /// This rule will be inserted as early as possible, while still taking into account dependencies,
    /// i.e. `.after(X).before_all()` causes this to be first rule after X.
    /// ```
    /// use markdown_it::common::ruler::Ruler;
    /// let mut chain = Ruler::<&str, fn (&mut String)>::new();
    ///
    /// chain.add("a", |s| s.push_str("A"));
    /// chain.add("c", |s| s.push_str("C")).after("a");
    /// chain.add("b", |s| s.push_str("B")).after("a").before_all();
    ///
    /// let mut result = String::new();
    /// for f in chain.iter() { f(&mut result); }
    /// // without before_all order will be ACB
    /// assert_eq!(result, "ABC");
    /// ```
    pub fn before_all(&mut self) -> &mut Self {
        self.prio = RuleItemPriority::BeforeAll;
        self
    }

    /// This rule will be inserted as late as possible, while still taking into account dependencies,
    /// i.e. `.before(X).after_all()` causes this to be last rule before X.
    /// Similar to [RuleItem::before_all](RuleItem::before_all).
    pub fn after_all(&mut self) -> &mut Self {
        self.prio = RuleItemPriority::AfterAll;
        self
    }

    /// Add another auxiliary identifier to this rule. It can be used to group together multiple
    /// rules with similar functionality.
    /// ```
    /// use markdown_it::common::ruler::Ruler;
    /// let mut chain = Ruler::<&str, fn (&mut String)>::new();
    ///
    /// chain.add("b", |s| s.push_str("B")).alias("BorC");
    /// chain.add("c", |s| s.push_str("C")).alias("BorC");
    /// chain.add("a", |s| s.push_str("A")).before("BorC");
    ///
    /// let mut result = String::new();
    /// for f in chain.iter() { f(&mut result); }
    /// assert_eq!(result, "ABC");
    /// ```
    pub fn alias(&mut self, mark: M) -> &mut Self {
        self.marks.push(mark);
        self
    }

    /// Require another rule identified by `mark`, panic if not found.
    pub fn require(&mut self, mark: M) -> &mut Self {
        self.cons.push(RuleItemConstraint::Require(mark));
        self
    }
}

#[derive(Debug)]
enum RuleItemConstraint<M> {
    Before(M),
    After(M),
    Require(M),
}

#[derive(Debug)]
enum RuleItemPriority {
    Normal,
    BeforeAll,
    AfterAll,
}


#[cfg(test)]
mod tests {
    use super::Ruler;

    #[test]
    #[should_panic(expected=r#"cyclic dependency: "A" < "B" < "C" < "D" < "E" < "F" < "A""#)]
    #[cfg(debug_assertions)]
    fn cyclic_dependency_debug() {
        let mut r = Ruler::new();
        r.add("%", ()).after("D");
        r.add("A", ()).after("B");
        r.add("E", ()).after("F");
        r.add("C", ()).after("D");
        r.add("B", ()).after("C");
        r.add("D", ()).after("E");
        r.add("F", ()).after("A");
        r.compile();
    }

    #[test]
    #[should_panic(expected=r#"cyclic dependency"#)]
    fn cyclic_dependency() {
        let mut r = Ruler::new();
        r.add("A", ()).after("B");
        r.add("B", ()).after("C");
        r.add("C", ()).after("A");
        r.compile();
    }


    #[test]
    #[should_panic(expected=r#"missing dependency: "C" requires "Z"#)]
    fn missing_require() {
        let mut r = Ruler::new();
        r.add("A", ());
        r.add("B", ()).require("A");
        r.add("C", ()).require("Z");
        r.compile();
    }
}
