// Process [link](<to> "stuff")
//
use crate::MarkdownIt;
use crate::inline::State;
use crate::helpers;

pub fn add(md: &mut MarkdownIt) {
    md.inline.ruler.add("link", rule);
}

fn rule(state: &mut State, silent: bool) -> bool {
    if state.src[state.pos..state.pos_max].chars().next().unwrap() != '[' { return false; }

    if let Some(result) = helpers::parse_link(state, state.pos, false) {
        //
        // We found the end of the link, and know for a fact it's a valid link;
        // so all that's left to do is to call tokenizer.
        //
        if !silent {
            if !state.pending.is_empty() { state.push_pending(); }

            let old_tokens = std::mem::take(state.tokens);
            let max = state.pos_max;

            state.link_level += 1;
            state.pos = result.label_start;
            state.pos_max = result.label_end;
            state.md.inline.tokenize(state);
            state.pos_max = max;

            let children = std::mem::replace(state.tokens, old_tokens);

            let token = state.push("link", "a", 0);
            token.attrs.push(("href", result.href.unwrap_or_default()));
            if let Some(x) = result.title {
                token.attrs.push(("title", x));
            }
            token.children = children;
            state.link_level -= 1;
        }

        state.pos = result.end;
        true
    } else {
        false
    }
}
