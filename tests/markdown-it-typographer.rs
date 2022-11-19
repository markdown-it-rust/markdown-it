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
    let node = md.parse(&(input.to_owned() + "\n"));

    // make sure we have sourcemaps for everything
    node.walk(|node, _| assert!(node.srcmap.is_some()));

    let result = node.render();
    assert_eq!(result, output);

    // make sure it doesn't crash without trailing \n
    let _ = md.parse(input.trim_end());
}
///////////////////////////////////////////////////////////////////////////
// TESTGEN: fixtures/markdown-it/typographer-extra.txt
#[rustfmt::skip]
mod fixtures_markdown_it_typographer_extra_txt {
use super::run;
// this part of the file is auto-generated
// don't edit it, otherwise your changes might be lost
#[test]
fn don_t_touch_text_in_autolinks() {
    let input = r#"URL with (C) (c) (R) (r) (TM) (tm): https://example.com/(c)(r)(tm)/(C)(R)(TM) what do you think?"#;
    let output = r#"<p>URL with © © ® ® ™ ™: <a href="https://example.com/(c)(r)(tm)/(C)(R)(TM)">https://example.com/(c)(r)(tm)/(C)(R)(TM)</a> what do you think?</p>"#;
    run(input, output);
}

#[test]
fn replacements_for_tm_should_allow_mixed_case_tm_and_tm() {
    let input = r#"These two should both end up the same as (TM) and (tm): (tM), (Tm)."#;
    let output = r#"<p>These two should both end up the same as ™ and ™: ™, ™.</p>"#;
    run(input, output);
}
// end of auto-generated module
}
///////////////////////////////////////////////////////////////////////////
// TESTGEN: fixtures/markdown-it/typographer.txt
#[rustfmt::skip]
mod fixtures_markdown_it_typographer_txt {
use super::run;
// this part of the file is auto-generated
// don't edit it, otherwise your changes might be lost
#[test]
fn unnamed() {
    let input = r#"(bad)"#;
    let output = r#"<p>(bad)</p>"#;
    run(input, output);
}

#[test]
fn copyright() {
    let input = r#"(c) (C)"#;
    let output = r#"<p>© ©</p>"#;
    run(input, output);
}

#[test]
fn reserved() {
    let input = r#"(r) (R)"#;
    let output = r#"<p>® ®</p>"#;
    run(input, output);
}

#[test]
fn trademark() {
    let input = r#"(tm) (TM)"#;
    let output = r#"<p>™ ™</p>"#;
    run(input, output);
}

#[test]
fn plus_minus() {
    let input = r#"+-5"#;
    let output = r#"<p>±5</p>"#;
    run(input, output);
}

#[test]
fn ellipsis() {
    let input = r#"test.. test... test..... test?..... test!...."#;
    let output = r#"<p>test… test… test… test?.. test!..</p>"#;
    run(input, output);
}

#[test]
fn dupes() {
    let input = r#"!!!!!! ???? ,,"#;
    let output = r#"<p>!!! ??? ,</p>"#;
    run(input, output);
}

#[test]
fn copyright_should_be_escapable() {
    let input = r#"\(c)"#;
    let output = r#"<p>(c)</p>"#;
    run(input, output);
}

#[test]
fn shouldn_t_replace_entities() {
    let input = r#"&#40;c) (c&#41; (c)"#;
    let output = r#"<p>(c) (c) ©</p>"#;
    run(input, output);
}

#[test]
fn dashes() {
    let input = r#"---markdownit --- super---

markdownit---awesome

abc ----

--markdownit -- super--

markdownit--awesome"#;
    let output = r#"<p>—markdownit — super—</p>
<p>markdownit—awesome</p>
<p>abc ----</p>
<p>–markdownit – super–</p>
<p>markdownit–awesome</p>"#;
    run(input, output);
}

#[test]
fn dashes_should_be_escapable() {
    let input = r#"foo \-- bar

foo -\- bar"#;
    let output = r#"<p>foo -- bar</p>
<p>foo -- bar</p>"#;
    run(input, output);
}

#[test]
fn regression_tests_for_624() {
    let input = r#"1---2---3

1--2--3

1 -- -- 3"#;
    let output = r#"<p>1—2—3</p>
<p>1–2–3</p>
<p>1 – – 3</p>"#;
    run(input, output);
}
// end of auto-generated module
}
