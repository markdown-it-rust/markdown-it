use crate::MarkdownIt;
use crate::core;

pub fn add(md: &mut MarkdownIt) {
    md.core.ruler.add("normalize", rule).before_all();
}

// Normalize input string (newlines and NULL character)
// https://spec.commonmark.org/0.29/#line-ending

fn rule(state: &mut core::State) {
    state.src = state.src.replace("\r\n", "\n")
                         .replace('\r', "\n")
                         .replace('\0', "\u{FFFD}");
}
