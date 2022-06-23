use crate::MarkdownIt;
use crate::inline::State;
use std::mem;

// Clean up tokens after emphasis and strikethrough postprocessing:
// merge adjacent text nodes into one and re-calculate all token levels
//
// This is necessary because initially emphasis delimiter markers (*, _, ~)
// are treated as their own separate text tokens. Then emphasis rule either
// leaves them as text (needed to merge with adjacent text) or turns them
// into opening/closing tags (which messes up levels inside).
//

pub fn add(md: &mut MarkdownIt) {
    md.inline.ruler2.add("fragments_join", postprocess)
        .after_all();
}

fn postprocess(state: &mut State) {
    let tokens = &mut state.tokens;
    let mut curr = 0;
    let mut last = 0;
    let max = tokens.len();

    while curr < max {
        if tokens[curr].name == "text" && curr + 1 < max && tokens[curr + 1].name == "text" {
            // collapse two adjacent text nodes
            let second_token_content = mem::take(&mut tokens[curr + 1].content);
            tokens[curr].content += &second_token_content;
            tokens.swap(curr, curr + 1);
        } else {
            if curr != last { tokens.swap(last, curr); }
            last += 1;
        }
        curr += 1;
    }

    if curr != last { tokens.truncate(last); }
}
