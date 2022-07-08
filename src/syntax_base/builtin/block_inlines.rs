use crate::Formatter;
use crate::MarkdownIt;
use crate::block;
use crate::token::{Token, TokenData};

#[derive(Debug)]
pub struct InlineNodes {
    pub content: String,
    pub mapping: Vec<(usize, usize)>,
}

// this token is supposed to be replaced by one or many actual tokens by inline rule
impl TokenData for InlineNodes {
    fn render(&self, _: &Token, _: &mut dyn Formatter) {
        unimplemented!()
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.block.ruler2.add("builtin::inline", rule)
        .before_all();
}

pub fn rule(state: &mut block::State) {
    let mut tokens = std::mem::take(state.tokens);
    walk(state, &mut tokens);
    *state.tokens = tokens;
}

pub fn walk(state: &mut block::State, tokens: &mut Vec<Token>) {
    let mut idx = 0;
    while idx < tokens.len() {
        if let Some(data) = tokens[idx].data.downcast_ref::<InlineNodes>() {
            let mut children = Vec::new();
            state.md.inline.parse(&data.content, state.md, state.env, &mut children);
            let len = children.len();
            tokens.splice(idx..idx+1, children);
            idx += len;
        } else {
            walk(state, &mut tokens[idx].children);
            idx += 1;
        }
    }
}
