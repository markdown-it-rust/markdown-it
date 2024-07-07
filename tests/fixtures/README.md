This file generates tests for markdown-it rust library. It gets one argument
(file name) and edits that file in-place, replacing special markers inside
with tests generated from fixtures.

Run it like this from tests/ folder:
```sh
for I in *.rs ; do deno run --allow-read --allow-write ./fixtures/testgen.js $I ; done
```

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
