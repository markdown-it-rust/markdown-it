// HTML block
//
use crate::MarkdownIt;
use crate::block::State;
use crate::common::html_re::HTML_OPEN_CLOSE_TAG_RE;
use crate::common::html_blocks::HTML_BLOCKS;
use once_cell::sync::Lazy;
use regex::Regex;

pub fn add(md: &mut MarkdownIt) {
    md.block.ruler.add("html_block", rule);
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
static HTML_SEQUENCES : Lazy<Vec<HTMLSequence>> = Lazy::new(|| {
    let mut result = Vec::new();

    result.push(HTMLSequence::new(
        Regex::new(r#"(?i)^<(script|pre|style|textarea)(\s|>|$)"#).unwrap(),
        Regex::new(r#"(?i)</(script|pre|style|textarea)>"#).unwrap(),
        true
    ));

    result.push(HTMLSequence::new(
        Regex::new(r#"^<!--"#).unwrap(),
        Regex::new(r#"-->"#).unwrap(),
        true
    ));

    result.push(HTMLSequence::new(
        Regex::new(r#"^<\?"#).unwrap(),
        Regex::new(r#"\?>"#).unwrap(),
        true
    ));

    result.push(HTMLSequence::new(
        Regex::new(r#"^<![A-Z]"#).unwrap(),
        Regex::new(r#">"#).unwrap(),
        true
    ));

    result.push(HTMLSequence::new(
        Regex::new(r#"^<!\[CDATA\["#).unwrap(),
        Regex::new(r#"\]\]>"#).unwrap(),
        true
    ));

    let block_names = HTML_BLOCKS.join("|");
    result.push(HTMLSequence::new(
        Regex::new(&format!("(?i)^</?({block_names})(\\s|/?>|$)")).unwrap(),
        Regex::new(r#"^$"#).unwrap(),
        true
    ));

    let open_close_tag_re = HTML_OPEN_CLOSE_TAG_RE.as_str();
    result.push(HTMLSequence::new(
        Regex::new(&format!("{open_close_tag_re}\\s*$")).unwrap(),
        Regex::new(r#"^$"#).unwrap(),
        false
    ));

    result
});

fn rule(state: &mut State, silent: bool) -> bool {
    // if it's indented more than 3 spaces, it should be a code block
    if (state.s_count[state.line] - state.blk_indent as i32) >= 4 { return false; }

    let pos = state.b_marks[state.line] + state.t_shift[state.line];
    let max = state.e_marks[state.line];
    let line_text = &state.src[pos..max];

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
            if state.s_count[next_line] < state.blk_indent as i32 { break; }

            let pos = state.b_marks[next_line] + state.t_shift[next_line];
            let max = state.e_marks[next_line];
            let line_text = &state.src[pos..max];

            if sequence.close.is_match(line_text) {
                if !line_text.is_empty() { next_line += 1; }
                break;
            }

            next_line += 1;
        }
    }

    state.line = next_line;

    let content = state.get_lines(start_line, next_line, state.blk_indent, true);
    let token = state.push("html_block", "", 0);
    token.map = Some([ start_line, next_line ]);
    token.content = content;

    true
}
