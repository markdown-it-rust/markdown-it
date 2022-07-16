use crate::{MarkdownIt, Node};

pub trait CoreRule : 'static {
    fn run(root: &mut Node, md: &MarkdownIt);
}

macro_rules! rule_builder {
    ($var: ident) => {
        pub struct RuleBuilder<'a, T> {
            item: &'a mut crate::common::ruler::RuleItem<crate::common::typekey::TypeKey, T>
        }

        impl<'a, T> RuleBuilder<'a, T> {
            pub(crate) fn new(item: &'a mut crate::common::ruler::RuleItem<crate::common::typekey::TypeKey, T>) -> Self {
                Self { item }
            }

            pub fn before<U: $var>(self) -> Self {
                self.item.before(crate::common::typekey::TypeKey::of::<U>());
                self
            }

            pub fn after<U: $var>(self) -> Self {
                self.item.after(crate::common::typekey::TypeKey::of::<U>());
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
                self.item.alias(crate::common::typekey::TypeKey::of::<U>());
                self
            }

            pub fn require<U: $var>(self) -> Self {
                self.item.require(crate::common::typekey::TypeKey::of::<U>());
                self
            }
        }
    };
}

rule_builder!(CoreRule);

pub(crate) use rule_builder;
