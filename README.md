# markdown-it

[<img alt="github" src="https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/rlidwka/markdown-it.rs)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/markdown-it)
[<img alt="crates.io" src="https://img.shields.io/crates/v/markdown-it.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/markdown-it)
[<img alt="build status" src="https://img.shields.io/github/workflow/status/rlidwka/markdown-it.rs/tests?style=for-the-badge" height="20">](https://github.com/rlidwka/markdown-it.rs/actions/workflows/ci.yml?query=branch%3Amaster)
[<img alt="coverage" src="https://img.shields.io/codecov/c/github/rlidwka/markdown-it.rs?style=for-the-badge" height="20">](https://app.codecov.io/gh/rlidwka/markdown-it.rs)

Rust port of popular [markdown-it.js](https://github.com/markdown-it/markdown-it) library.

TL;DR:
 - if you want to get result *fast*, use [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark)
 - if you want to render GFM exactly like github, use [comrak](https://github.com/kivikakk/comrak)
 - if you want to define your own syntax (like `@mentions`, `:emoji:`, custom html classes), use this library

You can check a [demo](https://rlidwka.github.io/markdown-it.rs/) in your browser *(it's Rust compiled into WASM)*.

### Features

 - 100% CommonMark compatibility
 - AST
 - Source maps (full support, not just on block tags like cmark)
 - Ability to write your own syntax of arbitrary complexity
   - to prove this point, CommonMark syntax itself is written as a plugin

### Usage

```rust
let parser = &mut markdown_it::MarkdownIt::new();
markdown_it::plugins::cmark::add(parser);
markdown_it::plugins::extra::add(parser);

let ast  = parser.parse("Hello **world**!");
let html = ast.render();

print!("{html}");
// prints "<p>Hello <strong>world</strong>!</p>"
```

For a guide on how to extend it, see `examples` folder.

### Notes

*This is an attempt at making a language-agnostic parser. You can probably parse AsciiDoc, reStructuredText or [any other](https://github.com/mundimark/awesome-markdown-alternatives) plain text format with this without too much effort. I&nbsp;might eventually write these as proof-of-concept.*
