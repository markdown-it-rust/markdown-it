// Parse backticks
//
use crate::Formatter;
use crate::MarkdownIt;
use crate::env;
use crate::inline;
use crate::token::{Token, TokenData};

#[derive(Debug)]
pub struct CodeInline {
    pub marker: char,
    pub marker_len: usize,
    pub content: String,
}

impl TokenData for CodeInline {
    fn render(&self, _: &Token, f: &mut dyn Formatter) {
        f.open("code", &[]);
        f.text(&self.content);
        f.close("code");
    }
}

#[derive(Debug, Default)]
struct BacktickCache {
    scanned: bool,
    max: Vec<usize>,
}

impl env::EnvMember for BacktickCache {
    type Scope = env::scope::Inline;
}

pub fn add(md: &mut MarkdownIt) {
    md.inline.ruler.add("backticks", rule);
}

fn rule(state: &mut inline::State, silent: bool) -> bool {
    let mut chars = state.src[state.pos..state.pos_max].chars();
    if chars.next().unwrap() != '`' { return false; }

    let mut pos = state.pos + 1;

    // scan marker length
    while let Some('`') = chars.next() {
        pos += 1;
    }

    // backtick length => last seen position
    let backticks = state.env.get_or_insert::<BacktickCache>();

    let marker = &state.src[state.pos..pos];
    let opener_len = pos - state.pos;

    if backticks.scanned && backticks.max[opener_len] <= state.pos {
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

        while let Some('`') = chars.next() {
            match_end += 1;
        }

        let closer_len = match_end - match_start;

        if closer_len == opener_len {
            // Found matching closer length.
            if !silent {
                let mut content = state.src[pos..match_start].to_owned().replace('\n', " ");
                if content.starts_with(' ') && content.ends_with(' ') && content.len() > 2 {
                    content = content[1..content.len() - 1].to_owned();
                }

                state.push(CodeInline {
                    marker: '`',
                    marker_len: opener_len,
                    content,
                });
            }
            state.pos = match_end;
            return true;
        }

        // Some different length found, put it in cache as upper limit of where closer can be found
        while backticks.max.len() <= closer_len { backticks.max.push(0); }
        backticks.max[closer_len] = match_start;
    }

    // Scanned through the end, didn't find anything
    backticks.scanned = true;

    if !silent { state.pending += marker; }
    state.pos += opener_len;
    true
}
