# Changelog

## 0.6.0 - 2023-08-03

### Added

 - added link reference definition as AST node (renders as empty) for roundtripping
   (https://github.com/rlidwka/markdown-it.rs/pull/22)

### Changed

 - only set max indent=4 if `code` blocks plugin is enabled
   (https://github.com/rlidwka/markdown-it.rs/pull/20)

### Fixed

 - fixed ambiguity between tables and setext headings
   (https://github.com/rlidwka/markdown-it.rs/pull/27)

## 0.5.1 - 2023-07-05

### Fixed

 - fixed panics in smartquotes (https://github.com/rlidwka/markdown-it.rs/issues/26)
 - fixed entity code unescaping (https://github.com/rlidwka/markdown-it.rs/issues/23)
 - multiple other minor bugfixes

## 0.5.0 - 2023-05-13

### Added

 - typographer plugin (https://github.com/rlidwka/markdown-it.rs/pull/4)
 - smartquotes plugin (https://github.com/rlidwka/markdown-it.rs/pull/5)
 - headings with ids plugin (https://github.com/rlidwka/markdown-it.rs/pull/18)

### Changed

 - reference map changed from a HashMap to a trait object, allowing user to override it
   (https://github.com/rlidwka/markdown-it.rs/pull/17)

## 0.0.0 - 0.4.0 (2022-05-21 - 2022-10-03)

Initial commits. Software was not stabilized yet, so changes weren't documented at that point.
