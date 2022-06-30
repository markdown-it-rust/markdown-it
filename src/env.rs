use std::fmt::Debug;

use crate::erasedset::ErasedSet;

type EnvState = ErasedSet;

#[derive(Debug)]
pub struct Env {
    block_state:      Vec<EnvState>,
    block_lvl_state:  Vec<EnvState>,
    inline_state:     Vec<EnvState>,
    inline_lvl_state: Vec<EnvState>,
}

pub mod scope {
    use super::Env;
    use super::EnvState;

    pub struct Block;
    pub struct BlockLvl;
    pub struct Inline;
    pub struct InlineLvl;

    pub trait EnvScope {
        fn get_scope(env: &Env) -> &Vec<EnvState>;
        fn get_scope_mut(env: &mut Env) -> &mut Vec<EnvState>;
    }

    impl EnvScope for Block {
        fn get_scope(env: &Env) -> &Vec<EnvState> {
            &env.block_state
        }
        fn get_scope_mut(env: &mut Env) -> &mut Vec<EnvState> {
            &mut env.block_state
        }
    }

    impl EnvScope for BlockLvl {
        fn get_scope(env: &Env) -> &Vec<EnvState> {
            &env.block_lvl_state
        }
        fn get_scope_mut(env: &mut Env) -> &mut Vec<EnvState> {
            &mut env.block_lvl_state
        }
    }

    impl EnvScope for Inline {
        fn get_scope(env: &Env) -> &Vec<EnvState> {
            &env.inline_state
        }
        fn get_scope_mut(env: &mut Env) -> &mut Vec<EnvState> {
            &mut env.inline_state
        }
    }

    impl EnvScope for InlineLvl {
        fn get_scope(env: &Env) -> &Vec<EnvState> {
            &env.inline_lvl_state
        }
        fn get_scope_mut(env: &mut Env) -> &mut Vec<EnvState> {
            &mut env.inline_lvl_state
        }
    }
}

impl Env {
    pub fn new() -> Self {
        Self {
            block_state:      Vec::new(),
            block_lvl_state:  Vec::new(),
            inline_state:     Vec::new(),
            inline_lvl_state: Vec::new(),
        }
    }

    pub fn get<T: 'static + EnvMember + Default + Debug>(&self) -> Option<&T> {
        use scope::EnvScope;
        let val = T::Scope::get_scope(self);
        val.last().unwrap().get::<T>()
    }

    pub fn get_or_insert<T: 'static + EnvMember + Default + Debug>(&mut self) -> &mut T {
        use scope::EnvScope;
        let val = T::Scope::get_scope_mut(self);
        val.last_mut().unwrap().get_or_insert_default::<T>()
    }

    pub fn state_push<S: scope::EnvScope>(&mut self) {
        let val = S::get_scope_mut(self);
        val.push(EnvState::new());
    }

    pub fn state_pop<S: scope::EnvScope>(&mut self) {
        let val = S::get_scope_mut(self);
        val.pop();
    }
}

pub trait EnvMember {
    type Scope : scope::EnvScope;
}
