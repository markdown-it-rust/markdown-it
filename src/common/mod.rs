//! Self-contained modules used for miscellaneous purposes.
//!
//! These are all candidates for being separated into different crates,
//! tell me if functionality they provide is useful enough to do that.

pub mod mdurl;
pub mod ruler;
pub mod sourcemap;
pub mod utils;

mod erasedset;
pub use erasedset::ErasedSet;

mod typekey;
pub use typekey::TypeKey;
