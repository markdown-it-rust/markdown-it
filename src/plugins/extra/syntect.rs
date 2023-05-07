//! Syntax highlighting for code blocks
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

use crate::parser::core::CoreRule;
use crate::parser::extset::MarkdownItExt;
use crate::plugins::cmark::block::code::CodeBlock;
use crate::plugins::cmark::block::fence::CodeFence;
use crate::{MarkdownIt, Node, NodeValue, Renderer};

#[derive(Debug)]
pub struct SyntectSnippet {
    pub html: String,
}

impl NodeValue for SyntectSnippet {
    fn render(&self, _: &Node, fmt: &mut dyn Renderer) {
        fmt.text_raw(&self.html);
    }
}

#[derive(Debug, Clone, Copy)]
struct SyntectSettings(&'static str);
impl MarkdownItExt for SyntectSettings {}

impl Default for SyntectSettings {
    fn default() -> Self {
        Self("InspiredGitHub")
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.add_rule::<SyntectRule>();
}

pub fn set_theme(md: &mut MarkdownIt, theme: &'static str) {
    md.ext.insert(SyntectSettings(theme));
}

pub struct SyntectRule;
impl CoreRule for SyntectRule {
    fn run(root: &mut Node, md: &MarkdownIt) {
        let ss = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let theme = &ts.themes[md.ext.get::<SyntectSettings>().copied().unwrap_or_default().0];

        root.walk_mut(|node, _| {
            let mut content = None;
            let mut language = None;

            if let Some(data) = node.cast::<CodeBlock>() {
                content = Some(&data.content);
            } else if let Some(data) = node.cast::<CodeFence>() {
                language = Some(data.info.clone());
                content = Some(&data.content);
            }

            if let Some(content) = content {
                let mut syntax = None;
                if let Some(language) = language {
                    syntax = ss.find_syntax_by_token(&language);
                }
                let syntax = syntax.unwrap_or_else(|| ss.find_syntax_plain_text());

                let html = highlighted_html_for_string(content, &ss, syntax, theme);

                if let Ok(html) = html {
                    node.replace(SyntectSnippet { html });
                }
            }
        });
    }
}
