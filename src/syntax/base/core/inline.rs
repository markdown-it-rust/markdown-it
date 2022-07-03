use crate::MarkdownIt;
use crate::core;
use crate::renderer;
use crate::token::{Token, TokenData};

#[derive(Debug)]
pub struct InlineNodes {
    pub content: String
}

// this token is supposed to be replaced by one or many actual tokens by inline rule
impl TokenData for InlineNodes {
    fn render(&self, _: &Token, _: &mut renderer::Formatter) {
        unimplemented!()
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.core.ruler.add("inline", rule)
        .after("block");
}

pub fn rule(state: &mut core::State) {
    let mut tokens = std::mem::take(&mut state.tokens);
    walk(state, &mut tokens);
    state.tokens = tokens;
}

pub fn walk(state: &mut core::State, tokens: &mut Vec<Token>) {
    let mut idx = 0;
    while idx < tokens.len() {
        if let Some(data) = tokens[idx].data.downcast_ref::<InlineNodes>() {
            let mut children = Vec::new();
            state.md.inline.parse(&data.content, state.md, &mut state.env, &mut children);
            let len = children.len();
            tokens.splice(idx..idx+1, children);
            idx += len;
        } else {
            walk(state, &mut tokens[idx].children);
            idx += 1;
        }
    }
}
