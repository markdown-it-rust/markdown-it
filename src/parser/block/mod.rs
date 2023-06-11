//! Block rule chain
use anyhow::Context;
use derivative::Derivative;
use once_cell::sync::OnceCell;

mod state;
pub use state::*;

mod rule;
pub use rule::*;

#[doc(hidden)]
pub mod builtin;

use crate::common::ruler::Ruler;
use crate::common::TypeKey;
use crate::parser::extset::RootExtSet;
use crate::parser::inline::InlineRoot;
use crate::parser::node::NodeEmpty;
use crate::{MarkdownIt, Node, Result};

#[derive(Clone)]
#[doc(hidden)]
pub struct RuleStruct {
    marker: TypeKey,
    check: fn (&mut BlockState) -> Option<()>,
    run: fn (&mut BlockState) -> Option<(Node, usize)>,
    try_run: fn (&mut BlockState) -> Result<Option<(Node, usize)>>,
}

struct RuleStructVecs {
    marker: Vec<TypeKey>,
    check: Vec<fn (&mut BlockState) -> Option<()>>,
    run: Vec<fn (&mut BlockState) -> Option<(Node, usize)>>,
    try_run: Vec<fn (&mut BlockState) -> Result<Option<(Node, usize)>>>,
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
/// Block-level tokenizer.
pub struct BlockParser {
    ruler: Ruler<TypeKey, RuleStruct>,
    #[derivative(Debug = "ignore")]
    compiled_rules: OnceCell<RuleStructVecs>,
}

impl BlockParser {
    pub fn new() -> Self {
        Self::default()
    }

    // Generate tokens for input range
    //
    pub fn tokenize(&self, state: &mut BlockState) {
        // _tokenize with CAN_FAIL=false never returns errors
        let _ = Self::_tokenize::<false>(self, state);
    }

    // Generate tokens for input range, but fail if any custom rule produces an error.
    // Note: inline state will be unusable if you get an Error from this function.
    //
    pub fn try_tokenize(&self, state: &mut BlockState) -> Result<()> {
        Self::_tokenize::<true>(self, state)
    }

    fn _tokenize<const CAN_FAIL: bool>(&self, state: &mut BlockState) -> Result<()> {
        stacker::maybe_grow(64*1024, 1024*1024, || {
            let rules = self.compiled_rules.get().expect("rules not compiled");
            let mut has_empty_lines = false;

            while state.line < state.line_max {
                state.line = state.skip_empty_lines(state.line);
                if state.line >= state.line_max { break; }

                // Termination condition for nested calls.
                // Nested calls currently used for blockquotes & lists
                if state.line_indent(state.line) < 0 { break; }

                // If nesting level exceeded - skip tail to the end. That's not ordinary
                // situation and we should not care about content.
                if state.level >= state.md.max_nesting {
                    state.line = state.line_max;
                    break;
                }

                // Try all possible rules.
                // On success, rule should:
                //
                // - update `state.line`
                // - update `state.tokens`
                // - return true
                let mut ok = None;

                if CAN_FAIL {
                    for (idx, rule) in rules.try_run.iter().enumerate() {
                        ok = rule(state).with_context(|| BlockRuleError {
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

                if let Some((mut node, len)) = ok {
                    state.line += len;
                    if !node.is::<NodeEmpty>() {
                        node.srcmap = state.get_map(state.line - len, state.line - 1);
                        state.node.children.push(node);
                    }
                } else {
                    // this can only happen if user disables paragraph rule
                    // push text as is, this behavior can change in the future;
                    // users should always have some kind of default block rule
                    let mut content = state.get_line(state.line).to_owned();
                    content.push('\n');
                    let node = Node::new(InlineRoot::new(
                        content,
                        vec![(0, state.line_offsets[state.line].first_nonspace)],
                    ));
                    state.node.children.push(node);
                    state.line += 1;
                }

                // set state.tight if we had an empty line before current tag
                // i.e. latest empty line should not count
                state.tight = !has_empty_lines;

                // paragraph might "eat" one newline after it in nested lists
                if state.is_empty(state.line - 1) {
                    has_empty_lines = true;
                }

                if state.line < state.line_max && state.is_empty(state.line) {
                    has_empty_lines = true;
                    state.line += 1;
                }
            }

            Ok(())
        })
    }

    // Process input string and push block tokens into `out_tokens`
    //
    pub fn parse(&self, src: &str, node: Node, md: &MarkdownIt, root_ext: &mut RootExtSet) -> Node {
        let mut state = BlockState::new(src, md, root_ext, node);
        self.tokenize(&mut state);
        state.node
    }

    // Process input string and push block tokens into `out_tokens`,
    // fail if any custom rule produces an error.
    //
    pub fn try_parse(&self, src: &str, node: Node, md: &MarkdownIt, root_ext: &mut RootExtSet) -> Result<Node> {
        let mut state = BlockState::new(src, md, root_ext, node);
        self.try_tokenize(&mut state)?;
        Ok(state.node)
    }

    pub fn add_rule<T: BlockRule>(&mut self) -> RuleBuilder<RuleStruct> {
        self.compiled_rules = OnceCell::new();
        let item = self.ruler.add(TypeKey::of::<T>(), RuleStruct {
            marker: TypeKey::of::<T>(),
            check: T::check,
            run: T::run,
            try_run: T::try_run,
        });
        RuleBuilder::new(item)
    }

    pub fn has_rule<T: BlockRule>(&self) -> bool {
        self.ruler.contains(TypeKey::of::<T>())
    }

    pub fn remove_rule<T: BlockRule>(&mut self) {
        self.compiled_rules = OnceCell::new();
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
