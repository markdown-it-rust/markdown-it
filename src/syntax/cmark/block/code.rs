// Code block (4 spaces padded)
//
use crate::{Node, NodeValue, Renderer};
use crate::parser::MarkdownIt;
use crate::parser::internals::block;

#[derive(Debug)]
pub struct CodeBlock {
    pub content: String,
}

impl NodeValue for CodeBlock {
    fn render(&self, _: &Node, fmt: &mut dyn Renderer) {
        fmt.cr();
        fmt.open("pre", &[]);
            fmt.open("code", &[]);
            fmt.text(&self.content);
            fmt.close("code");
        fmt.close("pre");
        fmt.cr();
    }

    fn render2(&self, node: &Node) -> crate::Html {
        crate::Html::Element(crate::HtmlElement {
            tag: "pre",
            attrs: vec![],
            children: Some(vec![
                crate::Html::Element(crate::HtmlElement {
                    tag: "code",
                    attrs: vec![],
                    children: Some(vec![crate::Html::Text(self.content.clone())]),
                    spacing: crate::HtmlSpacing::None,
                })
            ]),
            spacing: crate::HtmlSpacing::Around,
        })
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.block.ruler.add("code", rule);
}

fn rule(state: &mut block::State, silent: bool) -> bool {
    if silent { return false; }
    if state.line_indent(state.line) < 4 { return false; }

    let mut next_line = state.line + 1;
    let mut last = next_line;

    while next_line < state.line_max {
        if state.is_empty(next_line) {
            next_line += 1;
            continue;
        }

        if state.line_indent(next_line) >= 4 {
            next_line += 1;
            last = next_line;
            continue;
        }

        break;
    }

    let start_line = state.line;
    state.line = last;

    let (mut content, mapping) = state.get_lines(start_line, last, 4 + state.blk_indent, false);
    content += "\n";

    let mut node = Node::new(CodeBlock { content });
    node.srcmap = state.get_map_from_offsets(mapping[0].1, state.line_offsets[state.line - 1].line_end);
    state.push(node);

    true
}
