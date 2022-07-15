// Process [link](<to> "stuff")
//
use crate::Node;
use crate::parser::MarkdownIt;
use crate::parser::internals::common::normalize_reference;
use crate::parser::internals::common::unescape_all;
use crate::parser::internals::inline;
use crate::syntax::cmark::block::reference::ReferenceEnv;

#[derive(Debug)]
struct LinkCfg<const PREFIX: char>(fn (Option<String>, Option<String>) -> Node);

pub fn add<const ENABLE_NESTED: bool>(
    md: &mut MarkdownIt,
    f: fn (Option<String>, Option<String>) -> Node
) {
    md.env.insert(LinkCfg::<'\0'>(f));

    md.inline.ruler.add("generic::full_link", |state: &mut inline::State, silent: bool| -> bool {
        let mut chars = state.src[state.pos..state.pos_max].chars();
        if chars.next().unwrap() != '[' { return false; }
        let f = state.md.env.get::<LinkCfg<'\0'>>().unwrap().0;
        rule(state, silent, ENABLE_NESTED, 0, f)
    });
}

pub fn add_prefix<const PREFIX: char, const ENABLE_NESTED: bool>(
    md: &mut MarkdownIt,
    f: fn (Option<String>, Option<String>
) -> Node) {
    md.env.insert(LinkCfg::<PREFIX>(f));

    md.inline.ruler.add("generic::full_link", |state: &mut inline::State, silent: bool| -> bool {
        let mut chars = state.src[state.pos..state.pos_max].chars();
        if chars.next() != Some(PREFIX) { return false; }
        if chars.next() != Some('[') { return false; }
        let f = state.md.env.get::<LinkCfg<PREFIX>>().unwrap().0;
        rule(state, silent, ENABLE_NESTED, 1, f)
    });
}

fn rule(
    state: &mut inline::State,
    silent: bool,
    enable_nested: bool,
    offset: usize,
    f: fn (Option<String>, Option<String>) -> Node
) -> bool {
    let start = state.pos;

    if let Some(result) = parse_link(state, state.pos + offset, enable_nested) {
        //
        // We found the end of the link, and know for a fact it's a valid link;
        // so all that's left to do is to call tokenizer.
        //
        if !silent {
            let old_node = std::mem::replace(&mut state.node, f(result.href, result.title));
            let max = state.pos_max;

            state.link_level += 1;
            state.pos = result.label_start;
            state.pos_max = result.label_end;
            state.md.inline.tokenize(state);
            state.pos_max = max;

            let mut node = std::mem::replace(&mut state.node, old_node);
            node.srcmap = state.get_map(start, result.end);
            state.push(node);
            state.link_level -= 1;
        }

        state.pos = result.end;
        true
    } else {
        false
    }
}

// Parse link label
//
// this function assumes that first character ("[") already matches;
// returns the end of the label
//
pub fn parse_link_label(state: &mut inline::State, start: usize, enable_nested: bool) -> Option<usize> {
    let old_pos = state.pos;
    let mut found = false;
    let mut label_end = None;
    let mut level = 1;

    state.pos = start + 1;

    while let Some(ch) = state.src[state.pos..state.pos_max].chars().next() {
        if ch == ']' {
            level -= 1;
            if level == 0 {
                found = true;
                break;
            }
        }

        let prev_pos = state.pos;
        state.md.inline.skip_token(state);
        if ch == '[' {
            if prev_pos == state.pos - 1 {
                // increase level if we find text `[`, which is not a part of any token
                level += 1;
            } else if !enable_nested {
                state.pos = old_pos;
                return None;
            }
        }
    }

    if found {
        label_end = Some(state.pos);
    }

    // restore old state
    state.pos = old_pos;

    label_end
}


pub struct ParseLinkFragmentResult {
    pub pos:   usize,
    pub lines: usize,
    pub str:   String,
}


// Parse link destination
//
pub fn parse_link_destination(str: &str, start: usize, max: usize) -> Option<ParseLinkFragmentResult> {
    let mut chars = str[start..max].chars().peekable();
    let mut pos = start;

    if let Some('<') = chars.peek() {
        chars.next(); // skip '<'
        pos += 1;
        loop {
            match chars.next() {
                Some('\n' | '<') | None => return None,
                Some('>') => {
                    return Some(ParseLinkFragmentResult {
                        pos: pos + 1,
                        lines: 0,
                        str: unescape_all(&str[start + 1..pos]),
                    });
                }
                Some('\\') => {
                    match chars.next() {
                        None => return None,
                        Some(x) => pos += 1 + x.len_utf8(),
                    }
                }
                Some(x) => {
                    pos += x.len_utf8();
                }
            }
        }
    } else {
        let mut level : u32 = 0;
        loop {
            match chars.next() {
                // space + ascii control characters
                Some('\0'..=' ' | '\x7f') | None => break,
                Some('\\') => {
                    match chars.next() {
                        Some(' ') | None => break,
                        Some(x) => pos += 1 + x.len_utf8(),
                    }
                }
                Some('(') => {
                    level += 1;
                    if level > 32 { return None; }
                    pos += 1;
                }
                Some(')') => {
                    if level == 0 { break; }
                    level -= 1;
                    pos += 1;
                }
                Some(x) => {
                    pos += x.len_utf8();
                }
            }
        }

        if level != 0 { return None; }

        return Some(ParseLinkFragmentResult {
            pos,
            lines: 0,
            str: unescape_all(&str[start..pos]),
        });
    }
}


// Parse link title
//
pub fn parse_link_title(str: &str, start: usize, max: usize) -> Option<ParseLinkFragmentResult> {
    let mut chars = str[start..max].chars();
    let mut pos = start + 1;
    let mut lines = 0;
    let marker;

    match chars.next() {
        Some('"')  => marker = '"',
        Some('\'') => marker = '\'',
        Some('(')  => marker = ')',
        None | Some(_) => return None,
    }

    loop {
        match chars.next() {
            Some(ch) if ch == marker => {
                return Some(ParseLinkFragmentResult {
                    pos: pos + 1,
                    lines,
                    str: unescape_all(&str[start + 1..pos]),
                });
            }
            Some('(') if marker == ')' => {
                return None;
            }
            Some('\n') => {
                pos += 1;
                lines += 1;
            }
            Some('\\') => {
                match chars.next() {
                    None => return None,
                    Some(x) => pos += 1 + x.len_utf8(),
                }
            }
            Some(x) => {
                pos += x.len_utf8();
            }
            None => {
                return None;
            }
        }
    }
}

struct ParseLinkResult {
    pub label_start: usize,
    pub label_end: usize,
    pub href: Option<String>,
    pub title: Option<String>,
    pub end: usize,
}

// Parses [link](<to> "stuff")
//
// this function assumes that first character ("[") already matches
//
fn parse_link(state: &mut inline::State, pos: usize, enable_nested: bool) -> Option<ParseLinkResult> {
    let label_end;

    if let Some(x) = parse_link_label(state, pos, enable_nested) {
        label_end = x;
    } else {
        // parser failed to find ']', so it's not a valid link
        return None;
    }

    let label_start = pos + 1;
    let mut pos = label_end + 1;
    let mut chars = state.src[pos..state.pos_max].chars();
    let mut href = None;
    let mut title = None;

    if let Some('(') = chars.next() {
        //
        // Inline link
        //

        // [link](  <href>  "title"  )
        //        ^^ skipping these spaces
        pos += 1;
        while let Some(' ' | '\t' | '\n') = chars.next() {
            pos += 1;
        }

        // [link](  <href>  "title"  )
        //          ^^^^^^ parsing link destination
        if let Some(res) = parse_link_destination(&state.src, pos, state.pos_max) {
            let href_candidate = (state.md.normalize_link)(&res.str);
            if (state.md.validate_link)(&href_candidate) {
                pos = res.pos;
                href = Some(href_candidate);
            }

            // [link](  <href>  "title"  )
            //                ^^ skipping these spaces
            let mut chars = state.src[pos..state.pos_max].chars();
            while let Some(' ' | '\t' | '\n') = chars.next() {
                pos += 1;
            }

            if let Some(res) = parse_link_title(&state.src, pos, state.pos_max) {
                title = Some(res.str);
                pos = res.pos;

                // [link](  <href>  "title"  )
                //                         ^^ skipping these spaces
                let mut chars = state.src[pos..state.pos_max].chars();
                while let Some(' ' | '\t' | '\n') = chars.next() {
                    pos += 1;
                }
            }
        }

        if let Some(')') = state.src[pos..state.pos_max].chars().next() {
            return Some(ParseLinkResult {
                label_start,
                label_end,
                href,
                title,
                end: pos + 1,
            })
        }
    }

    //
    // Link reference
    //
    // TODO: check if I have any references?
    pos = label_end + 1;
    let mut maybe_label = None;

    match state.src[pos..state.pos_max].chars().next() {
        Some('[') => {
            if let Some(x) = parse_link_label(state, pos, false) {
                maybe_label = Some(&state.src[pos + 1..x]);
                pos = x + 1;
            } else {
                pos = label_end + 1;
            }
        }
        _ => pos = label_end + 1,
    }

    if let Some(references) = state.root_env.get::<ReferenceEnv>() {
        // covers label === '' and label === undefined
        // (collapsed reference link and shortcut reference link respectively)
        let label = if maybe_label.is_none() || maybe_label == Some("") {
            &state.src[label_start..label_end]
        } else {
            maybe_label.unwrap()
        };

        let lref = references.map.get(&normalize_reference(label));

        if let Some(r) = lref {
            Some(ParseLinkResult {
                label_start,
                label_end,
                href: Some(r.0.clone()),
                title: r.1.clone(),
                end: pos,
            })
        } else {
            None
        }
    } else {
        None
    }
}
