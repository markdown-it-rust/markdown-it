use derivative::Derivative;
use std::cmp::min;
use std::collections::HashMap;
use crate::Formatter;
use crate::MarkdownIt;
use crate::env;
use crate::inline;
use crate::sourcemap::SourcePos;
use crate::syntax_base::builtin::Text;
use crate::token::{Token, TokenData};

#[derive(Debug, Default)]
struct Pairs {
    rule_inserted: bool,
    map: HashMap<char, [Option<fn () -> Token>; 3]>
}

impl Pairs {
    pub fn set(&mut self, ch: char, len: u8, f: fn() -> Token) {
        assert!((1..=3).contains(&len), "only pairs with len=1..3 are supported");
        self.map.entry(ch).or_default()[len as usize - 1] = Some(f);
    }

    /*pub fn get(&self, ch: char, len: u8) -> Option<fn() -> Token> {
        assert!((1..=3).contains(&len), "only pairs with len=1..3 are supported");
        self.map.get(&ch)?[len as usize - 1]
    }*/
}

#[derive(Debug)]
pub struct EmphMarker {
    pub marker: char,
    pub length: usize,
}

// this token is supposed to be replaced by actual emph or text node
impl TokenData for EmphMarker {
    fn render(&self, _: &Token, _: &mut dyn Formatter) {
        unimplemented!()
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
struct Delimiter {
    // Starting marker
    marker: char,

    // Total length of these series of delimiters.
    length: usize,

    // A position of the token this delimiter corresponds to.
    token:  usize,

    // Boolean flags that determine if this delimiter could open or close
    // an emphasis.
    open:   bool,
    close:  bool,
}

#[derive(Default, Debug)]
struct Delimiters(Vec<Delimiter>);

// List of emphasis-like delimiters for current tag
impl Delimiters {
    pub fn push(&mut self, run: inline::state::DelimRun, token: usize) {
        self.0.push(Delimiter {
            marker: run.marker,
            length: run.length,
            token,
            open: run.can_open,
            close: run.can_close,
        })
    }
}

impl env::EnvMember for Delimiters {
    type Scope = env::scope::InlineLvl;
}

pub fn add_with<const MARKER: char, const LENGTH: u8, const CAN_SPLIT_WORD: bool>(md: &mut MarkdownIt, f: fn () -> Token) {
    let pairs = md.env.get_or_insert_default::<Pairs>();

    if !pairs.map.contains_key(&MARKER) {
        md.inline.ruler.add("generic::emph_pair_find", |state: &mut inline::State, silent: bool| -> bool {
            if silent { return false; }

            let mut chars = state.src[state.pos..state.pos_max].chars();
            if chars.next().unwrap() != MARKER { return false; }

            let scanned = state.scan_delims(state.pos, CAN_SPLIT_WORD);
            let mut token = Token::new(EmphMarker { marker: MARKER, length: scanned.length });
            token.map = state.get_map(state.pos, state.pos + scanned.length);
            state.push(token);
            state.pos += scanned.length;

            state.env.get_or_insert::<Delimiters>().push(scanned, state.tokens.len() - 1);
            true
        });
    }

    if !pairs.rule_inserted {
        md.inline.ruler2.add("generic::emph_pair_balance", rule);
        pairs.rule_inserted = true;
    }

    pairs.set(MARKER, LENGTH, f);
}

// For each opening emphasis-like marker find a matching closing one
//
fn rule(state: &mut inline::State) {
    let delimiters = state.env.get::<Delimiters>();
    if delimiters.is_none() { return; }
    let delimiters = delimiters.unwrap();

    let all_pairs = state.md.env.get::<Pairs>();
    if all_pairs.is_none() { return; }
    let all_pairs = all_pairs.unwrap();

    let mut out_tokens = Vec::new();
    let mut delim_idx = 0;
    let mut openers_bottom = HashMap::new();

    struct AuxInfo { remaining: usize, out_idx: usize, jumps: usize }
    let mut auxinfo = Vec::with_capacity(delimiters.0.len());

    for (idx, mut token) in std::mem::take(state.tokens).into_iter().enumerate() {
        let mut delim = None;

        // find a delimiter corresponding to this token,
        // assuming that delimiters are sorted by .token property;
        // so we increment idx++ and delim_idx++ trying to ensure that
        // delimiters[delim_idx].token == idx
        while delim_idx < delimiters.0.len() {
            let d = &delimiters.0[delim_idx];

            if d.token < idx {
                delim_idx += 1;
                continue;
            } else if d.token == idx {
                delim = Some(d);
                break;
            } else {
                break;
            }
        }

        if let Some(closer) = delim {
            let closer_idx = delim_idx;
            // with delimiter indexes being consecutive, assert that we didn't miss any
            debug_assert_eq!(auxinfo.len(), closer_idx);

            auxinfo.push(AuxInfo {
                remaining: closer.length,
                out_idx: usize::MAX, // will be set later
                jumps: 1,
            });

            if closer.close && closer_idx != 0 {
                // Previously calculated lower bounds (previous fails)
                // for each marker, each delimiter length modulo 3,
                // and for whether this closer can be an opener;
                // https://github.com/commonmark/cmark/commit/34250e12ccebdc6372b8b49c44fab57c72443460
                let openers_for_marker = openers_bottom.entry(closer.marker).or_insert([ 0usize; 6 ]);
                let openers_parameter = (closer.open as usize) * 3 + closer.length % 3;

                let min_opener_idx = openers_for_marker[openers_parameter];
                let mut opener_idx = closer_idx - auxinfo[closer_idx].jumps;
                let mut new_min_opener_idx = opener_idx;

                'outer: while auxinfo[closer_idx].remaining > 0 {
                    let opener = &delimiters.0[opener_idx];

                    if opener.open
                       && opener.marker == closer.marker
                       && !is_odd_match(opener, closer)
                       && auxinfo[opener_idx].remaining > 0 {

                        let max_marker_len = min(3, min(auxinfo[opener_idx].remaining, auxinfo[closer_idx].remaining));
                        let fns = all_pairs.map.get(&opener.marker).map(|x| *x).unwrap_or_default();

                        for marker_len in (1..=max_marker_len).rev() {
                            if fns[marker_len-1].is_none() { continue; }

                            let mut new_token = fns[marker_len-1].unwrap()();
                            new_token.children = fragments_join(out_tokens.split_off(auxinfo[opener_idx].out_idx + 1));

                            // cut marker_len chars from start, i.e. "12345" -> "345" (but they should be all the same)
                            auxinfo[closer_idx].remaining -= marker_len;
                            auxinfo[closer_idx].jumps = closer_idx - opener_idx;
                            let data = token.cast_mut::<EmphMarker>().expect("delimiter points at non-emph node");
                            data.length -= marker_len;
                            let mut end_map_pos = 0;
                            #[cfg(feature="sourcemap")]
                            if let Some(map) = token.map {
                                let (start, end) = map.get_byte_offsets();
                                token.map = Some(SourcePos::new(start + marker_len, end));
                                end_map_pos = start + marker_len;
                            }

                            // cut marker_len chars from end, i.e. "12345" -> "123" (but they should be all the same)
                            auxinfo[opener_idx].remaining -= marker_len;
                            let starttoken = out_tokens.last_mut().unwrap();
                            let data = starttoken.cast_mut::<EmphMarker>().expect("delimiter points at non-emph node");
                            data.length -= marker_len;
                            let mut start_map_pos = 0;
                            #[cfg(feature="sourcemap")]
                            if let Some(map) = starttoken.map {
                                let (start, end) = map.get_byte_offsets();
                                starttoken.map = Some(SourcePos::new(start, end - marker_len));
                                start_map_pos = end - marker_len;
                            }

                            new_token.map = state.get_map(start_map_pos, end_map_pos);
                            new_min_opener_idx = 0;
                            out_tokens.push(new_token);
                            continue 'outer;
                        }
                    }

                    if opener_idx <= min_opener_idx { break; }
                    opener_idx -= auxinfo[opener_idx].jumps;
                }

                if new_min_opener_idx != 0 {
                    // If match for this delimiter run failed, we want to set lower bound for
                    // future lookups. This is required to make sure algorithm has linear
                    // complexity.
                    //
                    // See details here:
                    // https://github.com/commonmark/cmark/issues/178#issuecomment-270417442
                    //
                    openers_for_marker[openers_parameter] = new_min_opener_idx;
                }
            }

            auxinfo.last_mut().unwrap().out_idx = out_tokens.len();
        }

        out_tokens.push(token);
    }

    *state.tokens = fragments_join(out_tokens);
}

fn is_odd_match(opener: &Delimiter, closer: &Delimiter) -> bool {
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
fn fragments_join(mut tokens: Vec<Token>) -> Vec<Token> {
    // replace all emph markers with text tokens
    for token in tokens.iter_mut() {
        if let Some(data) = token.cast::<EmphMarker>() {
            let content = data.marker.to_string().repeat(data.length);
            token.replace(Text { content });
        }
    }

    // collapse adjacent text tokens
    for idx in 1..tokens.len() {
        let ( tokens1, tokens2 ) = tokens.split_at_mut(idx);

        let token1 = tokens1.last_mut().unwrap();
        if let Some(t1_data) = token1.cast_mut::<Text>() {

            let token2 = tokens2.first_mut().unwrap();
            if let Some(t2_data) = token2.cast_mut::<Text>() {
                // concat contents
                let t2_content = std::mem::take(&mut t2_data.content);
                t1_data.content += &t2_content;

                // adjust source maps
                #[cfg(feature="sourcemap")]
                if let Some(map1) = token1.map {
                    if let Some(map2) = token2.map {
                        token1.map = Some(SourcePos::new(
                            map1.get_byte_offsets().0,
                            map2.get_byte_offsets().1
                        ));
                    }
                }

                tokens.swap(idx - 1, idx);
            }
        }
    }

    // remove all empty tokens
    tokens.retain(|token| {
        if let Some(data) = token.cast::<Text>() {
            !data.content.is_empty()
        } else {
            true
        }
    });

    tokens
}
