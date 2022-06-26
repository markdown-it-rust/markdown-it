use std::collections::HashMap;
use std::any::TypeId;
use std::any::Any;
use std::fmt::Debug;

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
        val.last_mut().unwrap().get_or_insert::<T>()
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

pub struct EnvState(HashMap<TypeId, Box<dyn EnvStateTrait>>);

impl EnvState {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get<T: 'static + Default + Debug>(&self) -> Option<&T> {
        let typeid = TypeId::of::<T>();
        let result = self.0.get(&typeid)?;
        // SAFETY: Hash is indexed by TypeId, therefore we know that whatever we got has the same TypeId as T.
        // With hash being private, new hash value can only be inserted as T::default() -> T from this function.
        // New value can be assigned by user via &mut T, but it should also be T.
        Some(unsafe { result.downcast_unsafe::<T>() })
    }

    pub fn get_or_insert<T: 'static + Default + Debug>(&mut self) -> &mut T {
        let typeid = TypeId::of::<T>();
        let result = self.0.entry(typeid).or_insert_with(|| Box::new(EnvStateStruct(T::default())));
        // SAFETY: see get()
        unsafe { result.downcast_mut_unsafe::<T>() }
    }
}

impl Debug for EnvState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_set().entries(
            self.0.iter().map(|(_, v)| v)
        ).finish()
    }
}

pub trait EnvMember {
    type Scope : scope::EnvScope;
}

trait EnvStateTrait : Any + Debug {
    fn my_type_id(&self) -> TypeId;
}

impl dyn EnvStateTrait {
    unsafe fn downcast_unsafe<T: 'static>(&self) -> &T {
        debug_assert_eq!(TypeId::of::<T>(), self.my_type_id());
        &*(self as *const dyn EnvStateTrait as *const T)
    }

    unsafe fn downcast_mut_unsafe<T: 'static>(&mut self) -> &mut T {
        debug_assert_eq!(TypeId::of::<T>(), self.my_type_id());
        &mut *(self as *mut dyn EnvStateTrait as *mut T)
    }
}

struct EnvStateStruct<T>(T);

impl<T: 'static + Debug> EnvStateTrait for EnvStateStruct<T> {
    fn my_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

impl<T: Debug> Debug for EnvStateStruct<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
