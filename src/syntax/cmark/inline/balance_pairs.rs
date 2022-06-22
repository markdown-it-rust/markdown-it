// For each opening emphasis-like marker find a matching closing one
//
use crate::MarkdownIt;
use crate::inline::State;
use std::collections::HashMap;

pub fn add(md: &mut MarkdownIt) {
    md.inline.ruler2.add("balance_pairs", postprocess);
}

fn postprocess(state: &mut State) {
    let max = state.delimiters.len();
    if max == 0 { return; }

    // header_idx is the first delimiter of the current (where closer is) delimiter ru
    let mut header_idx = 0;
    let mut last_token_idx: i32 = -2; // needs any value lower than -1
    let mut jumps = Vec::new();
    let mut openers_bottom = HashMap::new();

    for closer_idx in 0..max {
        let closer = &state.delimiters[closer_idx];

        jumps.push(0);

        // markers belong to same delimiter run if:
        //  - they have adjacent tokens
        //  - AND markers are the same
        //
        if state.delimiters[header_idx].marker != closer.marker || last_token_idx != closer.token as i32 - 1 {
            header_idx = closer_idx;
        }

        last_token_idx = closer.token as i32;

        if !closer.close { continue; }

        // Previously calculated lower bounds (previous fails)
        // for each marker, each delimiter length modulo 3,
        // and for whether this closer can be an opener;
        // https://github.com/commonmark/cmark/commit/34250e12ccebdc6372b8b49c44fab57c72443460
        let openers_for_marker = openers_bottom.entry(closer.marker).or_insert_with(|| [ -1i32; 6 ]);
        let openers_parameter = if closer.open { 3 } else { 0 } + closer.length % 3;

        let min_opener_idx = openers_for_marker[openers_parameter];
        let mut opener_idx : i32 = header_idx as i32 - jumps[header_idx] as i32 - 1;
        let mut new_min_opener_idx = opener_idx;

        while opener_idx > min_opener_idx {
            let opener = &state.delimiters[opener_idx as usize];

            if opener.marker != closer.marker {
                opener_idx -= jumps[opener_idx as usize] as i32 + 1;
                continue;
            }

            if opener.open && opener.end.is_none() {
                let mut is_odd_match = false;

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
                            is_odd_match = true;
                        }
                    }
                }

                if !is_odd_match {
                    // If previous delimiter cannot be an opener, we can safely skip
                    // the entire sequence in future checks. This is required to make
                    // sure algorithm has linear complexity (see *_*_*_*_*_... case).
                    //
                    let last_jump : usize = if opener_idx > 0 && !state.delimiters[opener_idx as usize - 1].open {
                        jumps[opener_idx as usize - 1] + 1
                    } else { 0 };

                    jumps[closer_idx] = closer_idx as usize - opener_idx as usize + last_jump;
                    jumps[opener_idx as usize] = last_jump;

                    state.delimiters[closer_idx].open  = false;
                    state.delimiters[opener_idx as usize].end   = Some(closer_idx);
                    state.delimiters[opener_idx as usize].close = false;
                    new_min_opener_idx = -1;
                    // treat next token as start of run,
                    // it optimizes skips in **<...>**a**<...>** pathological case
                    last_token_idx = -2;
                    break;
                }
            }

            opener_idx -= jumps[opener_idx as usize] as i32 + 1;
        }

        if new_min_opener_idx != -1 {
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
}
