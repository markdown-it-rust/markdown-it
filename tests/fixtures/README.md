This file generates tests for markdown-it rust library. It gets one argument
(file name) and edits that file in-place, replacing special markers inside
with tests generated from fixtures.

As an example, it replaces this:
```rs
///////////////////////////////////////////////////////////////////////////
// TESTGEN: fixtures/commonmark/spec.txt
... any content here ...
///////////////////////////////////////////////////////////////////////////
```

With approximately this:
```rs
///////////////////////////////////////////////////////////////////////////
// TESTGEN: fixtures/commonmark/spec.txt
#[test]
fn line_123() {
    run("*foo*", "<em>foo</em>");
}
... more tests here ...
///////////////////////////////////////////////////////////////////////////
```
