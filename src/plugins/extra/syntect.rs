//! Syntax highlighting for code blocks
use anyhow::anyhow;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

use crate::parser::core::CoreRule;
use crate::parser::extset::MarkdownItExt;
use crate::plugins::cmark::block::code::CodeBlock;
use crate::plugins::cmark::block::fence::CodeFence;
use crate::{MarkdownIt, Node, NodeValue, Renderer, Result};

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
    fn try_run(root: &mut Node, md: &MarkdownIt) -> Result<()> {
        Self::_run::<true>(root, md)?;
        Ok(())
    }

    fn run(root: &mut Node, md: &MarkdownIt) {
        let _ = Self::_run::<false>(root, md);
    }
}

impl SyntectRule {
    fn _run<const CAN_FAIL: bool>(root: &mut Node, md: &MarkdownIt) -> Result<()> {
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
                    let language = language.trim();
                    if !language.is_empty() {
                        syntax = ss.find_syntax_by_token(language);

                        if CAN_FAIL && syntax.is_none() {
                            return Err(anyhow!("syntax not found for language `{language}`"));
                        }
                    }
                }

                let syntax = syntax.unwrap_or_else(|| ss.find_syntax_plain_text());
                let html = highlighted_html_for_string(content, &ss, syntax, theme);

                if let Ok(html) = html {
                    node.replace(SyntectSnippet { html });
                }
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use indoc::{indoc, formatdoc};

    fn run(input: &str) -> String {
        let md = &mut crate::MarkdownIt::new();
        crate::plugins::cmark::block::fence::add(md);
        crate::plugins::extra::syntect::add(md);
        let node = md.parse(&(input.to_owned() + "\n"));
        node.walk(|node, _| {
            assert!(node.srcmap.is_some());
            Ok(())
        }).unwrap();
        node.render()
    }

    fn try_run(input: &str) -> crate::Result<String> {
        let md = &mut crate::MarkdownIt::new();
        crate::plugins::cmark::block::fence::add(md);
        crate::plugins::extra::syntect::add(md);
        let node = md.try_parse(&(input.to_owned() + "\n"))?;
        node.walk(|node, _| {
            assert!(node.srcmap.is_some());
            Ok(())
        }).unwrap();
        Ok(node.render())
    }

    #[test]
    fn no_lang_prefix() {
        let input = indoc!(r#"
            ```
            hello
            ```
        "#);

        let output = indoc!(r#"
            <pre style="background-color:#ffffff;">
            <span style="color:#323232;">hello
            </span></pre>
        "#);

        assert_eq!(run(input), output);
        assert_eq!(try_run(input).ok().unwrap(), output);
    }

    #[test]
    fn rust_highlight() {
        let input = indoc!(r#"
            ```rust
            let hello = "world";
            ```
        "#);

        let output = indoc!(r#"
            <pre style="background-color:#ffffff;">
            <span style="font-weight:bold;color:#a71d5d;">let</span>
            <span style="color:#323232;"> hello </span>
            <span style="font-weight:bold;color:#a71d5d;">= </span>
            <span style="color:#183691;">&quot;world&quot;</span>
            <span style="color:#323232;">;</span>
            </pre>
        "#);

        assert_eq!(run(input).replace('\n', ""), output.replace('\n', ""));
        assert_eq!(try_run(input).ok().unwrap().replace('\n', ""), output.replace('\n', ""));
    }

    #[test]
    fn rust_highlight_trim_spaces() {
        let input = &formatdoc!(r#"
            ```  rust{}
            let hello = "world";
            ```
        "#, "  ");

        let output = indoc!(r#"
            <pre style="background-color:#ffffff;">
            <span style="font-weight:bold;color:#a71d5d;">let</span>
            <span style="color:#323232;"> hello </span>
            <span style="font-weight:bold;color:#a71d5d;">= </span>
            <span style="color:#183691;">&quot;world&quot;</span>
            <span style="color:#323232;">;</span>
            </pre>
        "#);

        assert_eq!(run(input).replace('\n', ""), output.replace('\n', ""));
        assert_eq!(try_run(input).ok().unwrap().replace('\n', ""), output.replace('\n', ""));
    }

    #[test]
    fn unknown_lang() {
        let input = indoc!(r#"
            ```some-unknown-language
            hello
            ```
        "#);

        let output = indoc!(r#"
            <pre style="background-color:#ffffff;">
            <span style="color:#323232;">hello
            </span></pre>
        "#);

        assert_eq!(run(input), output);
        assert!(
            format!("{:?}", try_run(input).err().unwrap()).contains("syntax not found for language")
        );
    }
}
