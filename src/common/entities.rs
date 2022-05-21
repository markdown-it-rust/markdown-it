use entities;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref ENTITIES_HASH : HashMap<&'static str, &'static str> = {
        let mut mapping = HashMap::new();
        for e in &entities::ENTITIES {
            if e.entity.ends_with(';') {
                mapping.insert(e.entity, e.characters);
            }
        }
        mapping
    };
}
