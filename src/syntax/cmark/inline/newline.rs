// Process '\n'
//
use crate::MarkdownIt;
use crate::inline::State;

pub fn add(md: &mut MarkdownIt) {
    md.inline.ruler.add("newline", rule);
}

fn rule(state: &mut State, silent: bool) -> bool {
    let mut chars = state.src[state.pos..state.pos_max].chars();

    if chars.next().unwrap() != '\n' { return false; }

    state.pos += 1;

    // skip leading whitespaces from next line
    while let Some(' ' | '\t') = chars.next() {
        state.pos += 1;
    }

    // '  \n' -> hardbreak
    if !silent {
        let mut tail_size = 0;

        loop {
            match state.pending.pop() {
                Some(' ') => tail_size += 1,
                Some(ch) => {
                    state.pending.push(ch);
                    break;
                }
                None => break,
            }
        }

        if tail_size >= 2 {
            state.push("hardbreak", "br", 0);
        } else {
            state.push("softbreak", "br", 0);
        }
    }

    true
}
