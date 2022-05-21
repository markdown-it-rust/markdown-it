// Core state object
//
use crate::Env;
use crate::MarkdownIt;
use crate::Token;
use std::collections::HashMap;

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
            env: HashMap::new(),
            tokens: Vec::new(),
            inline_mode: false,
            md,
        }
    }
}