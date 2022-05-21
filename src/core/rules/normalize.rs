use crate::core::State;

// Normalize input string (newlines and NULL character)
// https://spec.commonmark.org/0.29/#line-ending

pub fn rule(state: &mut State) {
    state.src = state.src.replace("\r\n", "\n")
                         .replace("\r", "\n")
                         .replace("\0", "\u{FFFD}");
}
