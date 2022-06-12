// Parse backticks
//
use crate::MarkdownIt;
use crate::inline::State;

pub fn add(md: &mut MarkdownIt) {
    md.inline.ruler.push("backticks", rule);
}

fn rule(state: &mut State, silent: bool) -> bool {
    let mut chars = state.src[state.pos..state.pos_max].chars();
    if chars.next().unwrap() != '`' { return false; }

    let mut pos = state.pos + 1;

    // scan marker length
    loop {
        match chars.next() {
            Some('`')      => { pos += 1; }
            Some(_) | None => { break; }
        }
    }

    let marker = &state.src[state.pos..pos];
    let opener_len = pos - state.pos;

    if state.backticks_scanned && state.backticks[opener_len] <= state.pos {
        if !silent { state.pending += marker; }
        state.pos += opener_len;
        return true;
    }

    let mut match_start;
    let mut match_end = pos;

    // Nothing found in the cache, scan until the end of the line (or until marker is found)
    loop {
        match state.src[match_end..state.pos_max].find('`') {
            Some(x) => { match_start = match_end + x; }
            None =>    { break; }
        }

        // scan marker length
        match_end = match_start + 1;
        chars = state.src[match_end..state.pos_max].chars();

        loop {
            match chars.next() {
                Some('`')      => { match_end += 1; }
                Some(_) | None => { break; }
            }
        }

        let closer_len = match_end - match_start;

        if closer_len == opener_len {
            // Found matching closer length.
            if !silent {
                let mut content = state.src[pos..match_start].to_owned().replace("\n", " ");
                if content.starts_with(' ') && content.ends_with(' ') && content.len() > 2 {
                    content = content[1..content.len() - 1].to_owned();
                }
                let markup = marker.to_owned();

                let mut token = state.push("code_inline", "code", 0);
                token.markup = markup;
                token.content = content;
            }
            state.pos = match_end;
            return true;
        }

        // Some different length found, put it in cache as upper limit of where closer can be found
        while state.backticks.len() <= closer_len { state.backticks.push(0); }
        state.backticks[closer_len] = match_start;
    }

    // Scanned through the end, didn't find anything
    state.backticks_scanned = true;

    if !silent { state.pending += marker; }
    state.pos += opener_len;
    true
}
