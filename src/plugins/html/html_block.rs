// HTML block
//
use once_cell::sync::Lazy;
use regex::Regex;
use crate::{MarkdownIt, Node, NodeValue, Renderer};
use crate::parser::block::{BlockRule, BlockState};
use super::utils::blocks::*;
use super::utils::regexps::*;

#[derive(Debug)]
pub struct HtmlBlock {
    pub content: String,
}

impl NodeValue for HtmlBlock {
    fn render(&self, _: &Node, fmt: &mut dyn Renderer) {
        fmt.cr();
        fmt.text_raw(&self.content);
        fmt.cr();
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.block.add_rule::<HtmlBlockScanner>();
}

struct HTMLSequence {
    open: Regex,
    close: Regex,
    can_terminate_paragraph: bool,
}

impl HTMLSequence {
    pub fn new(open: Regex, close: Regex, can_terminate_paragraph: bool) -> Self {
        Self { open, close, can_terminate_paragraph }
    }
}

// An array of opening and corresponding closing sequences for html tags,
// last argument defines whether it can terminate a paragraph or not
//
static HTML_SEQUENCES : Lazy<[HTMLSequence; 7]> = Lazy::new(|| {
    let block_names = HTML_BLOCKS.join("|");
    let open_close_tag_re = HTML_OPEN_CLOSE_TAG_RE.as_str();

    [
        HTMLSequence::new(
            Regex::new(r#"(?i)^<(script|pre|style|textarea)(\s|>|$)"#).unwrap(),
            Regex::new(r#"(?i)</(script|pre|style|textarea)>"#).unwrap(),
            true
        ),

        HTMLSequence::new(
            Regex::new(r#"^<!--"#).unwrap(),
            Regex::new(r#"-->"#).unwrap(),
            true
        ),

        HTMLSequence::new(
            Regex::new(r#"^<\?"#).unwrap(),
            Regex::new(r#"\?>"#).unwrap(),
            true
        ),

        HTMLSequence::new(
            Regex::new(r#"^<![A-Z]"#).unwrap(),
            Regex::new(r#">"#).unwrap(),
            true
        ),

        HTMLSequence::new(
            Regex::new(r#"^<!\[CDATA\["#).unwrap(),
            Regex::new(r#"\]\]>"#).unwrap(),
            true
        ),

        HTMLSequence::new(
            Regex::new(&format!("(?i)^</?({block_names})(\\s|/?>|$)")).unwrap(),
            Regex::new(r#"^$"#).unwrap(),
            true
        ),

        HTMLSequence::new(
            Regex::new(&format!("{open_close_tag_re}\\s*$")).unwrap(),
            Regex::new(r#"^$"#).unwrap(),
            false
        ),
    ]
});

pub struct HtmlBlockScanner;
impl BlockRule for HtmlBlockScanner {
    fn run(state: &mut BlockState, silent: bool) -> bool {
        // if it's indented more than 3 spaces, it should be a code block
        if state.line_indent(state.line) >= 4 { return false; }

        let line_text = state.get_line(state.line);

        if let Some('<') = line_text.chars().next() {} else { return false; }

        let mut sequence = None;
        for seq in HTML_SEQUENCES.iter() {
            if seq.open.is_match(line_text) {
                sequence = Some(seq);
                break;
            }
        }

        if sequence.is_none() { return false; }
        let sequence = sequence.unwrap();

        if silent {
            // true if this sequence can be a terminator, false otherwise
            return sequence.can_terminate_paragraph;
        }

        let start_line = state.line;
        let mut next_line = state.line + 1;

        // If we are here - we detected HTML block.
        // Let's roll down till block end.
        if !sequence.close.is_match(line_text) {
            while next_line < state.line_max {
                if state.line_indent(next_line) < 0 { break; }

                let line_text = state.get_line(next_line);

                if sequence.close.is_match(line_text) {
                    if !line_text.is_empty() { next_line += 1; }
                    break;
                }

                next_line += 1;
            }
        }

        state.line = next_line;

        let (content, _) = state.get_lines(start_line, next_line, state.blk_indent, true);
        let mut node = Node::new(HtmlBlock { content });
        node.srcmap = state.get_map(start_line, next_line - 1);
        state.push(node);

        true
    }
}
