// Top-level rules executor. Glues block/inline parsers and does intermediate
// transformations.
//
mod state;
pub use state::State;

use crate::ruler::Ruler;
use crate::env::scope::{BlockLvl, Block};

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
        state.env.state_push::<Block>();
        state.env.state_push::<BlockLvl>();
        for rule in self.ruler.iter() {
            rule(state);
        }
        state.env.state_pop::<BlockLvl>();
        state.env.state_pop::<Block>();
    }
}
