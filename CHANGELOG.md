# Changelog

## 0.6.0 - WIP

### Added

 - added `md.try_parse()` function which may return an error, as opposed to existing
   `md.parse()` function which never does

 - added optional `try_run()` trait function for rules which can fail and will
   propagate errors when using `md.try_parse()`

### Changed

 - `Node::walk_*` methods now return `Result`, which allows you to terminate traversing early
 - `syntext` rule now trims spaces in fence info string

### Migration

For all `Node::walk_*` methods change the following:

```rust
// replace this:
node.walk(|node, _| {
    dbg!(node);
});

// with this (unwrap is safe here because walk only
// returns error when your function does):
node.walk(|node, _| {
    dbg!(node);
    Ok(())
}).unwrap();
```

## 0.5.0 - 2023-05-13

### Added

 - typographer plugin (https://github.com/rlidwka/markdown-it.rs/pull/4)
 - smartquotes plugin (https://github.com/rlidwka/markdown-it.rs/pull/5)
 - headings with ids plugin (https://github.com/rlidwka/markdown-it.rs/pull/18)

### Changed

 - reference map changed from a HashMap to a trait object, allowing user to override it
   (https://github.com/rlidwka/markdown-it.rs/pull/17)

## 0.0.0 - 0.4.0 (2022-05-21 - 2022-10-03)

Initial commits. Software was not stabilized yet, so changes weren't documented that point.
