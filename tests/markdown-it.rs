
fn run(input: &str, output: &str) {
    let output = if output.is_empty() { "".to_owned() } else { output.to_owned() + "\n" };
    let md = &mut markdown_it::MarkdownIt::new();
    markdown_it::plugins::cmark::add(md);
    markdown_it::plugins::html::add(md);
    markdown_it::plugins::extra::tables::add(md);
    let node = md.parse(&(input.to_owned() + "\n"));

    // make sure we have sourcemaps for everything
    node.walk(|node, _| assert!(node.srcmap.is_some()));

    let result = node.render();
    assert_eq!(result, output);

    // make sure it doesn't crash without trailing \n
    let _ = md.parse(input.trim_end());
}

///////////////////////////////////////////////////////////////////////////
// TESTGEN: fixtures/markdown-it/tables.txt
#[rustfmt::skip]
mod fixtures_markdown_it_tables_txt {
use super::run;
// this part of the file is auto-generated
// don't edit it, otherwise your changes might be lost
#[test]
fn simple() {
    let input = r#"| Heading 1 | Heading 2
| --------- | ---------
| Cell 1    | Cell 2
| Cell 3    | Cell 4"#;
    let output = r#"<table>
<thead>
<tr>
<th>Heading 1</th>
<th>Heading 2</th>
</tr>
</thead>
<tbody>
<tr>
<td>Cell 1</td>
<td>Cell 2</td>
</tr>
<tr>
<td>Cell 3</td>
<td>Cell 4</td>
</tr>
</tbody>
</table>"#;
    run(input, output);
}

#[test]
fn column_alignment() {
    let input = r#"| Header 1 | Header 2 | Header 3 | Header 4 |
| :------: | -------: | :------- | -------- |
| Cell 1   | Cell 2   | Cell 3   | Cell 4   |
| Cell 5   | Cell 6   | Cell 7   | Cell 8   |"#;
    let output = r#"<table>
<thead>
<tr>
<th style="text-align:center">Header 1</th>
<th style="text-align:right">Header 2</th>
<th style="text-align:left">Header 3</th>
<th>Header 4</th>
</tr>
</thead>
<tbody>
<tr>
<td style="text-align:center">Cell 1</td>
<td style="text-align:right">Cell 2</td>
<td style="text-align:left">Cell 3</td>
<td>Cell 4</td>
</tr>
<tr>
<td style="text-align:center">Cell 5</td>
<td style="text-align:right">Cell 6</td>
<td style="text-align:left">Cell 7</td>
<td>Cell 8</td>
</tr>
</tbody>
</table>"#;
    run(input, output);
}

#[test]
fn nested_emphases() {
    let input = r#"Header 1|Header 2|Header 3|Header 4
:-------|:------:|-------:|--------
Cell 1  |Cell 2  |Cell 3  |Cell 4
*Cell 5*|Cell 6  |Cell 7  |Cell 8"#;
    let output = r#"<table>
<thead>
<tr>
<th style="text-align:left">Header 1</th>
<th style="text-align:center">Header 2</th>
<th style="text-align:right">Header 3</th>
<th>Header 4</th>
</tr>
</thead>
<tbody>
<tr>
<td style="text-align:left">Cell 1</td>
<td style="text-align:center">Cell 2</td>
<td style="text-align:right">Cell 3</td>
<td>Cell 4</td>
</tr>
<tr>
<td style="text-align:left"><em>Cell 5</em></td>
<td style="text-align:center">Cell 6</td>
<td style="text-align:right">Cell 7</td>
<td>Cell 8</td>
</tr>
</tbody>
</table>"#;
    run(input, output);
}

#[test]
fn nested_tables_inside_blockquotes() {
    let input = r#"> foo|foo
> ---|---
> bar|bar
baz|baz"#;
    let output = r#"<blockquote>
<table>
<thead>
<tr>
<th>foo</th>
<th>foo</th>
</tr>
</thead>
<tbody>
<tr>
<td>bar</td>
<td>bar</td>
</tr>
</tbody>
</table>
</blockquote>
<p>baz|baz</p>"#;
    run(input, output);
}

#[test]
fn minimal_one_column() {
    let input = r#"| foo
|----
| test2"#;
    let output = r#"<table>
<thead>
<tr>
<th>foo</th>
</tr>
</thead>
<tbody>
<tr>
<td>test2</td>
</tr>
</tbody>
</table>"#;
    run(input, output);
}

#[test]
fn this_is_parsed_as_one_big_table() {
    let input = r#"-   foo|foo
---|---
bar|bar"#;
    let output = r#"<table>
<thead>
<tr>
<th>-   foo</th>
<th>foo</th>
</tr>
</thead>
<tbody>
<tr>
<td>bar</td>
<td>bar</td>
</tr>
</tbody>
</table>"#;
    run(input, output);
}

#[test]
fn second_line_should_not_contain_symbols_except_and() {
    let input = r#"foo|foo
-----|-----s
bar|bar"#;
    let output = r#"<p>foo|foo
-----|-----s
bar|bar</p>"#;
    run(input, output);
}

#[test]
fn second_line_should_contain_symbol() {
    let input = r#"foo|foo
-----:-----
bar|bar"#;
    let output = r#"<p>foo|foo
-----:-----
bar|bar</p>"#;
    run(input, output);
}

#[test]
fn second_line_should_not_have_empty_columns_in_the_middle() {
    let input = r#"foo|foo
-----||-----
bar|bar"#;
    let output = r#"<p>foo|foo
-----||-----
bar|bar</p>"#;
    run(input, output);
}

#[test]
fn wrong_alignment_symbol_position() {
    let input = r#"foo|foo
-----|-::-
bar|bar"#;
    let output = r#"<p>foo|foo
-----|-::-
bar|bar</p>"#;
    run(input, output);
}

#[test]
fn title_line_should_contain_symbol() {
    let input = r#"foo
-----|-----
bar|bar"#;
    let output = r#"<p>foo
-----|-----
bar|bar</p>"#;
    run(input, output);
}

#[test]
fn allow_tabs_as_a_separator_on_2nd_line() {
    let input = "|\tfoo\t|\tbar\t|
|\t---\t|\t---\t|
|\tbaz\t|\tquux\t|";
    let output = r#"<table>
<thead>
<tr>
<th>foo</th>
<th>bar</th>
</tr>
</thead>
<tbody>
<tr>
<td>baz</td>
<td>quux</td>
</tr>
</tbody>
</table>"#;
    run(input, output);
}

#[test]
fn should_terminate_paragraph() {
    let input = r#"paragraph
foo|foo
---|---
bar|bar"#;
    let output = r#"<p>paragraph</p>
<table>
<thead>
<tr>
<th>foo</th>
<th>foo</th>
</tr>
</thead>
<tbody>
<tr>
<td>bar</td>
<td>bar</td>
</tr>
</tbody>
</table>"#;
    run(input, output);
}

#[test]
fn another_complicated_backticks_case() {
    let input = r#"| Heading 1 | Heading 2
| --------- | ---------
| Cell 1 | Cell 2
| \\\`|\\\`"#;
    let output = r#"<table>
<thead>
<tr>
<th>Heading 1</th>
<th>Heading 2</th>
</tr>
</thead>
<tbody>
<tr>
<td>Cell 1</td>
<td>Cell 2</td>
</tr>
<tr>
<td>\`</td>
<td>\`</td>
</tr>
</tbody>
</table>"#;
    run(input, output);
}

#[test]
fn in_tables_should_not_count_as_escaped_backtick() {
    let input = r#"# | 1 | 2
--|--|--
x | `\` | `x`"#;
    let output = r#"<table>
<thead>
<tr>
<th>#</th>
<th>1</th>
<th>2</th>
</tr>
</thead>
<tbody>
<tr>
<td>x</td>
<td><code>\</code></td>
<td><code>x</code></td>
</tr>
</tbody>
</table>"#;
    run(input, output);
}

#[test]
fn tables_should_handle_escaped_backticks() {
    let input = r#"# | 1 | 2
--|--|--
x | \`\` | `x`"#;
    let output = r#"<table>
<thead>
<tr>
<th>#</th>
<th>1</th>
<th>2</th>
</tr>
</thead>
<tbody>
<tr>
<td>x</td>
<td>``</td>
<td><code>x</code></td>
</tr>
</tbody>
</table>"#;
    run(input, output);
}

#[test]
fn an_amount_of_rows_might_be_different_across_the_table_issue_171() {
    let input = r#"| 1 | 2 |
| :-----: |  :-----: |
| 3 | 4 | 5 | 6 |"#;
    let output = r#"<table>
<thead>
<tr>
<th style="text-align:center">1</th>
<th style="text-align:center">2</th>
</tr>
</thead>
<tbody>
<tr>
<td style="text-align:center">3</td>
<td style="text-align:center">4</td>
</tr>
</tbody>
</table>"#;
    run(input, output);
}

#[test]
fn an_amount_of_rows_might_be_different_across_the_table_2() {
    let input = r#"| 1 | 2 | 3 | 4 |
| :-----: |  :-----: |  :-----: |  :-----: |
| 5 | 6 |"#;
    let output = r#"<table>
<thead>
<tr>
<th style="text-align:center">1</th>
<th style="text-align:center">2</th>
<th style="text-align:center">3</th>
<th style="text-align:center">4</th>
</tr>
</thead>
<tbody>
<tr>
<td style="text-align:center">5</td>
<td style="text-align:center">6</td>
<td style="text-align:center"></td>
<td style="text-align:center"></td>
</tr>
</tbody>
</table>"#;
    run(input, output);
}

#[test]
fn allow_one_column_tables_issue_171() {
    let input = r#"| foo |
:-----:
| bar |"#;
    let output = r#"<table>
<thead>
<tr>
<th style="text-align:center">foo</th>
</tr>
</thead>
<tbody>
<tr>
<td style="text-align:center">bar</td>
</tr>
</tbody>
</table>"#;
    run(input, output);
}

#[test]
fn allow_indented_tables_issue_325() {
    let input = r#"  | Col1a | Col2a |
  | ----- | ----- |
  | Col1b | Col2b |"#;
    let output = r#"<table>
<thead>
<tr>
<th>Col1a</th>
<th>Col2a</th>
</tr>
</thead>
<tbody>
<tr>
<td>Col1b</td>
<td>Col2b</td>
</tr>
</tbody>
</table>"#;
    run(input, output);
}

#[test]
fn tables_should_not_be_indented_more_than_4_spaces_1st_line() {
    let input = r#"    | Col1a | Col2a |
  | ----- | ----- |
  | Col1b | Col2b |"#;
    let output = r#"<pre><code>| Col1a | Col2a |
</code></pre>
<p>| ----- | ----- |
| Col1b | Col2b |</p>"#;
    run(input, output);
}

#[test]
fn tables_should_not_be_indented_more_than_4_spaces_2nd_line() {
    let input = r#"  | Col1a | Col2a |
    | ----- | ----- |
  | Col1b | Col2b |"#;
    let output = r#"<p>| Col1a | Col2a |
| ----- | ----- |
| Col1b | Col2b |</p>"#;
    run(input, output);
}

#[test]
fn tables_should_not_be_indented_more_than_4_spaces_3rd_line() {
    let input = r#"  | Col1a | Col2a |
  | ----- | ----- |
    | Col1b | Col2b |"#;
    let output = r#"<table>
<thead>
<tr>
<th>Col1a</th>
<th>Col2a</th>
</tr>
</thead>
</table>
<pre><code>| Col1b | Col2b |
</code></pre>"#;
    run(input, output);
}

#[test]
fn allow_tables_with_empty_body() {
    let input = r#"  | Col1a | Col2a |
  | ----- | ----- |"#;
    let output = r#"<table>
<thead>
<tr>
<th>Col1a</th>
<th>Col2a</th>
</tr>
</thead>
</table>"#;
    run(input, output);
}

#[test]
fn align_row_should_be_at_least_as_large_as_any_actual_rows() {
    let input = r#"Col1a | Col1b | Col1c
----- | -----
Col2a | Col2b | Col2c"#;
    let output = r#"<p>Col1a | Col1b | Col1c
----- | -----
Col2a | Col2b | Col2c</p>"#;
    run(input, output);
}

#[test]
fn escaped_pipes_inside_backticks_don_t_split_cells() {
    let input = r#"| Heading 1 | Heading 2
| --------- | ---------
| Cell 1 | Cell 2
| `Cell 3\|` | Cell 4"#;
    let output = r#"<table>
<thead>
<tr>
<th>Heading 1</th>
<th>Heading 2</th>
</tr>
</thead>
<tbody>
<tr>
<td>Cell 1</td>
<td>Cell 2</td>
</tr>
<tr>
<td><code>Cell 3|</code></td>
<td>Cell 4</td>
</tr>
</tbody>
</table>"#;
    run(input, output);
}

#[test]
fn escape_before_escaped_pipes_inside_backticks_don_t_split_cells() {
    let input = r#"| Heading 1 | Heading 2
| --------- | ---------
| Cell 1 | Cell 2
| `Cell 3\\|` | Cell 4"#;
    let output = r#"<table>
<thead>
<tr>
<th>Heading 1</th>
<th>Heading 2</th>
</tr>
</thead>
<tbody>
<tr>
<td>Cell 1</td>
<td>Cell 2</td>
</tr>
<tr>
<td><code>Cell 3\|</code></td>
<td>Cell 4</td>
</tr>
</tbody>
</table>"#;
    run(input, output);
}

#[test]
fn regression_test_for_721_table_in_a_list_indented_with_tabs() {
    let input = "- Level 1

\t- Level 2

\t\t| Column 1 | Column 2 |
\t\t| -------- | -------- |
\t\t| abcdefgh | ijklmnop |";
    let output = r#"<ul>
<li>
<p>Level 1</p>
<ul>
<li>
<p>Level 2</p>
<table>
<thead>
<tr>
<th>Column 1</th>
<th>Column 2</th>
</tr>
</thead>
<tbody>
<tr>
<td>abcdefgh</td>
<td>ijklmnop</td>
</tr>
</tbody>
</table>
</li>
</ul>
</li>
</ul>"#;
    run(input, output);
}

#[test]
fn table_without_any_columns_is_not_a_table_724() {
    let input = r#"|
|
|"#;
    let output = r#"<p>|
|
|</p>"#;
    run(input, output);
}

#[test]
fn gfm_4_10_tables_extension_example_198() {
    let input = r#"| foo | bar |
| --- | --- |
| baz | bim |"#;
    let output = r#"<table>
<thead>
<tr>
<th>foo</th>
<th>bar</th>
</tr>
</thead>
<tbody>
<tr>
<td>baz</td>
<td>bim</td>
</tr>
</tbody>
</table>"#;
    run(input, output);
}

#[test]
fn gfm_4_10_tables_extension_example_199() {
    let input = r#"| abc | defghi |
:-: | -----------:
bar | baz"#;
    let output = r#"<table>
<thead>
<tr>
<th style="text-align:center">abc</th>
<th style="text-align:right">defghi</th>
</tr>
</thead>
<tbody>
<tr>
<td style="text-align:center">bar</td>
<td style="text-align:right">baz</td>
</tr>
</tbody>
</table>"#;
    run(input, output);
}

#[test]
fn gfm_4_10_tables_extension_example_200() {
    let input = r#"| f\|oo  |
| ------ |
| b `\|` az |
| b **\|** im |"#;
    let output = r#"<table>
<thead>
<tr>
<th>f|oo</th>
</tr>
</thead>
<tbody>
<tr>
<td>b <code>|</code> az</td>
</tr>
<tr>
<td>b <strong>|</strong> im</td>
</tr>
</tbody>
</table>"#;
    run(input, output);
}

#[test]
fn gfm_4_10_tables_extension_example_201() {
    let input = r#"| abc | def |
| --- | --- |
| bar | baz |
> bar"#;
    let output = r#"<table>
<thead>
<tr>
<th>abc</th>
<th>def</th>
</tr>
</thead>
<tbody>
<tr>
<td>bar</td>
<td>baz</td>
</tr>
</tbody>
</table>
<blockquote>
<p>bar</p>
</blockquote>"#;
    run(input, output);
}

#[test]
fn gfm_4_10_tables_extension_example_202() {
    let input = r#"| abc | def |
| --- | --- |
| bar | baz |
bar

bar"#;
    let output = r#"<table>
<thead>
<tr>
<th>abc</th>
<th>def</th>
</tr>
</thead>
<tbody>
<tr>
<td>bar</td>
<td>baz</td>
</tr>
<tr>
<td>bar</td>
<td></td>
</tr>
</tbody>
</table>
<p>bar</p>"#;
    run(input, output);
}

#[test]
fn gfm_4_10_tables_extension_example_203() {
    let input = r#"| abc | def |
| --- |
| bar |"#;
    let output = r#"<p>| abc | def |
| — |
| bar |</p>"#;
    run(input, output);
}

#[test]
fn gfm_4_10_tables_extension_example_204() {
    let input = r#"| abc | def |
| --- | --- |
| bar |
| bar | baz | boo |"#;
    let output = r#"<table>
<thead>
<tr>
<th>abc</th>
<th>def</th>
</tr>
</thead>
<tbody>
<tr>
<td>bar</td>
<td></td>
</tr>
<tr>
<td>bar</td>
<td>baz</td>
</tr>
</tbody>
</table>"#;
    run(input, output);
}

#[test]
fn gfm_4_10_tables_extension_example_205() {
    let input = r#"| abc | def |
| --- | --- |"#;
    let output = r#"<table>
<thead>
<tr>
<th>abc</th>
<th>def</th>
</tr>
</thead>
</table>"#;
    run(input, output);
}

#[test]
fn a_list_takes_precedence_in_case_of_ambiguity() {
    let input = r#"a | b
- | -
1 | 2"#;
    let output = r#"<p>a | b</p>
<ul>
<li>| -
1 | 2</li>
</ul>"#;
    run(input, output);
}
// end of auto-generated module
}
