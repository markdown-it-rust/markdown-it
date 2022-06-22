// Process ![image](<src> "title")
//
use crate::MarkdownIt;
use crate::inline::State;
use crate::helpers;

pub fn add(md: &mut MarkdownIt) {
    md.inline.ruler.add("image", rule);
}

fn rule(state: &mut State, silent: bool) -> bool {
    let mut chars = state.src[state.pos..state.pos_max].chars();
    if chars.next().unwrap() != '!' { return false; }
    if let Some('[') = chars.next() {} else { return false; }

    if let Some(result) = helpers::parse_link(state, state.pos + 1, true) {
        //
        // We found the end of the link, and know for a fact it's a valid link;
        // so all that's left to do is to call tokenizer.
        //
        if !silent {
            let content = state.src[result.label_start..result.label_end].to_owned();

            let mut tokens = Vec::new();
            state.md.inline.parse(&content, state.md, state.env, &mut tokens, state.state_level + 1);

            let token = state.push("image", "img", 0);
            token.attrs.push(("src", result.href.unwrap_or_default()));
            token.attrs.push(("alt", String::new()));
            token.children = tokens;
            token.content = content;

            if let Some(x) = result.title {
                token.attrs.push(("title", x));
            }
        }

        state.pos = result.end;
        true
    } else {
        false
    }
}
