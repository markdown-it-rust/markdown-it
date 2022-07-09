use crate::Formatter;
use crate::parser::internals::block;
use crate::parser::MarkdownIt;
use crate::{Node, NodeValue};

#[derive(Debug)]
pub struct InlineNodes {
    pub content: String,
    pub mapping: Vec<(usize, usize)>,
}

// this token is supposed to be replaced by one or many actual tokens by inline rule
impl NodeValue for InlineNodes {
    fn render(&self, _: &Node, _: &mut dyn Formatter) {
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

pub fn walk(state: &mut block::State, tokens: &mut Vec<Node>) {
    let mut idx = 0;
    while idx < tokens.len() {
        // TODO: generic walk
        if let Some(data) = tokens[idx].cast_mut::<InlineNodes>() {
            let mut children = Vec::new();
            let content = std::mem::take(&mut data.content);
            let mapping = std::mem::take(&mut data.mapping);
            state.md.inline.parse(content, mapping, state.md, state.env, &mut children);
            let len = children.len();
            tokens.splice(idx..idx+1, children);
            idx += len;
        } else {
            walk(state, &mut tokens[idx].children);
            idx += 1;
        }
    }
}
