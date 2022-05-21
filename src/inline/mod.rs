// Tokenizes paragraph content.
//
mod state;
pub use state::State;

use crate::Env;
use crate::MarkdownIt;
use crate::ruler::Ruler;
use crate::token::Token;
pub mod rules;

pub type Rule = fn (&mut State, silent: bool) -> bool;
pub type Rule2 = fn (&mut State);

#[derive(Debug)]
pub struct Parser {
    // [[Ruler]] instance. Keep configuration of inline rules.
    ruler: Ruler<Rule>,

    // [[Ruler]] instance. Second ruler used for post-processing
    // (e.g. in emphasis-like rules).
    ruler2: Ruler<Rule2>,
}

impl Parser {
    pub fn new() -> Self {
        let mut result = Self {
            ruler: Ruler::new(),
            ruler2: Ruler::new(),
        };

        result.ruler.push("text",            rules::text::rule);
        //result.ruler.push("linkify",         rules::linkify::rule);
        result.ruler.push("newline",         rules::newline::rule);
        result.ruler.push("escape",          rules::escape::rule);
        result.ruler.push("backticks",       rules::backticks::rule);
        result.ruler.push("strikethrough",   rules::strikethrough::rule);
        result.ruler.push("emphasis",        rules::emphasis::rule);
        result.ruler.push("link",            rules::link::rule);
        result.ruler.push("image",           rules::image::rule);
        result.ruler.push("autolink",        rules::autolink::rule);
        result.ruler.push("html_inline",     rules::html_inline::rule);
        result.ruler.push("entity",          rules::entity::rule);
        result.ruler.compile();

        // `rule2` ruleset was created specifically for emphasis/strikethrough
        // post-processing and may be changed in the future.
        //
        // Don't use this for anything except pairs (plugins working with `balance_pairs`).
        //
        result.ruler2.push("balance_pairs",  rules::balance_pairs::postprocess);
        result.ruler2.push("strikethrough",  rules::strikethrough::postprocess);
        result.ruler2.push("emphasis",       rules::emphasis::postprocess);
        // rules for pairs separate '**' into its own text tokens, which may be left unused,
        // rule below merges unused segments back with the rest of the text
        result.ruler2.push("fragments_join", rules::fragments_join::postprocess);
        result.ruler2.compile();

        result
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
            for rule in self.ruler.get_rules("") {
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
                for rule in self.ruler.get_rules("") {
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

        for rule in self.ruler2.get_rules("") {
            rule(&mut state);
        }
    }
}
