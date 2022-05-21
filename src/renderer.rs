/**
 * class Renderer
 *
 * Generates HTML from parsed token stream. Each instance has independent
 * copy of rules. Those can be rewritten with ease. Also, you can add new
 * rules if you create plugin and adds new token types.
 **/
use crate::common::escape_html;
use crate::common::unescape_all;
use crate::token::Token;
use crate::token::TokenAttrs;
use crate::MarkdownIt;
use derivative::Derivative;
use std::collections::HashMap;

pub type Rule = fn (tokens: &Vec<Token>, idx: usize, md: &MarkdownIt) -> String;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Renderer {
    /*
     * Renderer#rules -> Object
     *
     * Contains render rules for tokens. Can be updated and extended.
     *
     * ##### Example
     *
     * ```javascript
     * var md = require('markdown-it')();
     *
     * md.renderer.rules.strong_open  = function () { return '<b>'; };
     * md.renderer.rules.strong_close = function () { return '</b>'; };
     *
     * var result = md.renderInline(...);
     * ```
     *
     * Each rule is called as independent static function with fixed signature:
     *
     * ```javascript
     * function my_token_render(tokens, idx, options, env, renderer) {
     *   // ...
     *   return renderedHTML;
     * }
     * ```
     *
     * See [source code](https://github.com/markdown-it/markdown-it/blob/master/lib/renderer.js)
     * for more details and examples.
     */
    #[derivative(Debug="ignore")]
    rules: HashMap<&'static str, Rule>,
}

impl Renderer {
    // Creates new [[Renderer]] instance and fill [[Renderer#rules]] with defaults.
    pub fn new() -> Self {
        let mut result = Self { rules: HashMap::new() };
        result.rules.insert("code_inline", rules::code_inline);
        result.rules.insert("code_block",  rules::code_block);
        result.rules.insert("fence",       rules::fence);
        result.rules.insert("image",       rules::image);
        result.rules.insert("hardbreak",   rules::hardbreak);
        result.rules.insert("softbreak",   rules::softbreak);
        result.rules.insert("text",        rules::text);
        result.rules.insert("html_block",  rules::html_block);
        result.rules.insert("html_inline", rules::html_inline);
        result
    }

    /**
     * Renderer.renderAttrs(token) -> String
     *
     * Render token attributes to string.
     **/
    pub fn render_attrs(&self, attrs: &TokenAttrs) -> String {
        let mut result = String::new();

        for (name, value) in attrs {
            let escaped_name = escape_html(name);
            let escaped_value = escape_html(&value);
            result += &format!(" {escaped_name}=\"{escaped_value}\"");
        }

        result
    }

    /**
     * Renderer.renderToken(tokens, idx, options) -> String
     * - tokens (Array): list of tokens
     * - idx (Numbed): token index to render
     * - options (Object): params of parser instance
     *
     * Default token renderer. Can be overriden by custom function
     * in [[Renderer#rules]].
     **/
    pub fn render_token(&self, tokens: &Vec<Token>, idx: usize, md: &MarkdownIt) -> String {
        let mut result = String::new();
        let token = &tokens[idx];
        let mut need_lf = false;

        // Tight list paragraphs
        if token.hidden { return result; }

        // Insert a newline between hidden paragraph and subsequent opening
        // block-level tag.
        //
        // For example, here we should insert a newline before blockquote:
        //  - a
        //    >
        if token.block && token.nesting != -1 && idx != 0 && tokens[idx - 1].hidden {
            result.push('\n');
        }

        // Add token name, e.g. `<img`
        result.push('<');
        if token.nesting == -1 { result.push('/'); }
        result += token.tag;

        // Encode attributes, e.g. `<img src="foo"
        result += &self.render_attrs(&token.attrs);

        // Add a slash for self-closing tags, e.g. `<img src="foo" /`
        if token.nesting == 0 && md.options.xhtml_out {
            result += " /";
        }

        if token.block {
            need_lf = true;

            if token.nesting == 1 {
                if idx + 1 < tokens.len() {
                    let next_token = &tokens[idx + 1];

                    if next_token.name == "inline" || next_token.hidden {
                        // Block-level tag containing an inline tag.
                        need_lf = false;
                    } else if next_token.nesting == -1 && next_token.tag == token.tag {
                        // Opening tag + closing tag of the same type. E.g. `<li></li>`.
                        need_lf = false;
                    }
                }
            }
        }

        result.push('>');
        if need_lf { result.push('\n'); }

        result
    }

    /**
     * Renderer.renderInline(tokens, options, env) -> String
     * - tokens (Array): list on block tokens to render
     * - options (Object): params of parser instance
     *
     * The same as [[Renderer.render]], but for single token of `inline` type.
     **/
    pub fn render_inline(&self, tokens: &Vec<Token>, md: &MarkdownIt) -> String {
        let mut result = String::new();

        for (idx, token) in tokens.iter().enumerate() {
            if let Some(rule) = self.rules.get(token.name) {
                result += &rule(&tokens, idx, md);
            } else {
                result += &self.render_token(&tokens, idx, md);
            }
        }

        result
    }

    /** internal
     * Renderer.renderInlineAsText(tokens, options, env) -> String
     * - tokens (Array): list on block tokens to render
     * - options (Object): params of parser instance
     * - env (Object): additional data from parsed input (references, for example)
     *
     * Special kludge for image `alt` attributes to conform CommonMark spec.
     * Don't try to use it! Spec requires to show `alt` content with stripped markup,
     * instead of simple escaping.
     **/
    pub fn render_inline_as_text(&self, tokens: &Vec<Token>, md: &MarkdownIt) -> String {
        let mut result = String::new();

        for token in tokens.iter() {
            match token.name {
                "text" => result += &token.content,
                "image" => result += &self.render_inline_as_text(&token.children, md),
                "softbreak" => result += "\n",
                _ => {}
            }
        }

        result
    }

    /**
     * Renderer.render(tokens, options, env) -> String
     * - tokens (Array): list on block tokens to render
     * - options (Object): params of parser instance
     *
     * Takes token stream and generates HTML. Probably, you will never need to call
     * this method directly.
     **/
    pub fn render(&self, tokens: &Vec<Token>, md: &MarkdownIt) -> String {
        let mut result = String::new();

        for (idx, token) in tokens.iter().enumerate() {
            if token.name == "inline" {
                result += &self.render_inline(&token.children, md);
            } else {
                if let Some(rule) = self.rules.get(token.name) {
                    result += &rule(&tokens, idx, md);
                } else {
                    result += &self.render_token(&tokens, idx, md);
                }
            }
        }

        result
    }
}

mod rules {
    use crate::MarkdownIt;
    use crate::token::Token;
    use crate::token::TokenAttrs;
    use super::escape_html;
    use super::unescape_all;

    pub fn code_inline(tokens: &Vec<Token>, idx: usize, md: &MarkdownIt) -> String {
        let token = &tokens[idx];
        let attrs = md.renderer.render_attrs(&token.attrs);
        let content = escape_html(&token.content);

        format!("<code{attrs}>{content}</code>")
    }

    pub fn code_block(tokens: &Vec<Token>, idx: usize, md: &MarkdownIt) -> String {
        let token = &tokens[idx];
        let attrs = md.renderer.render_attrs(&token.attrs);
        let content = escape_html(&token.content);

        format!("<pre><code{attrs}>{content}</code></pre>\n")
    }

    pub fn fence(tokens: &Vec<Token>, idx: usize, md: &MarkdownIt) -> String {
        let token = &tokens[idx];
        let info = unescape_all(&token.info);
        let mut split = info.split_whitespace();
        let lang_name = split.next().unwrap_or("");
        //let lang_attrs = split.collect::<Vec<&str>>().join(" ");

        // TODO: highlight
        let mut highlighted = escape_html(&token.content);

        if highlighted.starts_with("<pre") {
            highlighted.push('\n');
            return highlighted;
        }

        let attrs;

        // If language exists, inject class gently, without modifying original token.
        // May be, one day we will add .deepClone() for token and simplify this part, but
        // now we prefer to keep things local.
        if !lang_name.is_empty() {
            let lang_prefix = md.options.lang_prefix;
            let mut has_class = false;
            let mut attrs_with_class = token.attrs.iter().map(
                |(k, v)| if *k == "class" {
                    has_class = true;
                    (*k, format!("{v} {lang_prefix}{lang_name}"))
                } else {
                    (*k, v.clone())
                }
            ).collect::<TokenAttrs>();

            if !has_class {
                attrs_with_class.push(("class", format!("{lang_prefix}{lang_name}")));
            }

            attrs = md.renderer.render_attrs(&attrs_with_class);
        } else {
            attrs = md.renderer.render_attrs(&token.attrs);
        }

        format!("<pre><code{attrs}>{highlighted}</code></pre>\n")
    }

    pub fn image(tokens: &Vec<Token>, idx: usize, md: &MarkdownIt) -> String {
        let token = &tokens[idx];

        // "alt" attr MUST be set, even if empty. Because it's mandatory and
        // should be placed on proper position for tests.
        //
        // Replace content with actual value

        let attrs_with_alt = token.attrs.iter().map(
            |(k, v)| if *k == "alt" {
                (*k, md.renderer.render_inline_as_text(&token.children, md))
            } else {
                (*k, v.clone())
            }
        ).collect();

        let attrs = md.renderer.render_attrs(&attrs_with_alt);
        let xhtml = if md.options.xhtml_out { " /" } else { "" };

        format!("<img{attrs}{xhtml}>")
    }

    pub fn hardbreak(_tokens: &Vec<Token>, _idx: usize, md: &MarkdownIt) -> String {
        if md.options.xhtml_out {
            "<br />\n".to_owned()
        } else {
            "<br>\n".to_owned()
        }
    }

    pub fn softbreak(_tokens: &Vec<Token>, _idx: usize, md: &MarkdownIt) -> String {
        if !md.options.breaks {
            "\n".to_owned()
        } else if md.options.xhtml_out {
            "<br />\n".to_owned()
        } else {
            "<br>\n".to_owned()
        }
    }

    pub fn text(tokens: &Vec<Token>, idx: usize, _md: &MarkdownIt) -> String {
        escape_html(&tokens[idx].content)
    }

    pub fn html_block(tokens: &Vec<Token>, idx: usize, _md: &MarkdownIt) -> String {
        tokens[idx].content.to_owned()
    }

    pub fn html_inline(tokens: &Vec<Token>, idx: usize, _md: &MarkdownIt) -> String {
        tokens[idx].content.to_owned()
    }
}
