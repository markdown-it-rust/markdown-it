// Top-level rules executor. Glues block/inline parsers and does intermediate
// transformations.
//
mod state;
pub use state::State;

use crate::rulers::ruler::Ruler;

pub type Rule = fn (&mut State);

#[derive(Debug)]
pub struct Parser {
    // [[Ruler]] instance. Keep configuration of core rules.
    pub ruler: Ruler<&'static str, Rule>,
}

impl Parser {
    pub fn new() -> Self {
        Self { ruler: Ruler::new() }
    }

    // Executes core chain rules.
    //
    pub fn process(&self, state: &mut State) {
        for rule in self.ruler.iter() {
            rule(state);
        }
    }
}
