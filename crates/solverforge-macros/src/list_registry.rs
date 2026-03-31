use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Debug)]
pub(crate) struct ListEntityMetadata {
    pub element_collection_name: String,
}

static LIST_ENTITY_REGISTRY: OnceLock<Mutex<HashMap<String, ListEntityMetadata>>> = OnceLock::new();

fn registry() -> &'static Mutex<HashMap<String, ListEntityMetadata>> {
    LIST_ENTITY_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn record_list_entity_metadata(entity_name: &str, element_collection_name: String) {
    let mut registry = registry()
        .lock()
        .expect("solverforge list metadata registry should be available");
    registry.insert(
        entity_name.to_string(),
        ListEntityMetadata {
            element_collection_name,
        },
    );
}

pub(crate) fn lookup_list_entity_metadata(entity_name: &str) -> Option<ListEntityMetadata> {
    let registry = registry()
        .lock()
        .expect("solverforge list metadata registry should be available");
    registry.get(entity_name).cloned()
}
