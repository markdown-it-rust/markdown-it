// Core state object
//
use crate::env::Env;
use crate::MarkdownIt;
use crate::Token;

#[derive(Debug)]
pub struct State<'a> {
    pub src: String,
    pub env: Env,
    pub tokens: Vec<Token>,
    pub inline_mode: bool,
    pub md: &'a MarkdownIt, // link to parser instance
}

impl<'a> State<'a> {
    pub fn new(src: &str, md: &'a MarkdownIt) -> Self {
        Self {
            src: src.to_owned(),
            env: Env::new(),
            tokens: Vec::new(),
            inline_mode: false,
            md,
        }
    }
}
