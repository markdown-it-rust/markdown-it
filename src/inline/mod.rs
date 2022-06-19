// Tokenizes paragraph content.
//
pub mod state;
pub use state::State;

use crate::Env;
use crate::MarkdownIt;
use crate::markvec::MarkVec;
use crate::token::Token;

pub type Rule = fn (&mut State, silent: bool) -> bool;
pub type Rule2 = fn (&mut State);

#[derive(Debug)]
pub struct Parser {
    // [[Ruler]] instance. Keep configuration of inline rules.
    pub ruler: MarkVec<&'static str, Rule>,

    // [[Ruler]] instance. Second ruler used for post-processing
    // (e.g. in emphasis-like rules).
    pub ruler2: MarkVec<&'static str, Rule2>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            ruler: MarkVec::new(),
            ruler2: MarkVec::new(),
        }
    }

    // Skip single token by running all rules in validation mode;
    // returns `true` if any rule reported success
    //
    pub fn skip_token(&self, state: &mut State) {
        let pos = state.pos;
        let mut ok = false;
        let max_nesting = state.md.options.max_nesting.unwrap_or(100);

        if let Some(x) = state.cache.get(&pos) {
            state.pos = *x;
            return;
        }

        if state.state_level < max_nesting {
            for (_, rule) in self.ruler.iter() {
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
        let end = state.pos_max;
        let max_nesting = state.md.options.max_nesting.unwrap_or(100);

        while state.pos < end {
            // Try all possible rules.
            // On success, rule should:
            //
            // - update `state.pos`
            // - update `state.tokens`
            // - return true
            let mut ok = false;
            let prev_pos = state.pos;

            if state.state_level < max_nesting {
                for (_, rule) in self.ruler.iter() {
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
            state.pending.push(ch);
            state.pos += ch.len_utf8();
        }

        if !state.pending.is_empty() { state.push_pending(); }
    }

    // Process input string and push inline tokens into `out_tokens`
    //
    pub fn parse(&self, src: &str, md: &MarkdownIt, env: &mut Env, out_tokens: &mut Vec<Token>, level: u32) {
        let mut state = State::new(src, md, env, out_tokens, level);
        self.tokenize(&mut state);

        for (_, rule) in self.ruler2.iter() {
            rule(&mut state);
        }
    }
}
