# markdown-it

[<img alt="github" src="https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/rlidwka/markdown-it.rs)
[<img alt="crates.io" src="https://img.shields.io/crates/v/markdown-it.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/markdown-it)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-not%20yet-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/markdown-it)
[<img alt="build status" src="https://img.shields.io/github/workflow/status/rlidwka/markdown-it.rs/CI?style=for-the-badge" height="20">](https://github.com/rlidwka/markdown-it.rs/actions/workflows/ci.yml?query=branch%3Amaster)
[<img alt="coverage" src="https://img.shields.io/codecov/c/github/rlidwka/markdown-it.rs?style=for-the-badge" height="20">](https://app.codecov.io/gh/rlidwka/markdown-it.rs)

Rust port of [markdown-it](https://github.com/markdown-it/markdown-it) javascript library.

**Work In Progress**

### Usage

```rust
let parser = &mut markdown_it::MarkdownIt::new();
markdown_it::plugins::cmark::add(parser);

let ast  = parser.parse("Hello **world**!");
let html = ast.render();

print!("{html}");
// prints "<p>Hello <strong>world</strong>!</p>"
```

The rest of the API is very experimental and will likely change tomorrow.
