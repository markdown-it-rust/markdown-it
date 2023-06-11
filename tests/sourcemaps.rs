use markdown_it::Node;
use markdown_it::common::sourcemap::SourceWithLineStarts;

fn run(input: &str, f: fn (&Node, SourceWithLineStarts)) {
    let md = &mut markdown_it::MarkdownIt::new();
    markdown_it::plugins::cmark::add(md);
    markdown_it::plugins::html::add(md);
    let node = md.parse(input);
    node.walk(|node, _| {
        assert!(node.srcmap.is_some());
        Ok(())
    }).unwrap();
    f(&node, SourceWithLineStarts::new(input));
}

fn getmap(node: &Node, map: &SourceWithLineStarts) -> ((u32, u32), (u32, u32)) {
    node.srcmap.unwrap().get_positions(map)
}

#[test]
fn paragraph() {
    // same as commonmark.js
    run("foo   \n     \n     \n\n  barbaz\n\tquux   \n", |node, map| {
        assert_eq!(
            getmap(&node.children[0], &map),
            ((1, 1), (1, 6)),
        );
        assert_eq!(
            getmap(&node.children[1], &map),
            ((5, 3), (6, 8)),
        );
    });
}

#[test]
fn hr() {
    // same as commonmark.js
    run(" ---  \n\n  * * *\n", |node, map| {
        assert_eq!(
            getmap(&node.children[0], &map),
            ((1, 2), (1, 6)),
        );
        assert_eq!(
            getmap(&node.children[1], &map),
            ((3, 3), (3, 7)),
        );
    });
}

#[test]
fn heading() {
    // same as commonmark.js
    run("  \n  ### foo ###  \n\n", |node, map| {
        assert_eq!(
            getmap(&node.children[0], &map),
            ((2, 3), (2, 15)),
        );
    });

    run("  #\n", |node, map| {
        assert_eq!(
            getmap(&node.children[0], &map),
            ((1, 3), (1, 3)),
        );
    });
}

#[test]
fn lheading() {
    // same as commonmark.js
    run("  foo\n bar\n ----\n\n", |node, map| {
        assert_eq!(
            getmap(&node.children[0], &map),
            ((1, 3), (3, 5)),
        );
    });
}

#[test]
fn fence() {
    // same as commonmark.js
    run("  ~~~ foo ~~~\n", |node, map| {
        assert_eq!(
            getmap(&node.children[0], &map),
            ((1, 3), (1, 13)),
        );
    });

    run("  ```\n 12\n", |node, map| {
        assert_eq!(
            getmap(&node.children[0], &map),
            ((1, 3), (2, 3)),
        );
    });

    run("```\n\n\n\n", |node, map| {
        assert_eq!(
            getmap(&node.children[0], &map),
            ((1, 1), (4, 0)),
        );
    });

    run("~~~\na\nb\n~~~  \nc\n", |node, map| {
        assert_eq!(
            getmap(&node.children[0], &map),
            ((1, 1), (4, 5)),
        );
    });
}

#[test]
fn html_block() {
    // same as commonmark.js
    run("  <div>\n", |node, map| {
        assert_eq!(
            getmap(&node.children[0], &map),
            ((1, 3), (1, 7)),
        );
    });

    run("<div>\n</div>  \n", |node, map| {
        assert_eq!(
            getmap(&node.children[0], &map),
            ((1, 1), (2, 8)),
        );
    });
}

#[test]
fn code_block() {
    // this should be (1, 5), (1, 9)
    // for simplicity, we point source maps for block tags to first
    // nonspace character, but it isn't quite correct for code blocks
    run("      foo\n", |node, map| {
        assert_eq!(
            getmap(&node.children[0], &map),
            ((1, 7), (1, 9)),
        );
    });

    run("   a\n    b\n     c\n", |node, map| {
        assert_eq!(
            getmap(&node.children[0], &map),
            ((1, 4), (3, 6)),
        );
    });

    // this I believe to be error in commonmark, code block
    // only have 1 line as per spec, but cmark reports 3 lines
    run("    foobar  \n    \n    \n\nbar\n", |node, map| {
        assert_eq!(
            getmap(&node.children[0], &map),
            ((1, 5), (1, 12)),
        );
    });
}

#[test]
fn blockquotes() {
    // same as commonmark.js
    run("  > foo  \n", |node, map| {
        assert_eq!(
            getmap(&node.children[0], &map),
            ((1, 3), (1, 9)),
        );
    });

    run("> foo\nbar\n\n", |node, map| {
        assert_eq!(
            getmap(&node.children[0], &map),
            ((1, 1), (2, 3)),
        );
    });
}

#[test]
fn lists() {
    // same as commonmark.js
    run(" 1. foo\n 2. bar\n", |node, map| {
        assert_eq!(
            getmap(&node.children[0], &map),
            ((1, 2), (2, 7)),
        );

        assert_eq!(
            getmap(&node.children[0].children[0], &map),
            ((1, 2), (1, 7)),
        );
    });

    run(" - foo\n\n - bar\n", |node, map| {
        assert_eq!(
            getmap(&node.children[0], &map),
            ((1, 2), (3, 6)),
        );

        assert_eq!(
            getmap(&node.children[0].children[0], &map),
            ((1, 2), (2, 0)),
        );

        assert_eq!(
            getmap(&node.children[0].children[1], &map),
            ((3, 2), (3, 6)),
        );
    });
}

#[test]
fn autolinks() {
    run("foo <http://google.com> bar", |node, map| {
        assert_eq!(
            getmap(&node.children[0].children[1], &map),
            ((1, 5), (1, 23)),
        );

        assert_eq!(
            getmap(&node.children[0].children[1].children[0], &map),
            ((1, 6), (1, 22)),
        );
    });
}

#[test]
fn emphasis() {
    run("***foo***", |node, map| {
        assert_eq!(
            getmap(&node.children[0].children[0], &map),
            ((1, 1), (1, 9)),
        );

        assert_eq!(
            getmap(&node.children[0].children[0].children[0], &map),
            ((1, 2), (1, 8)),
        );
    });

    run("aaa **bb _cc_ dd** eee", |node, map| {
        assert_eq!(
            getmap(&node.children[0].children[1], &map),
            ((1, 5), (1, 18)),
        );

        assert_eq!(
            getmap(&node.children[0].children[1].children[1], &map),
            ((1, 10), (1, 13)),
        );
    });
}

#[test]
fn newline() {
    run("foo  \nbar \nbaz\nquux", |node, map| {
        assert_eq!(
            getmap(&node.children[0].children[1], &map),
            ((1, 4), (2, 0)),
        );

        assert_eq!(
            getmap(&node.children[0].children[3], &map),
            ((2, 4), (3, 0)),
        );

        assert_eq!(
            getmap(&node.children[0].children[5], &map),
            ((4, 0), (4, 0)),
        );

        /*let marks : Vec<_> = node.children[0].children.iter().map(|x| getmap(x, &map)).collect();
        assert_eq!(marks, [
            ((1, 1), (1, 5)),
            ((2, 0), (2, 1)),
            ((2, 1), (2, 3)),
            ((3, 0), (3, 0)),
            ((3, 1), (3, 3)),
        ]);*/
    });
}

#[test]
fn escapes() {
    run("foo\\Δ\\*bar", |node, map| {
        assert_eq!(
            getmap(&node.children[0].children[1], &map),
            ((1, 4), (1, 5)),
        );

        assert_eq!(
            getmap(&node.children[0].children[2], &map),
            ((1, 6), (1, 7)),
        );
    });

    run("  foo  \\\n  bar  ", |node, map| {
        assert_eq!(
            getmap(&node.children[0].children[1], &map),
            ((1, 8), (2, 2)),
        );
    });
}

#[test]
fn entities() {
    run("aa &nbsp; bb &#20; cc", |node, map| {
        assert_eq!(
            getmap(&node.children[0].children[1], &map),
            ((1, 4), (1, 9)),
        );

        assert_eq!(
            getmap(&node.children[0].children[3], &map),
            ((1, 14), (1, 18)),
        );

        /*let marks : Vec<_> = node.children[0].children.iter().map(|x| getmap(x, &map)).collect();
        assert_eq!(marks, [
            ((1, 1), (1, 5)),
            ((2, 0), (2, 1)),
            ((2, 1), (2, 3)),
            ((3, 0), (3, 0)),
            ((3, 1), (3, 3)),
        ]);*/
    });
}

#[test]
fn html_inline() {
    run("foo <bar> baz", |node, map| {
        assert_eq!(
            getmap(&node.children[0].children[1], &map),
            ((1, 5), (1, 9)),
        );
    });
}

#[test]
fn backticks() {
    run("foo ```bar``` baz", |node, map| {
        assert_eq!(
            getmap(&node.children[0].children[1], &map),
            ((1, 5), (1, 13)),
        );

        assert_eq!(
            getmap(&node.children[0].children[1].children[0], &map),
            ((1, 8), (1, 10)),
        );
    });

    run("foo ` bar ` baz", |node, map| {
        assert_eq!(
            getmap(&node.children[0].children[1], &map),
            ((1, 5), (1, 11)),
        );

        assert_eq!(
            getmap(&node.children[0].children[1].children[0], &map),
            ((1, 7), (1, 9)),
        );
    });
}

#[test]
fn imglink() {
    run("foo [bar](baz) quux", |node, map| {
        assert_eq!(
            getmap(&node.children[0].children[1], &map),
            ((1, 5), (1, 14)),
        );
    });

    run("foo ![bar](baz) quux", |node, map| {
        assert_eq!(
            getmap(&node.children[0].children[1], &map),
            ((1, 5), (1, 15)),
        );
    });
}
