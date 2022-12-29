# Headings with IDs

This example extension provides Markdown headings with IDs that can be used as
links to specific sections. As an example, with this extension this input...

```markdown
## Link to me plz
```

...provides this output:

```html
<h2 id="link-to-me-plz">Link to me plz</h2>
```

With the `id` in place, you can jump straight to the section using the
`#link-to-me-plz` subpath.

To run this example:

```bash
cargo run --example headings-with-ids
```
