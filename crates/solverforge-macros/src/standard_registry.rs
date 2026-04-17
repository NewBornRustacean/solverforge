use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Debug)]
pub(crate) struct StandardVariableMetadata {
    pub field_name: String,
    pub allows_unassigned: bool,
    pub value_range_provider: Option<String>,
    pub countable_range: Option<(i64, i64)>,
    pub provider_is_entity_field: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct StandardEntityMetadata {
    pub variables: Vec<StandardVariableMetadata>,
}

static STANDARD_ENTITY_REGISTRY: OnceLock<Mutex<HashMap<String, StandardEntityMetadata>>> =
    OnceLock::new();

fn registry() -> &'static Mutex<HashMap<String, StandardEntityMetadata>> {
    STANDARD_ENTITY_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn record_standard_entity_metadata(
    entity_name: &str,
    variables: Vec<StandardVariableMetadata>,
) {
    let mut registry = registry()
        .lock()
        .expect("solverforge standard metadata registry should be available");
    registry.insert(
        entity_name.to_string(),
        StandardEntityMetadata { variables },
    );
}

pub(crate) fn lookup_standard_entity_metadata(entity_name: &str) -> Option<StandardEntityMetadata> {
    let registry = registry()
        .lock()
        .expect("solverforge standard metadata registry should be available");
    registry.get(entity_name).cloned()
}
