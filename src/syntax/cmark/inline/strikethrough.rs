// ~~strike through~~
//
use crate::MarkdownIt;
use crate::inline::State;
use crate::inline::state::Delimiter;
use std::mem;

pub fn add(md: &mut MarkdownIt) {
    md.inline.ruler.push("strikethrough", rule);
    md.inline.ruler2.push("strikethrough", postprocess);
}

// Insert each marker as a separate text token, and add it to delimiter list
//
fn rule(state: &mut State, silent: bool) -> bool {
    if silent { return false; }

    if state.src[state.pos..state.pos_max].chars().next().unwrap() != '~' { return false; }

    let scanned = state.scan_delims(state.pos, true);
    let mut len = scanned.length;

    if len < 2 { return false; }

    if len % 2 != 0 {
        let mut token = state.push("text", "", 0);
        token.content = "~".to_owned();
        len -= 1;
    }

    for _ in 0..len/2 {
        let token = state.push("text", "", 0);
        token.content = "~~".to_owned();

        state.delimiters.push(Delimiter {
            marker: '~',
            length: 0, // disable "rule of 3" length checks meant for emphasis
            token:  state.tokens.len() - 1,
            end:    None,
            open:   scanned.can_open,
            close:  scanned.can_close
        });
    }

    state.pos += scanned.length;

    true
}

fn process_delimiters(state: &mut State, delimiters: &Vec<Delimiter>) {
    let mut lone_markers = Vec::new();

    for i in 0..delimiters.len() {
        let start_delim = &delimiters[i];

        if start_delim.marker != '~' { continue; }

        // Process only opening markers
        if start_delim.end.is_none() { continue; }

        let start_delim_end = start_delim.end.unwrap();
        let end_delim = &delimiters[start_delim_end];

        let mut token;

        token = &mut state.tokens[start_delim.token];
        token.name    = "s_open";
        token.tag     = "s";
        token.nesting = 1;
        token.content = String::new();
        token.markup  = "~~".to_owned();

        token = &mut state.tokens[end_delim.token];
        token.name    = "s_close";
        token.tag     = "s";
        token.nesting = -1;
        token.content = String::new();
        token.markup  = "~~".to_owned();

        let idx = end_delim.token - 1;
        if state.tokens[idx].name == "text" && state.tokens[idx].content == "~" {
            lone_markers.push(idx);
        }
    }

    // If a marker sequence has an odd number of characters, it's split
    // like this: `~~~~~` -> `~` + `~~` + `~~`, leaving one marker at the
    // start of the sequence.
    //
    // So, we have to move all those markers after subsequent s_close tags.
    //
    let state_tokens_len = state.tokens.len();
    for i in lone_markers.iter().rev() {
        let mut j = i + 1;

        while j < state_tokens_len && state.tokens[j].name == "s_close" {
            j += 1;
        }

        j -= 1;

        if *i != j {
            state.tokens.swap(*i, j);
        }
    }
}

// Walk through delimiter list and replace text tokens with tags
//
fn postprocess(state: &mut State) {
    let delimiters = mem::replace(&mut state.delimiters, Vec::new());
    process_delimiters(state, &delimiters);
    state.delimiters = delimiters;
}
