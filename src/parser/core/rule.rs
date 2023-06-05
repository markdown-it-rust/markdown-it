use std::fmt::Display;

use crate::common::TypeKey;
use crate::{MarkdownIt, Node, Result};

/// Each member of core rule chain must implement this trait
pub trait CoreRule : 'static {
    /// Perform arbitrary operations on the root Node.
    fn run(root: &mut Node, md: &MarkdownIt);

    /// Same as `run()`, but used for functions that can fail. Use functions like
    /// `try_parse()` instead of `parse()` to retrieve this error.
    fn try_run(root: &mut Node, md: &MarkdownIt) -> Result<()> {
        // NOTE: Send+Sync bound is required for compatibility with anyhow!()
        Self::run(root, md);
        Ok(())
    }
}

macro_rules! rule_builder {
    ($var: ident) => {
        /// Adjust positioning of a newly added rule in the chain.
        pub struct RuleBuilder<'a, T> {
            item: &'a mut crate::common::ruler::RuleItem<crate::common::TypeKey, T>
        }

        impl<'a, T> RuleBuilder<'a, T> {
            pub(crate) fn new(item: &'a mut crate::common::ruler::RuleItem<crate::common::TypeKey, T>) -> Self {
                Self { item }
            }

            pub fn before<U: $var>(self) -> Self {
                self.item.before(crate::common::TypeKey::of::<U>());
                self
            }

            pub fn after<U: $var>(self) -> Self {
                self.item.after(crate::common::TypeKey::of::<U>());
                self
            }

            pub fn before_all(self) -> Self {
                self.item.before_all();
                self
            }

            pub fn after_all(self) -> Self {
                self.item.after_all();
                self
            }

            pub fn alias<U: $var>(self) -> Self {
                self.item.alias(crate::common::TypeKey::of::<U>());
                self
            }

            pub fn require<U: $var>(self) -> Self {
                self.item.require(crate::common::TypeKey::of::<U>());
                self
            }
        }
    };
}

rule_builder!(CoreRule);

pub(crate) use rule_builder;

#[derive(Debug)]
pub struct CoreRuleError {
    pub name: TypeKey,
}

impl Display for CoreRuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("core rule `{}` returned an error", self.name.short_name()))
    }
}
