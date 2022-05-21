// Top-level rules executor. Glues block/inline parsers and does intermediate
// transformations.
//
mod state;
pub use state::State;

use crate::ruler::Ruler;
pub mod rules;

pub type Rule = fn (&mut State);

#[derive(Debug)]
pub struct Parser {
    // [[Ruler]] instance. Keep configuration of core rules.
    ruler: Ruler<Rule>,
}

impl Parser {
    pub fn new() -> Self {
        let mut result = Self { ruler: Ruler::new() };
        result.ruler.push("normalize",    rules::normalize::rule);
        result.ruler.push("block",        rules::block::rule);
        result.ruler.push("inline",       rules::inline::rule);
        //result.ruler.push("linkify",      rules::linkify::rule);
        //result.ruler.push("replacements", rules::replacements::rule);
        //result.ruler.push("smartquotes",  rules::smartquotes::rule);
        // `text_join` finds `text_special` tokens (for escape sequences)
        // and joins them with the rest of the text
        result.ruler.push("text_join",    rules::text_join::rule);
        result.ruler.compile();
        result
    }

    // Executes core chain rules.
    //
    pub fn process(&self, state: &mut State) {
        for rule in self.ruler.get_rules("") {
            rule(state);
        }
    }
}
