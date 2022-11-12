fn run(input: &str, output: &str) {
    let output = if output.is_empty() {
        "".to_owned()
    } else {
        output.to_owned() + "\n"
    };
    let md = &mut markdown_it::MarkdownIt::new();
    markdown_it::plugins::cmark::add(md);
    markdown_it::plugins::html::add(md);
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
