use crate::MarkdownIt;
use crate::core::State;
use crate::token::Token;

pub fn add(md: &mut MarkdownIt) {
    md.core.ruler.push("block", rule);
}

fn rule(state: &mut State) {
    if state.inline_mode {
        let mut token = Token::new("inline", "", 0);
        token.content = state.src.clone();
        token.map = Some([0, 1]);
        state.tokens.push(token);
    } else {
        state.md.block.parse(&state.src, state.md, &mut state.env, &mut state.tokens);
    }
}
