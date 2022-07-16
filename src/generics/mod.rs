//! Use these to build your own markdown syntax.
//!
//! Some markdown structures are very similar under the hood, for example:
//!  - `*emphasis*`, `^supertext^` and `~~strikethrough~~`
//!  - `[link]()` and `![image]()`
//!
//! In order to reuse the code between all those, a notion of generic
//! markdown structures was created. If you want to use syntax like
//! `=this=` or `++that++`, you only need to specify a character marker
//! and a renderer function, these rules will figure out the rest.
//!
pub mod inline;
