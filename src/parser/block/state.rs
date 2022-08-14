// Parser state class
//
use crate::{MarkdownIt, Node};
use crate::common::sourcemap::SourcePos;
use crate::common::utils::calc_right_whitespace_with_tabstops;
use crate::parser::extset::RootExtSet;

#[derive(Debug)]
#[readonly::make]
/// Sandbox object containing data required to parse block structures.
pub struct BlockState<'a, 'b> where 'b: 'a {
    /// Markdown source.
    #[readonly]
    pub src: &'b str,

    /// Link to parser instance.
    #[readonly]
    pub md: &'a MarkdownIt,

    pub root_ext: &'b mut RootExtSet,

    /// Current node, your rule is supposed to add children to it.
    pub node: Node,

    pub line_offsets: Vec<LineOffset>,

    /// Current block content indent (for example, if we are
    /// inside a list, it would be positioned after list marker).
    pub blk_indent: usize,

    /// Current line in src.
    pub line: usize,

    /// Maximum allowed line in src.
    pub line_max: usize,

    /// True if there are no empty lines between paragraphs, used to
    /// toggle loose/tight mode for lists.
    pub tight: bool,

    /// indent of the current list block.
    pub list_indent: Option<u32>,

    pub level: u32,
}

/// Holds start/end/etc. positions for a specific source text line.
#[derive(Debug, Clone)]
pub struct LineOffset {
    /// `line_start` is the actual start of the line.
    ///
    ///     # const IGNORE : &str = stringify! {
    ///     "  >  blockquote\r\n"
    ///      ^-- it will always point here (must not be modified by rules)
    ///     # };
    pub line_start: usize,

    /// `line_end` is first newline character after the line,
    /// or position after string length if there aren't any newlines left.
    ///
    ///     # const IGNORE : &str = stringify! {
    ///     "  >  blockquote\r\n"
    ///                     ^-- it will point here
    ///     # };
    pub line_end: usize,

    /// `first_nonspace` is the byte offset of the first non-space character in
    /// the current line.
    ///
    ///     # const IGNORE : &str = stringify! {
    ///     "   >  blockquote\r\n"
    ///            ^-- it will point here when paragraph is parsed
    ///         ^----- it is initially pointed here
    ///     # };
    ///
    /// It will be modified by rules (list and blockquote), chars before it
    /// must be treated as whitespaces.
    ///
    pub first_nonspace: usize,

    /// `indent_nonspace` is the indent (amount of virtual spaces from start)
    /// of first non-space character in the current line, taking into account
    /// tab expansion.
    ///
    /// For example, in case of ` \t foo`, indent is 5 (tab ends at multiple of 4,
    /// then one space after it). Only tabs and spaces are counted for it,
    /// so no funny unicode business (if cmark supported unicode spaces, they'd
    /// be counted as 1 each regardless of utf8 width).
    ///
    /// You should compare `indent_nonspace` with `state.blkindent` when determining
    /// real indent after taking into account lists.
    ///
    /// Most block rules in commonmark are indented 0..=3, and >=4 is code block.
    /// Special value of ident_nonspace=-1 is used by this library as a sign
    /// that this rule can only be a paragraph continuation (used in blockquotes),
    /// so you must take into account that any math can end up negative.
    ///
    pub indent_nonspace: i32,
}

impl<'a, 'b> BlockState<'a, 'b> {
    pub fn new(src: &'b str, md: &'a MarkdownIt, root_ext: &'b mut RootExtSet, node: Node) -> Self {
        let mut result = Self {
            src,
            md,
            root_ext,
            node,
            line_offsets: Vec::new(),
            blk_indent: 0,
            line: 0,
            line_max: 0,
            tight: false,
            list_indent: None,
            level: 0,
        };

        result.generate_caches();
        result
    }

    fn generate_caches(&mut self) {
        // Create caches
        // Generate markers.
        let mut chars = self.src.chars().peekable();
        let mut indent_found = false;
        let mut indent = 0;
        let mut offset = 0;
        let mut start = 0;
        let mut pos = 0;

        loop {
            match chars.next() {
                Some(ch @ (' ' | '\t')) if !indent_found => {
                    indent += 1;
                    offset += if ch == '\t' { 4 - offset % 4 } else { 1 };
                    pos += 1;
                }
                ch @ (Some('\n' | '\r') | None) => {
                    self.line_offsets.push(LineOffset {
                        line_start: start,
                        line_end: pos,
                        first_nonspace: start + indent,
                        indent_nonspace: offset,
                    });

                    if ch == Some('\r') && chars.peek() == Some(&'\n') {
                        // treat CR+LF as one linebreak
                        chars.next();
                        pos += 1;
                    }

                    indent_found = false;
                    indent = 0;
                    offset = 0;
                    start = pos + 1;
                    pos += 1;

                    if ch.is_none() || chars.peek().is_none() {
                        break;
                    }
                }
                Some(ch) => {
                    indent_found = true;
                    pos += ch.len_utf8();
                }
            }
        }

        self.line_max = self.line_offsets.len();
    }

    #[must_use]
    pub fn test_rules_at_line(&mut self) -> bool {
        for rule in self.md.block.ruler.iter() {
            if rule.0(self).is_some() {
                return true;
            }
        }
        false
    }

    #[must_use]
    #[inline]
    pub fn is_empty(&self, line: usize) -> bool {
        if let Some(offsets) = self.line_offsets.get(line) {
            offsets.first_nonspace >= offsets.line_end
        } else {
            false
        }
    }

    pub fn skip_empty_lines(&self, from: usize) -> usize {
        let mut line = from;
        while line != self.line_max && self.is_empty(line) {
            line += 1;
        }
        line
    }

    // return line indent of specific line, taking into account blockquotes and lists;
    // it may be negative if a text has less indentation than current list item
    #[must_use]
    #[inline]
    pub fn line_indent(&self, line: usize) -> i32 {
        if line < self.line_max {
            self.line_offsets[line].indent_nonspace - self.blk_indent as i32
        } else {
            0
        }
    }

    // return a single line, trimming initial spaces
    #[must_use]
    #[inline]
    pub fn get_line(&self, line: usize) -> &str {
        if line < self.line_max {
            let pos = self.line_offsets[line].first_nonspace;
            let max = self.line_offsets[line].line_end;
            &self.src[pos..max]
        } else {
            ""
        }
    }

    // Cut a range of lines begin..end (not including end) from the source without preceding indent.
    // Returns a string (lines) plus a mapping (start of each line in result -> start of each line in source).
    pub fn get_lines(&self, begin: usize, end: usize, indent: usize, keep_last_lf: bool) -> (String, Vec<(usize, usize)>) {
        debug_assert!(begin <= end);

        let mut line = begin;
        let mut result = String::new();
        let mut mapping = Vec::new();

        while line < end {
            let offsets = &self.line_offsets[line];
            let last = offsets.line_end;
            let add_last_lf = line + 1 < end || keep_last_lf;

            let (num_spaces, first) = calc_right_whitespace_with_tabstops(
                &self.src[offsets.line_start..offsets.first_nonspace],
                offsets.indent_nonspace - indent as i32
            );

            mapping.push(( result.len(), offsets.line_start+first ));
            result += &" ".repeat(num_spaces as usize);
            result += &self.src[offsets.line_start+first..last];
            if add_last_lf { result.push('\n'); }
            line += 1;
        }

        ( result, mapping )
    }

    #[must_use]
    #[inline]
    pub fn get_map(&self, start_line: usize, end_line: usize) -> Option<SourcePos> {
        debug_assert!(start_line <= end_line);

        Some(SourcePos::new(
            self.line_offsets[start_line].first_nonspace,
            self.line_offsets[end_line].line_end
        ))
    }

    #[must_use]
    #[inline]
    pub fn get_map_from_offsets(&self, start_pos: usize, end_pos: usize) -> Option<SourcePos> {
        debug_assert!(start_pos <= end_pos);

        Some(SourcePos::new(start_pos, end_pos))
    }
}
