[markdown-it](https://github.com/markdown-it/markdown-it) js library rewritten in rust

This is mostly a research project at this stage. Main goals are:

 - comparing speed of rust native code, rust wasm code, and javascript
 - check out how well rust code can be extended by plugins (spoiler alert: not that well)
 - clean up markdown-it.js original code with the aid of rust compiler (unused variables and such)

This is not a strict port, there are many improvements that can't be made in original js library for compatibility reasons.

### What works

100% of commonmark tests pass.

no plugins yet, but it's being worked on

### Usage

```rs
let md = markdown_it::MarkdownIt::new(None);
assert_eq!(md.render("Hello **world**!"), "<p>Hello <strong>world</strong>!</p>\n");
```

The rest of the API is very experimental and will likely change tomorrow.

### Benchmarks

On 10 kB test file:

 - `markdown-it`: 480 μs
 - `comrak`: 380 μs

so not good, but it's being worked on
