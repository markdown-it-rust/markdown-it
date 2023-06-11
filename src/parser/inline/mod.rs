//! Inline rule chain
use anyhow::Context;
use derivative::Derivative;
use once_cell::sync::OnceCell;
use std::collections::HashMap;

mod state;
pub use state::*;

mod rule;
pub use rule::*;

#[doc(hidden)]
pub mod builtin;

pub use builtin::inline_parser::InlineRoot;
pub use builtin::skip_text::{Text, TextSpecial};
use builtin::skip_text::TextScannerImpl;

use crate::{MarkdownIt, Node, Result};
use crate::common::TypeKey;
use crate::common::ruler::Ruler;
use crate::parser::extset::{InlineRootExtSet, RootExtSet};

use super::node::NodeEmpty;

#[derive(Clone)]
#[doc(hidden)]
pub struct RuleStruct {
    marker: TypeKey,
    check: fn (&mut InlineState) -> Option<usize>,
    run: fn (&mut InlineState) -> Option<(Node, usize)>,
    try_run: fn (&mut InlineState) -> Result<Option<(Node, usize)>>,
}

// use (Vec<A>, Vec<B>, Vec<C>) instead of Vec<(A, B, C)> for cache locality,
// since only one thing will be accessed at a time, and code is hot
struct RuleStructVecs {
    marker: Vec<TypeKey>,
    check: Vec<fn (&mut InlineState) -> Option<usize>>,
    run: Vec<fn (&mut InlineState) -> Option<(Node, usize)>>,
    try_run: Vec<fn (&mut InlineState) -> Result<Option<(Node, usize)>>>,
}

impl RuleStructVecs {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            marker: Vec::with_capacity(capacity),
            check: Vec::with_capacity(capacity),
            run: Vec::with_capacity(capacity),
            try_run: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, rule: RuleStruct) {
        self.marker.push(rule.marker);
        self.check.push(rule.check);
        self.run.push(rule.run);
        self.try_run.push(rule.try_run);
    }
}

#[derive(Derivative, Default)]
#[derivative(Debug)]
/// Inline-level tokenizer.
pub struct InlineParser {
    ruler: Ruler<TypeKey, RuleStruct>,
    #[derivative(Debug = "ignore")]
    compiled_rules: OnceCell<RuleStructVecs>,
    text_charmap: HashMap<char, Vec<TypeKey>>,
    text_impl: OnceCell<TextScannerImpl>,
}

impl InlineParser {
    pub fn new() -> Self {
        Self::default()
    }

    // Skip single token by running all rules in validation mode;
    // returns `true` if any rule reported success
    //
    pub fn skip_token(&self, state: &mut InlineState) {
        stacker::maybe_grow(64*1024, 1024*1024, || {
            let rules = self.compiled_rules.get().expect("rules not compiled");
            let mut ok = None;

            if state.level < state.md.max_nesting {
                for rule in rules.check.iter() {
                    ok = rule(state);
                    if ok.is_some() {
                        break;
                    }
                };
            } else {
                // Too much nesting, just skip until the end of the paragraph.
                //
                // NOTE: this will cause links to behave incorrectly in the following case,
                //       when an amount of `[` is exactly equal to `maxNesting + 1`:
                //
                //       [[[[[[[[[[[[[[[[[[[[[foo]()
                //
                // TODO: remove this workaround when CM standard will allow nested links
                //       (we can replace it by preventing links from being parsed in
                //       validation mode)
                //
                state.pos = state.pos_max;
            }

            if let Some(len) = ok {
                state.pos += len;
            } else {
                let ch = state.src[state.pos..state.pos_max].chars().next().unwrap();
                state.pos += ch.len_utf8();
            }
        });
    }

    // Generate tokens for input range.
    //
    pub fn tokenize(&self, state: &mut InlineState) {
        // _tokenize with CAN_FAIL=false never returns errors
        let _ = Self::_tokenize::<false>(self, state);
    }

    // Generate tokens for input range, but fail if any custom rule produces an error.
    // Note: inline state will be unusable if you get an Error from this function.
    //
    pub fn try_tokenize(&self, state: &mut InlineState) -> Result<()> {
        Self::_tokenize::<true>(self, state)
    }

    fn _tokenize<const CAN_FAIL: bool>(&self, state: &mut InlineState) -> Result<()> {
        stacker::maybe_grow(64*1024, 1024*1024, || {
            let rules = self.compiled_rules.get().expect("rules not compiled");
            let end = state.pos_max;

            while state.pos < end {
                // Try all possible rules.
                // On success, rule should:
                //
                // - update `state.pos`
                // - update `state.tokens`
                // - return true
                let mut ok = None;

                if state.level < state.md.max_nesting {
                    if CAN_FAIL {
                        for (idx, rule) in rules.try_run.iter().enumerate() {
                            ok = rule(state).with_context(|| InlineRuleError {
                                name: rules.marker[idx],
                            })?;
                            if ok.is_some() {
                                break;
                            }
                        };
                    } else {
                        for rule in rules.run.iter() {
                            ok = rule(state);
                            if ok.is_some() {
                                break;
                            }
                        };
                    }
                }

                if let Some((mut node, len)) = ok {
                    state.pos += len;
                    if !node.is::<NodeEmpty>() {
                        node.srcmap = state.get_map(state.pos - len, state.pos);
                        state.node.children.push(node);
                        if state.pos >= end { break; }
                    }
                    continue;
                }

                let ch = state.src[state.pos..state.pos_max].chars().next().unwrap();
                let len = ch.len_utf8();
                state.trailing_text_push(state.pos, state.pos + len);
                state.pos += len;
            }

            Ok(())
        })
    }

    // Process input string and push inline tokens into `out_tokens`
    //
    pub fn parse(
        &self,
        src: String,
        srcmap: Vec<(usize, usize)>,
        node: Node,
        md: &MarkdownIt,
        root_ext: &mut RootExtSet,
        inline_ext: &mut InlineRootExtSet,
    ) -> Node {
        let mut state = InlineState::new(src, srcmap, md, root_ext, inline_ext, node);
        self.tokenize(&mut state);
        state.node
    }

    // Process input string and push inline tokens into `out_tokens`,
    // fail if any custom rule produces an error.
    //
    pub fn try_parse(
        &self,
        src: String,
        srcmap: Vec<(usize, usize)>,
        node: Node,
        md: &MarkdownIt,
        root_ext: &mut RootExtSet,
        inline_ext: &mut InlineRootExtSet,
    ) -> Result<Node> {
        let mut state = InlineState::new(src, srcmap, md, root_ext, inline_ext, node);
        self.try_tokenize(&mut state)?;
        Ok(state.node)
    }

    pub fn add_rule<T: InlineRule>(&mut self) -> RuleBuilder<RuleStruct> {
        self.compiled_rules = OnceCell::new();

        if T::MARKER != '\0' {
            let charvec = self.text_charmap.entry(T::MARKER).or_insert(vec![]);
            charvec.push(TypeKey::of::<T>());
        }

        let item = self.ruler.add(TypeKey::of::<T>(), RuleStruct {
            marker: TypeKey::of::<T>(),
            check: T::check,
            run: T::run,
            try_run: T::try_run,
        });

        RuleBuilder::new(item)
    }

    pub fn has_rule<T: InlineRule>(&self) -> bool {
        self.ruler.contains(TypeKey::of::<T>())
    }

    pub fn remove_rule<T: InlineRule>(&mut self) {
        self.compiled_rules = OnceCell::new();

        if T::MARKER != '\0' {
            let mut charvec = self.text_charmap.remove(&T::MARKER).unwrap_or_default();
            charvec.retain(|x| *x != TypeKey::of::<T>());
            self.text_charmap.insert(T::MARKER, charvec);
        }

        self.ruler.remove(TypeKey::of::<T>());
    }

    fn compile(&self) {
        self.compiled_rules.get_or_init(|| {
            let compiled_rules = self.ruler.compile();
            let mut result = RuleStructVecs::with_capacity(compiled_rules.len());

            for rule in compiled_rules {
                result.push(rule);
            }
            result
        });
    }
}
