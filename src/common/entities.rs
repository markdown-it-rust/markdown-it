use entities;
use once_cell::sync::Lazy;
use std::collections::HashMap;

pub static ENTITIES_HASH : Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut mapping = HashMap::new();
    for e in &entities::ENTITIES {
        if e.entity.ends_with(';') {
            mapping.insert(e.entity, e.characters);
        }
    }
    mapping
});
