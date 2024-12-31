//! Syntax highlighting for code blocks
use syntect::highlighting::ThemeSet;
pub use syntect::html::ClassStyle;
use syntect::html::{highlighted_html_for_string, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

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
enum SyntectMode {
    InlineStyles { theme: &'static str },
    CssClasses { class_style: ClassStyle },
}

#[derive(Debug, Clone, Copy)]
struct SyntectSettings(SyntectMode);
impl MarkdownItExt for SyntectSettings {}

impl Default for SyntectSettings {
    fn default() -> Self {
        Self(SyntectMode::InlineStyles {
            theme: "InspiredGitHub",
        })
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.add_rule::<SyntectRule>();
}

/// Use inline styles with the given theme
pub fn set_theme(md: &mut MarkdownIt, theme: &'static str) {
    md.ext
        .insert(SyntectSettings(SyntectMode::InlineStyles { theme }));
}

/// Generate spans with css classes applied
///
/// This is an alternative to using a theme. You are responsible for including a style sheet.
pub fn use_css_classes(md: &mut MarkdownIt, class_style: ClassStyle) {
    md.ext
        .insert(SyntectSettings(SyntectMode::CssClasses { class_style }));
}

pub struct SyntectRule;
impl CoreRule for SyntectRule {
    fn run(root: &mut Node, md: &MarkdownIt) {
        let ss = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let mode = md
            .ext
            .get::<SyntectSettings>()
            .copied()
            .unwrap_or_default()
            .0;

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

                let html = match mode {
                    SyntectMode::InlineStyles { theme } => {
                        highlighted_html_for_string(content, &ss, syntax, &ts.themes[theme])
                    }
                    SyntectMode::CssClasses { class_style } => {
                        let mut html_generator =
                            ClassedHTMLGenerator::new_with_class_style(syntax, &ss, class_style);
                        for line in LinesWithEndings::from(content) {
                            if let Err(_) =
                                html_generator.parse_html_for_line_which_includes_newline(line)
                            {
                                return;
                            };
                        }
                        Ok(html_generator.finalize())
                    }
                };

                if let Ok(html) = html {
                    node.replace(SyntectSnippet { html });
                }
            }
        });
    }
}
