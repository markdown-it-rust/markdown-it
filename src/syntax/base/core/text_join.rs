use crate::MarkdownIt;
use crate::core::State;
use std::mem;

pub fn add(md: &mut MarkdownIt) {
    md.core.ruler.add("text_join", rule)
        .after_all();
}

// Join raw text tokens with the rest of the text
//
// This is set as a separate rule to provide an opportunity for plugins
// to run text replacements after text join, but before escape join.
//
// For example, `\:)` shouldn't be replaced with an emoji.
//

fn rule(state: &mut State) {
    for block_token in &mut state.tokens {
        if block_token.name != "inline" { continue; }

        for token in &mut block_token.children {
            if token.name == "text_special" {
                token.name = "text";
            }
        }

        let tokens = &mut block_token.children;
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
}
