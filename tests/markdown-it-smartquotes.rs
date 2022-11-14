fn run(input: &str, output: &str) {
    let output = if output.is_empty() {
        "".to_owned()
    } else {
        output.to_owned() + "\n"
    };
    let md = &mut markdown_it::MarkdownIt::new();
    markdown_it::plugins::cmark::add(md);
    markdown_it::plugins::html::add(md);
    markdown_it::plugins::extra::linkify::add(md);
    markdown_it::plugins::extra::typographer::add(md);
    markdown_it::plugins::extra::smartquotes::add(md);
    let node = md.parse(&(input.to_owned() + "\n"));

    // make sure we have sourcemaps for everything
    node.walk(|node, _| assert!(node.srcmap.is_some()));

    let result = node.render();
    assert_eq!(result, output);

    // make sure it doesn't crash without trailing \n
    let _ = md.parse(input.trim_end());
}
///////////////////////////////////////////////////////////////////////////
// TESTGEN: fixtures/markdown-it/smartquotes.txt
#[rustfmt::skip]
mod fixtures_markdown_it_smartquotes_txt {
use super::run;
// this part of the file is auto-generated
// don't edit it, otherwise your changes might be lost
#[test]
fn should_parse_nested_quotes() {
    let input = r#""foo 'bar' baz"

'foo 'bar' baz'"#;
    let output = r#"<p>“foo ‘bar’ baz”</p>
<p>‘foo ‘bar’ baz’</p>"#;
    run(input, output);
}

#[test]
fn should_not_overlap_quotes() {
    let input = r#"'foo "bar' baz""#;
    let output = r#"<p>‘foo &quot;bar’ baz&quot;</p>"#;
    run(input, output);
}

#[test]
fn should_match_quotes_on_the_same_level() {
    let input = r#""foo *bar* baz""#;
    let output = r#"<p>“foo <em>bar</em> baz”</p>"#;
    run(input, output);
}

#[test]
fn should_handle_adjacent_nested_quotes() {
    let input = r#"'"double in single"'

"'single in double'""#;
    let output = r#"<p>‘“double in single”’</p>
<p>“‘single in double’”</p>"#;
    run(input, output);
}

#[test]
fn should_not_match_quotes_on_different_levels() {
    let input = r#"*"foo* bar"

"foo *bar"*

*"foo* bar *baz"*"#;
    let output = r#"<p><em>&quot;foo</em> bar&quot;</p>
<p>&quot;foo <em>bar&quot;</em></p>
<p><em>&quot;foo</em> bar <em>baz&quot;</em></p>"#;
    run(input, output);
}

#[test]
fn smartquotes_should_not_overlap_with_other_tags() {
    let input = r#"*foo "bar* *baz" quux*"#;
    let output = r#"<p><em>foo &quot;bar</em> <em>baz&quot; quux</em></p>"#;
    run(input, output);
}

#[test]
fn should_try_and_find_matching_quote_in_this_case() {
    let input = r#""foo "bar 'baz""#;
    let output = r#"<p>&quot;foo “bar 'baz”</p>"#;
    run(input, output);
}

#[test]
fn should_not_touch_inches_in_quotes() {
    let input = r#""Monitor 21"" and "Monitor"""#;
    let output = r#"<p>“Monitor 21&quot;” and “Monitor”&quot;</p>"#;
    run(input, output);
}

#[test]
fn should_render_an_apostrophe_as_a_rsquo() {
    let input = r#"This isn't and can't be the best approach to implement this..."#;
    let output = r#"<p>This isn’t and can’t be the best approach to implement this…</p>"#;
    run(input, output);
}

#[test]
fn apostrophe_could_end_the_word_that_s_why_original_smartypants_replaces_all_of_them_as_rsquo() {
    let input = r#"users' stuff"#;
    let output = r#"<p>users’ stuff</p>"#;
    run(input, output);
}

#[test]
fn quotes_between_punctuation_chars() {
    let input = r#""(hai)"."#;
    let output = r#"<p>“(hai)”.</p>"#;
    run(input, output);
}

#[test]
fn quotes_at_the_start_end_of_the_tokens() {
    let input = r#""*foo* bar"

"foo *bar*"

"*foo bar*""#;
    let output = r#"<p>“<em>foo</em> bar”</p>
<p>“foo <em>bar</em>”</p>
<p>“<em>foo bar</em>”</p>"#;
    run(input, output);
}

#[test]
fn should_treat_softbreak_as_a_space() {
    let input = r#""this"
and "that".

"this" and
"that"."#;
    let output = r#"<p>“this”
and “that”.</p>
<p>“this” and
“that”.</p>"#;
    run(input, output);
}

#[test]
fn should_treat_hardbreak_as_a_space() {
    let input = r#""this"\
and "that".

"this" and\
"that"."#;
    let output = r#"<p>“this”<br>
and “that”.</p>
<p>“this” and<br>
“that”.</p>"#;
    run(input, output);
}

#[test]
fn should_allow_quotes_adjacent_to_other_punctuation_characters_643() {
    let input = r#"The dog---"'man's' best friend""#;
    let output = r#"<p>The dog—“‘man’s’ best friend”</p>"#;
    run(input, output);
}

#[test]
fn should_parse_quotes_adjacent_to_code_block_677() {
    let input = r#""test `code`"

"`code` test""#;
    let output = r#"<p>“test <code>code</code>”</p>
<p>“<code>code</code> test”</p>"#;
    run(input, output);
}

#[test]
fn should_parse_quotes_adjacent_to_inline_html_677() {
    let input = r#""test <br>"

"<br> test""#;
    let output = r#"<p>“test <br>”</p>
<p>“<br> test”</p>"#;
    run(input, output);
}

#[test]
fn should_be_escapable() {
    let input = r#""foo"

\"foo"

"foo\""#;
    let output = r#"<p>“foo”</p>
<p>&quot;foo&quot;</p>
<p>&quot;foo&quot;</p>"#;
    run(input, output);
}

#[test]
fn should_not_replace_entities() {
    let input = r#"&quot;foo&quot;

&quot;foo"

"foo&quot;"#;
    let output = r#"<p>&quot;foo&quot;</p>
<p>&quot;foo&quot;</p>
<p>&quot;foo&quot;</p>"#;
    run(input, output);
}
// end of auto-generated module
}
