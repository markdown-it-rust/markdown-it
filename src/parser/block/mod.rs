//! Block rule chain
mod state;
pub use state::*;

mod rule;
pub use rule::*;

#[doc(hidden)]
pub mod builtin;

use crate::{MarkdownIt, Node};
use crate::common::TypeKey;
use crate::common::ruler::Ruler;
use crate::parser::extset::RootExtSet;
use crate::parser::inline::InlineRoot;
use crate::parser::node::NodeEmpty;

type RuleFns = (
    fn (&mut BlockState) -> Option<()>,
    fn (&mut BlockState) -> Option<(Node, usize)>,
);

#[derive(Debug, Default)]
/// Block-level tokenizer.
pub struct BlockParser {
    ruler: Ruler<TypeKey, RuleFns>,
}

impl BlockParser {
    pub fn new() -> Self {
        Self::default()
    }

    // Generate tokens for input range
    //
    pub fn tokenize(&self, state: &mut BlockState) {
        stacker::maybe_grow(64*1024, 1024*1024, || {
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

                for rule in self.ruler.iter() {
                    ok = rule.1(state);
                    if ok.is_some() {
                        break;
                    }
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
        });
    }

    // Process input string and push block tokens into `out_tokens`
    //
    pub fn parse(&self, src: &str, node: Node, md: &MarkdownIt, root_ext: &mut RootExtSet) -> Node {
        let mut state = BlockState::new(src, md, root_ext, node);
        self.tokenize(&mut state);
        state.node
    }

    pub fn add_rule<T: BlockRule>(&mut self) -> RuleBuilder<RuleFns> {
        let item = self.ruler.add(TypeKey::of::<T>(), (T::check, T::run));
        RuleBuilder::new(item)
    }

    pub fn has_rule<T: BlockRule>(&mut self) -> bool {
        self.ruler.contains(TypeKey::of::<T>())
    }

    pub fn remove_rule<T: BlockRule>(&mut self) {
        self.ruler.remove(TypeKey::of::<T>());
    }
}
