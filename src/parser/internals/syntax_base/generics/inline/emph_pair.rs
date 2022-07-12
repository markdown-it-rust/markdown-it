use std::cmp::min;
use crate::{Node, NodeValue};
use crate::parser::MarkdownIt;
use crate::parser::internals::inline;
use crate::parser::internals::sourcemap::SourcePos;
use crate::parser::internals::syntax_base::builtin::Text;

#[derive(Debug, Default)]
struct PairConfig<const MARKER: char> {
    inserted: bool,
    fns: [Option<fn () -> Node>; 3],
}

#[derive(Debug, Default)]
struct OpenersBottom<const MARKER: char>([ usize; 6 ]);

#[derive(Debug, Clone)]
pub struct EmphMarker {
    // Starting marker
    pub marker:    char,

    // Total length of these series of delimiters.
    pub length:    usize,

    // Remaining length that's not already matched to other delimiters.
    pub remaining: usize,

    // Boolean flags that determine if this delimiter could open or close
    // an emphasis.
    pub open:      bool,
    pub close:     bool,
}

// this node is supposed to be replaced by actual emph or text node
impl NodeValue for EmphMarker {}

pub fn add_with<const MARKER: char, const LENGTH: u8, const CAN_SPLIT_WORD: bool>(md: &mut MarkdownIt, f: fn () -> Node) {
    let pair_config = md.env.get_or_insert_default::<PairConfig<MARKER>>();
    pair_config.fns[LENGTH as usize - 1] = Some(f);

    if !pair_config.inserted {
        pair_config.inserted = true;
        md.inline.ruler.add("generic::emph_pair_find", |state: &mut inline::State, silent: bool| -> bool {
            if silent { return false; }

            let mut chars = state.src[state.pos..state.pos_max].chars();
            if chars.next().unwrap() != MARKER { return false; }

            let scanned = state.scan_delims(state.pos, CAN_SPLIT_WORD);
            let mut token = Node::new(EmphMarker {
                marker:    MARKER,
                length:    scanned.length,
                remaining: scanned.length,
                open:      scanned.can_open,
                close:     scanned.can_close,
            });
            token.srcmap = state.get_map(state.pos, state.pos + scanned.length);
            state.push(token);
            state.pos += scanned.length;
            if scanned.can_close {
                scan_and_match_delimiters::<MARKER>(state);
            }
            true
        });
    }

    if !md.inline.ruler2.contains("generic::emph_fragments_join") {
        md.inline.ruler2.add("generic::emph_fragments_join", |state| {
            state.node.walk_mut(|node, _| fragments_join(node));
        });
    }
}


// Assuming last token is a closing delimiter we just inserted,
// try to find opener(s). If any are found, move stuff to nested emph node.
fn scan_and_match_delimiters<const MARKER: char>(state: &mut inline::State) {
    if state.node.children.len() == 1 { return; } // must have at least opener and closer

    let mut closer_token = state.node.children.pop().unwrap();
    let mut closer = closer_token.cast_mut::<EmphMarker>().unwrap().clone();
    debug_assert!(closer.close);

    // Previously calculated lower bounds (previous fails)
    // for each marker, each delimiter length modulo 3,
    // and for whether this closer can be an opener;
    // https://github.com/commonmark/cmark/commit/34250e12ccebdc6372b8b49c44fab57c72443460
    let openers_for_marker = state.node.env.get_or_insert_default::<OpenersBottom<MARKER>>();
    let openers_parameter = (closer.open as usize) * 3 + closer.length % 3;

    let min_opener_idx = openers_for_marker.0[openers_parameter];

    let mut idx = state.node.children.len() - 1;
    let mut new_min_opener_idx = idx;
    'outer: while idx > min_opener_idx {
        idx -= 1;

        if let Some(opener) = state.node.children[idx].cast::<EmphMarker>() {
            if opener.open && opener.marker == closer.marker && !is_odd_match(opener, &closer) {
                while closer.remaining > 0 {
                    // opener is retrieved and cast again on each iteration to satisfy desires of The Borrow Checker,
                    // I wish it didn't have to be this way
                    let mut opener = state.node.children[idx].cast_mut::<EmphMarker>().unwrap();
                    let max_marker_len = min(3, min(opener.remaining, closer.remaining));
                    let mut matched_rule = None;
                    let fns = &state.md.env.get::<PairConfig<MARKER>>().unwrap().fns;
                    for marker_len in (1..=max_marker_len).rev() {
                        if let Some(f) = fns[marker_len-1] {
                            matched_rule = Some((marker_len, f));
                            break;
                        }
                    }

                    // If matched_fn isn't found, it can only mean that function is defined for larger marker
                    // than we have (e.g. function defined for **, we have *).
                    // Treat this as "marker not found".
                    if matched_rule.is_none() { continue 'outer; }

                    let (marker_len, marker_fn) = matched_rule.unwrap();

                    closer.remaining -= marker_len;
                    opener.remaining -= marker_len;
                    let opener_is_empty = opener.remaining == 0;

                    let mut new_token = marker_fn();
                    new_token.children = state.node.children.split_off(idx + 1);

                    // cut marker_len chars from start, i.e. "12345" -> "345"
                    let mut end_map_pos = 0;
                    if let Some(map) = closer_token.srcmap {
                        let (start, end) = map.get_byte_offsets();
                        closer_token.srcmap = Some(SourcePos::new(start + marker_len, end));
                        end_map_pos = start + marker_len;
                    }

                    // cut marker_len chars from end, i.e. "12345" -> "123"
                    let mut start_map_pos = 0;
                    let mut opener_token = state.node.children.last_mut().unwrap();
                    if let Some(map) = opener_token.srcmap {
                        let (start, end) = map.get_byte_offsets();
                        opener_token.srcmap = Some(SourcePos::new(start, end - marker_len));
                        start_map_pos = end - marker_len;
                    }

                    new_token.srcmap = state.get_map(start_map_pos, end_map_pos);

                    // remove empty node as a small optimization so we can do less work later
                    if opener_is_empty { state.node.children.pop(); }

                    new_min_opener_idx = 0;
                    state.node.children.push(new_token);

                    // node is removed, no reason to continue working on it
                    if opener_is_empty { break; }
                }
            }
        }
    }

    if new_min_opener_idx != 0 {
        // If match for this delimiter run failed, we want to set lower bound for
        // future lookups. This is required to make sure algorithm has linear
        // complexity.
        //
        // See details here:
        // https://github.com/commonmark/cmark/issues/178#issuecomment-270417442
        //
        let openers_for_marker = state.node.env.get_or_insert_default::<OpenersBottom<MARKER>>();
        openers_for_marker.0[openers_parameter] = new_min_opener_idx;
    }

    // remove empty node as a small optimization so we can do less work later
    if closer.remaining > 0 {
        closer_token.replace(closer);
        state.node.children.push(closer_token);
    }
}


fn is_odd_match(opener: &EmphMarker, closer: &EmphMarker) -> bool {
    // from spec:
    //
    // If one of the delimiters can both open and close emphasis, then the
    // sum of the lengths of the delimiter runs containing the opening and
    // closing delimiters must not be a multiple of 3 unless both lengths
    // are multiples of 3.
    //
    if opener.close || closer.open {
        if (opener.length + closer.length) % 3 == 0 {
            if opener.length % 3 != 0 || closer.length % 3 != 0 {
                return true;
            }
        }
    }

    false
}


// Clean up tokens after emphasis and strikethrough postprocessing:
// merge adjacent text nodes into one and re-calculate all token levels
//
// This is necessary because initially emphasis delimiter markers (*, _, ~)
// are treated as their own separate text tokens. Then emphasis rule either
// leaves them as text (needed to merge with adjacent text) or turns them
// into opening/closing tags (which messes up levels inside).
//
fn fragments_join(node: &mut Node) {
    // replace all emph markers with text tokens
    for token in node.children.iter_mut() {
        if let Some(data) = token.cast::<EmphMarker>() {
            let content = data.marker.to_string().repeat(data.remaining);
            token.replace(Text { content });
        }
    }

    // collapse adjacent text tokens
    for idx in 1..node.children.len() {
        let ( tokens1, tokens2 ) = node.children.split_at_mut(idx);

        let token1 = tokens1.last_mut().unwrap();
        if let Some(t1_data) = token1.cast_mut::<Text>() {

            let token2 = tokens2.first_mut().unwrap();
            if let Some(t2_data) = token2.cast_mut::<Text>() {
                // concat contents
                let t2_content = std::mem::take(&mut t2_data.content);
                t1_data.content += &t2_content;

                // adjust source maps
                if let Some(map1) = token1.srcmap {
                    if let Some(map2) = token2.srcmap {
                        token1.srcmap = Some(SourcePos::new(
                            map1.get_byte_offsets().0,
                            map2.get_byte_offsets().1
                        ));
                    }
                }

                node.children.swap(idx - 1, idx);
            }
        }
    }

    // remove all empty tokens
    node.children.retain(|token| {
        if let Some(data) = token.cast::<Text>() {
            !data.content.is_empty()
        } else {
            true
        }
    });
}
