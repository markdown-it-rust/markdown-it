// Process [link](<to> "stuff")
//
use crate::inline::State;
use crate::helpers;

pub fn rule(state: &mut State, silent: bool) -> bool {
    if state.src[state.pos..state.pos_max].chars().next().unwrap() != '[' { return false; }

    if let Some(result) = helpers::parse_link(state, state.pos, false) {
        //
        // We found the end of the link, and know for a fact it's a valid link;
        // so all that's left to do is to call tokenizer.
        //
        if !silent {
            let token = state.push("link_open", "a", 1);
            token.attrs.push(("href", result.href.unwrap_or_default()));
            if let Some(x) = result.title {
                token.attrs.push(("title", x));
            }

            let content = state.src[result.label_start..result.label_end].to_owned();
            let mut tokens = Vec::new();

            state.link_level += 1;
            state.md.inline.parse(&content, state.md, state.env, &mut tokens, state.state_level + 1);
            state.link_level -= 1;

            state.tokens.append(&mut tokens);

            state.push("link_close", "a", -1);
        }

        state.pos = result.end;
        true
    } else {
        false
    }
}
