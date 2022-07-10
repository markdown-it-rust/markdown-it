// Tokenizes paragraph content.
//
pub mod state;
pub use state::State;

use crate::Node;
use crate::parser::MarkdownIt;
use crate::parser::internals::env::Env;
use crate::parser::internals::env::scope::{Inline, InlineLvl};
use crate::parser::internals::ruler::Ruler;

use super::syntax_base::builtin::Root;

pub type Rule = fn (&mut State, silent: bool) -> bool;
pub type Rule2 = fn (&mut State);

#[derive(Debug)]
pub struct Parser {
    // [[Ruler]] instance. Keep configuration of inline rules.
    pub ruler: Ruler<&'static str, Rule>,

    // [[Ruler]] instance. Second ruler used for post-processing
    // (e.g. in emphasis-like rules).
    pub ruler2: Ruler<&'static str, Rule2>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            ruler: Ruler::new(),
            ruler2: Ruler::new(),
        }
    }

    // Skip single token by running all rules in validation mode;
    // returns `true` if any rule reported success
    //
    pub fn skip_token(&self, state: &mut State) {
        let pos = state.pos;
        let mut ok = false;

        if let Some(x) = state.cache.get(&pos) {
            state.pos = *x;
            return;
        }

        if state.level < state.md.max_nesting {
            for rule in self.ruler.iter() {
                ok = rule(state, true);
                if ok {
                    if pos >= state.pos { panic!("inline rule didn't increment state.pos"); }
                    break;
                }
            }
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

        if !ok {
            let ch = state.src[state.pos..state.pos_max].chars().next().unwrap();
            state.pos += ch.len_utf8();
        }
        state.cache.insert(pos, state.pos);
    }

    // Generate tokens for input range
    //
    pub fn tokenize(&self, state: &mut State) {
        state.env.state_push::<InlineLvl>();

        let end = state.pos_max;

        while state.pos < end {
            // Try all possible rules.
            // On success, rule should:
            //
            // - update `state.pos`
            // - update `state.tokens`
            // - return true
            let mut ok = false;
            let prev_pos = state.pos;

            if state.level < state.md.max_nesting {
                for rule in self.ruler.iter() {
                    ok = rule(state, false);
                    if ok {
                        if prev_pos >= state.pos { panic!("inline rule didn't increment state.pos"); }
                        break;
                    }
                }
            }

            if ok {
                if state.pos >= end { break; }
                continue;
            }

            let ch = state.src[state.pos..state.pos_max].chars().next().unwrap();
            let len = ch.len_utf8();
            state.trailing_text_push(state.pos, state.pos + len);
            state.pos += len;
        }

        for rule in self.ruler2.iter() {
            rule(state);
        }

        state.env.state_pop::<InlineLvl>();
    }

    // Process input string and push inline tokens into `out_tokens`
    //
    pub fn parse(&self, src: String, srcmap: Vec<(usize, usize)>, md: &MarkdownIt, env: &mut Env) -> Node {
        let node = Node::new(Root);
        let mut state = State::new(src, srcmap, md, env, node);
        state.env.state_push::<Inline>();
        self.tokenize(&mut state);
        state.env.state_pop::<Inline>();
        state.node
    }
}
