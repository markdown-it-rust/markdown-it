use crate::MarkdownIt;
use crate::core;

pub fn add(md: &mut MarkdownIt) {
    md.core.ruler.add("block", rule);
}

fn rule(state: &mut core::State) {
    state.md.block.parse(&state.src, state.md, &mut state.env, &mut state.tokens);
}
