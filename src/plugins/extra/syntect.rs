//! Syntax highlighting for code blocks
use crate::{MarkdownIt, Node, NodeValue, Renderer};
use crate::parser::extset::MarkdownItExt;
use crate::parser::core::CoreRule;
use crate::plugins::cmark::block::code::CodeBlock;
use crate::plugins::cmark::block::fence::CodeFence;

use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

#[derive(Debug)]
pub struct SyntectSnippet {
    pub html: String,
}

impl NodeValue for SyntectSnippet {
    fn render(&self, _: &Node, fmt: &mut dyn Renderer) {
        fmt.text_raw(&self.html);
    }
}

#[derive(Debug)]
struct SyntectSettings(&'static str);
impl MarkdownItExt for SyntectSettings {}

pub fn add(md: &mut MarkdownIt) {
    add_with_theme(md, "InspiredGitHub");
}

pub fn add_with_theme(md: &mut MarkdownIt, theme: &'static str) {
    md.add_rule::<SyntectRule>();
    md.ext.insert(SyntectSettings(theme));
}

pub struct SyntectRule;
impl CoreRule for SyntectRule {
    fn run(root: &mut Node, md: &MarkdownIt) {
        let ss = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let theme = &ts.themes[md.ext.get::<SyntectSettings>().unwrap().0];

        dbg!(&ts.themes.keys());

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
