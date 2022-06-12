// Block-level tokenizer
//
pub mod state;
pub use state::State;

use crate::Env;
use crate::MarkdownIt;
use crate::ruler::Ruler;
use crate::token::Token;

pub type Rule = fn (&mut State, bool) -> bool;

#[derive(Debug)]
pub struct Parser {
    // [[Ruler]] instance. Keep configuration of block rules.
    pub ruler: Ruler<Rule>,
}

impl Parser {
    pub fn new() -> Self {
        Self { ruler: Ruler::new() }
    }

    // Generate tokens for input range
    //
    pub fn tokenize(&self, state: &mut State) {
        let max_nesting = state.md.options.max_nesting.unwrap_or(100);
        let mut has_empty_lines = false;

        while state.line < state.line_max {
            state.line = state.skip_empty_lines(state.line);
            if state.line >= state.line_max { break; }

            // Termination condition for nested calls.
            // Nested calls currently used for blockquotes & lists
            if state.s_count[state.line] < state.blk_indent as i32 { break; }

            // If nesting level exceeded - skip tail to the end. That's not ordinary
            // situation and we should not care about content.
            if state.level >= max_nesting {
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

            for rule in self.ruler.get_rules() {
                ok = rule(state, false);
                if ok {
                    if prev_line >= state.line { panic!("block rule didn't increment state.line"); }
                    break;
                }
            }

            // this can only happen if user disables paragraph rule
            if !ok { panic!("none of the block rules matched"); }

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
    pub fn parse(&self, src: &str, md: &MarkdownIt, env: &mut Env, out_tokens: &mut Vec<Token>) {
        let mut state = State::new(src, md, env, out_tokens);
        self.tokenize(&mut state)
    }
}
