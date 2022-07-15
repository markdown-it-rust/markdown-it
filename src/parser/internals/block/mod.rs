// Block-level tokenizer
//
pub mod state;
pub use state::State;

use crate::Node;
use crate::parser::MarkdownIt;
use crate::parser::internals::erasedset::ErasedSet;
use crate::parser::internals::ruler::Ruler;

pub type Rule = fn (&mut State, bool) -> bool;

#[derive(Debug, Default)]
pub struct BlockParser {
    // [[Ruler]] instance. Keep configuration of block rules.
    pub ruler: Ruler<&'static str, Rule>,
}

impl BlockParser {
    pub fn new() -> Self {
        Self::default()
    }

    // Generate tokens for input range
    //
    pub fn tokenize(&self, state: &mut State) {
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
            let mut ok = false;
            let prev_line = state.line;

            for rule in self.ruler.iter() {
                ok = rule(state, false);
                if ok {
                    assert!(state.line > prev_line, "block rule didn't increment state.line");
                    break;
                }
            }

            // this can only happen if user disables paragraph rule
            assert!(ok, "none of the block rules matched");

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
    }

    // Process input string and push block tokens into `out_tokens`
    //
    pub fn parse(&self, src: &str, node: Node, md: &MarkdownIt, env: &mut ErasedSet) -> Node {
        let mut state = State::new(src, md, env, node);
        self.tokenize(&mut state);
        state.node
    }
}
