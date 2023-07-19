//! GFM tables
//!
//! <https://github.github.com/gfm/#tables-extension->
use crate::common::sourcemap::SourcePos;
use crate::parser::block::{BlockRule, BlockState};
use crate::parser::extset::RenderExt;
use crate::parser::inline::InlineRoot;
use crate::plugins::cmark::block::heading::HeadingScanner;
use crate::plugins::cmark::block::list::ListScanner;
use crate::{MarkdownIt, Node, NodeValue, Renderer};

#[derive(Debug)]
pub struct Table {
    pub alignments: Vec<ColumnAlignment>,
}

impl NodeValue for Table {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        let old_context = fmt.ext().remove::<TableRenderContext>();
        fmt.ext().insert(TableRenderContext { head: false, alignments: self.alignments.clone(), index: 0 });

        fmt.cr();
        fmt.open("table", &node.attrs);
        fmt.cr();
        fmt.contents(&node.children);
        fmt.cr();
        fmt.close("table");
        fmt.cr();

        old_context.map(|ctx| fmt.ext().insert(ctx));
    }
}

#[derive(Debug, Default)]
pub struct TableRenderContext {
    pub head: bool,
    pub index: usize,
    pub alignments: Vec<ColumnAlignment>,
}

impl RenderExt for TableRenderContext {}

#[derive(Debug)]
pub struct TableHead;

impl NodeValue for TableHead {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        let ctx = fmt.ext().get_or_insert_default::<TableRenderContext>();
        ctx.head = true;

        fmt.cr();
        fmt.open("thead", &node.attrs);
        fmt.cr();
        fmt.contents(&node.children);
        fmt.cr();
        fmt.close("thead");
        fmt.cr();

        let ctx = fmt.ext().get_or_insert_default::<TableRenderContext>();
        ctx.head = false;
    }
}

#[derive(Debug)]
pub struct TableBody;

impl NodeValue for TableBody {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.cr();
        fmt.open("tbody", &node.attrs);
        fmt.cr();
        fmt.contents(&node.children);
        fmt.cr();
        fmt.close("tbody");
        fmt.cr();
    }
}

#[derive(Debug)]
pub struct TableRow;

impl NodeValue for TableRow {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        let ctx = fmt.ext().get_or_insert_default::<TableRenderContext>();
        ctx.index = 0;

        fmt.cr();
        fmt.open("tr", &node.attrs);
        fmt.cr();
        fmt.contents(&node.children);
        fmt.cr();
        fmt.close("tr");
        fmt.cr();
    }
}

#[derive(Debug)]
pub struct TableCell;

impl NodeValue for TableCell {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        let ctx = fmt.ext().get_or_insert_default::<TableRenderContext>();
        let tag = if ctx.head { "th" } else { "td" };

        let mut attrs = node.attrs.clone();

        match ctx.alignments.get(ctx.index).copied().unwrap_or_default() {
            ColumnAlignment::None => (),
            ColumnAlignment::Left => attrs.push(("style", "text-align:left".to_owned())),
            ColumnAlignment::Right => attrs.push(("style", "text-align:right".to_owned())),
            ColumnAlignment::Center => attrs.push(("style", "text-align:center".to_owned())),
        }

        ctx.index += 1;

        fmt.open(tag, &attrs);
        fmt.contents(&node.children);
        fmt.close(tag);
        fmt.cr();
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.block.add_rule::<TableScanner>()
        .before::<ListScanner>()
        .before::<HeadingScanner>();
}

#[doc(hidden)]
pub struct TableScanner;

#[derive(Debug)]
struct RowContent {
    str: String,
    srcmap: Vec<(usize, usize)>,
}

#[derive(Debug, Clone, Copy)]
pub enum ColumnAlignment {
    None,
    Left,
    Right,
    Center,
}

impl Default for ColumnAlignment {
    fn default() -> Self { Self::None }
}

impl TableScanner {
    fn scan_row(line: &str) -> Vec<RowContent> {
        let mut result = Vec::new();
        let mut str = String::new();
        let mut srcmap = vec![(0, 0)];
        let mut is_escaped = false;
        let mut is_leading = true;

        for (pos, ch) in line.char_indices() {
            match ch {
                ' ' | '\t' if is_leading => {
                    srcmap[0].1 += 1;
                }
                '|' => {
                    is_leading = false;
                    if is_escaped {
                        str.push_str(&line[srcmap.last().unwrap().1..pos-1]);
                        srcmap.push((str.len(), pos));
                    } else {
                        str.push_str(&line[srcmap.last().unwrap().1..pos]);
                        result.push(RowContent {
                            str: std::mem::take(&mut str),
                            srcmap: std::mem::take(&mut srcmap),
                        });
                        srcmap = vec![(0, pos + 1)];
                        is_escaped = false;
                        is_leading = true;
                    }
                }
                '\\' => {
                    is_leading = false;
                    is_escaped = true;
                }
                _ => {
                    is_leading = false;
                    is_escaped = false;
                }
            }
        }

        str.push_str(&line[srcmap.last().unwrap().1..]);
        result.push(RowContent {
            str,
            srcmap,
        });

        // trim trailing spaces
        for content in result.iter_mut() {
            while content.str.ends_with([ ' ', '\t' ]) {
                content.str.pop();
            }
        }

        // remove last cell if empty
        if let Some(RowContent { str, srcmap: _ }) = result.last() {
            if str.is_empty() { result.pop(); }
        }

        // remove first cell if empty
        if let Some(RowContent { str, srcmap: _ }) = result.first() {
            if str.is_empty() { result.remove(0); }
        }

        result
    }

    fn scan_alignment_row(line: &str) -> Option<Vec<ColumnAlignment>> {
        // quick check second line, only allow :-| and spaces
        // (this is for performance only)
        let mut has_delimiter = false;
        for ch in line.chars() {
            match ch {
                '|'| ':' => { has_delimiter = true },      
                '-' | ' ' | '\t' => (),
                _ => return None,
            }
        }
        if !has_delimiter { return None; }

        // if first character is '-', then second character must not be a space
        // (due to parsing ambiguity with list)
        if line.starts_with("- ") { return None; }

        let mut result = Vec::new();

        for RowContent { str, srcmap: _ } in Self::scan_row(line) {
            let mut alignment : u8 = 0;
            let mut cell = str.as_str();

            if cell.starts_with(':') {
                alignment |= 1;
                cell = &cell[1..];
            }

            if cell.ends_with(':') {
                alignment |= 2;
                cell = &cell[..cell.len()-1];
            }

            // only allow '-----' in the remainder
            if cell.is_empty() || cell.contains(|c| c != '-') {
                return None;
            }

            result.push(match alignment {
                0 => ColumnAlignment::None,
                1 => ColumnAlignment::Left,
                2 => ColumnAlignment::Right,
                3 => ColumnAlignment::Center,
                _ => unreachable!(),
            });
        }

        Some(result)
    }

    fn scan_header(state: &BlockState) -> Option<(Vec<RowContent>, Vec<ColumnAlignment>)> {
        // should have at least two lines
        if state.line + 2 > state.line_max { return None; }

        if state.line_indent(state.line) >= state.md.max_indent { return None; }

        let next_line = state.line + 1;
        if state.line_indent(next_line) < 0 { return None; }

        if state.line_indent(next_line) >= state.md.max_indent { return None; }

        let alignments = Self::scan_alignment_row(state.get_line(next_line))?;
        let header_row = Self::scan_row(state.get_line(state.line));

        // header row must match the delimiter row in the number of cells
        if header_row.len() != alignments.len() {
            return None;
        }

        // table without any columns is not a table, see markdown-it#724
        if header_row.is_empty() {
            return None;
        }

        Some(( header_row, alignments ))
    }
}

impl BlockRule for TableScanner {
    fn check(state: &mut BlockState) -> Option<()> {
        if state.node.is::<TableBody>() { return None; }

        Self::scan_header(state).map(|_| ())
    }

    fn run(state: &mut BlockState) -> Option<(Node, usize)> {
        let ( header_row, alignments ) = Self::scan_header(state)?;
        let table_cell_count = header_row.len();
        let mut table_node = Node::new(Table { alignments });

        let mut thead_node = Node::new(TableHead);
        thead_node.srcmap = state.get_map(state.line, state.line + 1);

        let mut row_node = Node::new(TableRow);
        row_node.srcmap = state.get_map(state.line, state.line);

        fn add_cell(row_node: &mut Node, cell: String, srcmap: Vec<(usize, usize)>) {
            let mut cell_node = Node::new(TableCell);
            let (start, _) = row_node.srcmap.unwrap().get_byte_offsets();
            cell_node.srcmap = Some(SourcePos::new(
                start + srcmap.first().unwrap().1,
                start + srcmap.last().unwrap().1 + cell.len() - srcmap.last().unwrap().0,
            ));
            if !cell.is_empty() {
                let mapping = srcmap.into_iter().map(|(dstpos, srcpos)| (dstpos, srcpos + start)).collect();
                cell_node.children.push(Node::new(InlineRoot::new(cell, mapping)));
            }
            row_node.children.push(cell_node);
        }

        for RowContent { str: cell, srcmap } in header_row {
            add_cell(&mut row_node, cell, srcmap);
        }

        thead_node.children.push(row_node);
        table_node.children.push(thead_node);

        let tbody_node = Node::new(TableBody);
        let old_node = std::mem::replace(&mut state.node, tbody_node);

        //
        // Iterate table rows
        //

        let start_line = state.line;
        state.line += 2;

        while state.line < state.line_max {
            //
            // Try to check if table is terminated or continued.
            //
            if state.line_indent(state.line) < 0 { break; }

            if state.line_indent(state.line) >= state.md.max_indent { break; }

            // stop if the line is empty
            if state.is_empty(state.line) { break; }

            // fail if terminating block found
            if state.test_rules_at_line() { break; }

            let mut row_node = Node::new(TableRow);
            row_node.srcmap = state.get_map(state.line, state.line);
            let line = state.get_line(state.line);

            let mut body_row = Self::scan_row(line);
            let mut end_of_line = RowContent { str: String::new(), srcmap: vec![(0, line.len())] };

            for index in 0..table_cell_count {
                let RowContent { str: cell, srcmap } = body_row.get_mut(index).unwrap_or(&mut end_of_line);
                add_cell(&mut row_node, cell.clone(), srcmap.clone());
            }

            state.node.children.push(row_node);
            state.line += 1;
        }

        let mut tbody_node = std::mem::replace(&mut state.node, old_node);

        if !tbody_node.children.is_empty() {
            tbody_node.srcmap = state.get_map(start_line + 2, state.line - 1);
            table_node.children.push(tbody_node);
        }

        let line_count = state.line - start_line;
        state.line = start_line;
        Some((table_node, line_count))
    }
}


#[cfg(test)]
mod tests {
    use super::TableScanner;

    #[test]
    fn should_split_cells() {
        assert_eq!(TableScanner::scan_row("").len(), 0);
        assert_eq!(TableScanner::scan_row("a").len(), 1);
        assert_eq!(TableScanner::scan_row("a | b").len(), 2);
        assert_eq!(TableScanner::scan_row("a | b | c").len(), 3);
    }

    #[test]
    fn should_ignore_leading_trailing_empty_cells() {
        assert_eq!(TableScanner::scan_row("foo | bar").len(), 2);
        assert_eq!(TableScanner::scan_row("foo | bar |").len(), 2);
        assert_eq!(TableScanner::scan_row("| foo | bar").len(), 2);
        assert_eq!(TableScanner::scan_row("| foo | bar |").len(), 2);
        assert_eq!(TableScanner::scan_row("| | foo | bar | |").len(), 4);
        assert_eq!(TableScanner::scan_row("|").len(), 0);
        assert_eq!(TableScanner::scan_row("||").len(), 1);
    }

    #[test]
    fn should_trim_cell_content() {
        assert_eq!(TableScanner::scan_row("|foo|")[0].str, "foo");
        assert_eq!(TableScanner::scan_row("| foo |")[0].str, "foo");
        assert_eq!(TableScanner::scan_row("|\tfoo\t|")[0].str, "foo");
        assert_eq!(TableScanner::scan_row("| \t foo \t |")[0].str, "foo");
    }

    #[test]
    fn should_process_backslash_escapes() {
        assert_eq!(TableScanner::scan_row(r#"| foo\bar |"#)[0].str, r#"foo\bar"#);
        assert_eq!(TableScanner::scan_row(r#"| foo\|bar |"#)[0].str, r#"foo|bar"#);
        assert_eq!(TableScanner::scan_row(r#"| foo\\|bar |"#)[0].str, r#"foo\|bar"#);
        assert_eq!(TableScanner::scan_row(r#"| foo\\\|bar |"#)[0].str, r#"foo\\|bar"#);
        assert_eq!(TableScanner::scan_row(r#"| foo\\\\|bar |"#)[0].str, r#"foo\\\|bar"#);
    }

    #[test]
    fn should_trim_cell_content_srcmaps() {
        let row = TableScanner::scan_row("| foo | \tbar\t |");
        assert_eq!(row[0].str, "foo");
        assert_eq!(row[0].srcmap, vec![(0, 2)]);
        assert_eq!(row[1].str, "bar");
        assert_eq!(row[1].srcmap, vec![(0, 9)]);
    }

    #[test]
    fn should_process_backslash_escapes_srcmaps() {
        let row = TableScanner::scan_row(r#"|  foo\\|bar\\\|baz\  |"#);
        assert_eq!(row[0].str, r#"foo\|bar\\|baz\"#);
        assert_eq!(row[0].srcmap, vec![(0, 3), (4, 8), (10, 15)]);
    }

    #[test]
    fn require_pipe_or_colon_in_align_row() {
        let md = &mut crate::MarkdownIt::new();
        crate::plugins::extra::tables::add(md);
        let html = md.parse("foo\n---\nbar").render();
        assert_eq!(html.trim(), "foo\n---\nbar");
        let html = md.parse("|foo\n---\nbar").render();
        assert_eq!(html.trim(), "|foo\n---\nbar");
        let html = md.parse("foo\n|---\nbar").render();
        assert!(html.trim().starts_with("<table"));
        let html = md.parse("foo\n:---\nbar").render();
        assert!(html.trim().starts_with("<table"));
    }
}
