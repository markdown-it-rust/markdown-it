// Process *this* and _that_
//
use crate::MarkdownIt;
use crate::inline::State;
use crate::inline::state::Delimiter;

pub fn add(md: &mut MarkdownIt) {
    md.inline.ruler.add("emphasis", rule);
    md.inline.ruler2.add("emphasis", postprocess)
        .require("balance_pairs")
        .after("balance_pairs")
        .before("fragments_join");
}

// Insert each marker as a separate text token, and add it to delimiter list
//
fn rule(state: &mut State, silent: bool) -> bool {
    if silent { return false; }

    let mut chars = state.src[state.pos..state.pos_max].chars();
    let marker = chars.next().unwrap();

    if marker != '_' && marker != '*' { return false; }

    let scanned = state.scan_delims(state.pos, marker == '*');

    for _ in 0..scanned.length {
        let token = state.push("text", "", 0);
        token.content = marker.into();

        state.delimiters.push(Delimiter {
            marker,
            length: scanned.length,
            token:  state.tokens.len() - 1,
            end:    None,
            open:   scanned.can_open,
            close:  scanned.can_close
        });
    }

    state.pos += scanned.length;

    true
}

// Walk through delimiter list and replace text tokens with tags
//
fn postprocess(state: &mut State) {
    let mut skip_next = false;

    for i in (0..state.delimiters.len()).rev() {
        if skip_next {
            skip_next = false;
            continue;
        }

        let start_delim = &state.delimiters[i];

        if start_delim.marker != '_' && start_delim.marker != '*' { continue; }

        // Process only opening markers
        if start_delim.end.is_none() { continue; }

        let start_delim_end = start_delim.end.unwrap();
        let end_delim = &state.delimiters[start_delim_end];

        // If the previous delimiter has the same marker and is adjacent to this one,
        // merge those into one strong delimiter.
        //
        // `<em><em>whatever</em></em>` -> `<strong>whatever</strong>`
        //
        let is_strong = i > 0 &&
                        state.delimiters[i - 1].end.unwrap_or_default() == start_delim_end + 1 &&
                        // check that first two markers match and adjacent
                        state.delimiters[i - 1].marker == start_delim.marker &&
                        state.delimiters[i - 1].token == start_delim.token - 1 &&
                        // check that last two markers are adjacent (we can safely assume they match)
                        state.delimiters[start_delim_end + 1].token == end_delim.token + 1;

        let mut token;

        token = &mut state.tokens[start_delim.token];
        token.name    = if is_strong { "strong_open" } else { "em_open" };
        token.tag     = if is_strong { "strong" } else { "em" };
        token.nesting = 1;
        token.content = String::new();
        token.markup  = String::new();
        token.markup.push(start_delim.marker);
        if is_strong { token.markup.push(start_delim.marker); }

        token = &mut state.tokens[end_delim.token];
        token.name    = if is_strong { "strong_close" } else { "em_close" };
        token.tag     = if is_strong { "strong" } else { "em" };
        token.nesting = -1;
        token.content = String::new();
        token.markup  = String::new();
        token.markup.push(start_delim.marker);
        if is_strong { token.markup.push(start_delim.marker); }

        if is_strong {
            state.tokens[state.delimiters[i - 1].token].content = String::new();
            state.tokens[state.delimiters[start_delim_end + 1].token].content = String::new();
            skip_next = true;
        }
    }
}
