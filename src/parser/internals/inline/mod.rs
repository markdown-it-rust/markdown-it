// Tokenizes paragraph content.
//
pub mod state;
pub use state::State;

use crate::Node;
use crate::parser::{MarkdownIt, rule_builder};
use crate::parser::internals::erasedset::{ErasedSet, TypeKey};
use crate::parser::internals::ruler::Ruler;

type RuleFn = fn (&mut State, bool) -> bool;

#[derive(Debug, Default)]
pub struct InlineParser {
    // [[Ruler]] instance. Keep configuration of inline rules.
    ruler: Ruler<TypeKey, RuleFn>,
}

impl InlineParser {
    pub fn new() -> Self {
        Self::default()
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
                    assert!(state.pos > pos, "inline rule didn't increment state.pos");
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
                        assert!(state.pos > prev_pos, "inline rule didn't increment state.pos");
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
    }

    // Process input string and push inline tokens into `out_tokens`
    //
    pub fn parse(&self, src: String, srcmap: Vec<(usize, usize)>, node: Node, md: &MarkdownIt, env: &mut ErasedSet) -> Node {
        let mut state = State::new(src, srcmap, md, env, node);
        self.tokenize(&mut state);
        state.node
    }

    pub fn add_rule<T: InlineRule>(&mut self) -> RuleBuilder<RuleFn> {
        let item = self.ruler.add(TypeKey::of::<T>(), T::run);
        RuleBuilder::new(item)
    }

    pub fn has_rule<T: InlineRule>(&mut self) -> bool {
        self.ruler.contains(TypeKey::of::<T>())
    }

    pub fn remove_rule<T: InlineRule>(&mut self) {
        self.ruler.remove(TypeKey::of::<T>());
    }
}

pub trait InlineRule : 'static {
    const MARKER: char;
    fn run(state: &mut State, silent: bool) -> bool;
}

rule_builder!(InlineRule);
