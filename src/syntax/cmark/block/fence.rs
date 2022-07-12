// fences (``` lang, ~~~ lang)
//
use crate::{Node, NodeValue, Renderer};
use crate::parser::MarkdownIt;
use crate::parser::internals::block;
use crate::parser::internals::common::unescape_all;

#[derive(Debug)]
pub struct CodeFence {
    pub info: String,
    pub marker: char,
    pub marker_len: usize,
    pub content: String,
    pub lang_prefix: &'static str,
}

impl NodeValue for CodeFence {
    fn render(&self, _: &Node, fmt: &mut dyn Renderer) {
        let info = unescape_all(&self.info);
        let mut split = info.split_whitespace();
        let lang_name = split.next().unwrap_or("");
        let mut attrs = Vec::new();
        let class;

        if !lang_name.is_empty() {
            class = format!("{}{}", self.lang_prefix, lang_name);
            attrs.push(("class", class.as_str()));
        }

        fmt.cr();
        fmt.open("pre", &[]);
            fmt.open("code", &attrs);
            fmt.text(&self.content);
            fmt.close("code");
        fmt.close("pre");
        fmt.cr();
    }

    fn render2(&self, node: &Node) -> crate::Html {
        let info = unescape_all(&self.info);
        let mut split = info.split_whitespace();
        let lang_name = split.next().unwrap_or("");
        let mut attrs = Vec::new();
        let class;

        if !lang_name.is_empty() {
            class = format!("{}{}", self.lang_prefix, lang_name);
            attrs.push(("class", class));
        }

        crate::Html::Element(crate::HtmlElement {
            tag: "pre",
            attrs: vec![],
            children: Some(vec![
                crate::Html::Element(crate::HtmlElement {
                    tag: "code",
                    attrs,
                    children: Some(vec![crate::Html::Text(self.content.clone())]),
                    spacing: crate::HtmlSpacing::None,
                })
            ]),
            spacing: crate::HtmlSpacing::Around,
        })
    }
}

#[derive(Debug, Default)]
struct FenceSettings(std::cell::Cell<&'static str>);

pub fn add(md: &mut MarkdownIt) {
    add_with_lang_prefix(md, "language-");
}

pub fn add_with_lang_prefix(md: &mut MarkdownIt, lang_prefix: &'static str) {
    md.block.ruler.add("fence", rule);
    md.env.get_or_insert_default::<FenceSettings>().0.set(lang_prefix);
}

fn rule(state: &mut block::State, silent: bool) -> bool {
    // if it's indented more than 3 spaces, it should be a code block
    if state.line_indent(state.line) >= 4 { return false; }

    let line = state.get_line(state.line);
    let mut chars = line.chars();
    let marker;

    if let Some(ch @ ('~' | '`')) = chars.next() {
        marker = ch;
    } else {
        return false;
    }

    // scan marker length
    let mut len = 1;
    while Some(marker) == chars.next() { len += 1; }

    if len < 3 { return false; }

    let params = &line[len..];

    if marker == '`' && params.contains(marker) { return false; }

    // Since start is found, we can report success here in validation mode
    if silent { return true; }

    let start_line = state.line;
    let mut next_line = state.line;
    let mut have_end_marker = false;

    // search end of block
    'outer: loop {
        next_line += 1;
        if next_line >= state.line_max {
            // unclosed block should be autoclosed by end of document.
            // also block seems to be autoclosed by end of parent
            break;
        }

        let line = state.get_line(next_line);

        if !line.is_empty() && state.line_indent(next_line) < 0 {
            // non-empty line with negative indent should stop the list:
            // - ```
            //  test
            break;
        }

        let mut chars = line.chars().peekable();

        if Some(marker) != chars.next() { continue; }

        if state.line_indent(next_line) >= 4 {
            // closing fence should be indented less than 4 spaces
            continue;
        }

        // scan marker length
        let mut len_end = 1;
        while Some(&marker) == chars.peek() {
            chars.next();
            len_end += 1;
        }

        // closing code fence must be at least as long as the opening one
        if len_end < len { continue; }

        // make sure tail has spaces only
        loop {
            match chars.next() {
                Some(' ' | '\t') => {},
                Some(_) => continue 'outer,
                None => {
                    have_end_marker = true;
                    break 'outer;
                }
            }
        }
    }

    // If a fence has heading spaces, they should be removed from its inner block
    let indent = state.line_offsets[start_line].indent_nonspace;
    let (content, _) = state.get_lines(start_line + 1, next_line, indent as usize, true);
    let params = params.to_owned();

    let lang_prefix = state.md.env.get::<FenceSettings>().unwrap().0.get();
    let mut node = Node::new(CodeFence {
        info: params,
        marker,
        marker_len: len,
        content,
        lang_prefix,
    });
    node.srcmap = state.get_map(start_line, next_line - if have_end_marker { 0 } else { 1 });
    state.push(node);

    state.line = next_line + if have_end_marker { 1 } else { 0 };

    true
}
