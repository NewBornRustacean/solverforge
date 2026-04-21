use crate::planning_entity::expand_derive;
use syn::parse_quote;

#[test]
fn golden_entity_expansion_includes_descriptor_and_planning_id() {
    let input = parse_quote! {
        struct Task {
            #[planning_id]
            id: String,
            #[planning_variable(allows_unassigned = true, value_range = "workers")]
            worker_idx: Option<usize>,
            #[planning_list_variable(element_collection = "all_tasks")]
            chain: Vec<usize>,
        }
    };

    let expanded = expand_derive(input)
        .expect("entity expansion should succeed")
        .to_string();

    assert!(expanded.contains("impl :: solverforge :: __internal :: PlanningEntity for Task"));
    assert!(expanded.contains("impl :: solverforge :: __internal :: PlanningId for Task"));
    assert!(expanded.contains("with_allows_unassigned (true)"));
    assert!(expanded.contains("with_value_range (\"workers\")"));
    assert!(expanded.contains("with_id_field (stringify ! (id))"));
    assert!(expanded.contains("pub fn entity_descriptor"));
    assert!(expanded.contains("pub const __SOLVERFORGE_LIST_VARIABLE_COUNT : usize = 1"));
    assert!(expanded
        .contains("const __SOLVERFORGE_LIST_ELEMENT_COLLECTION : & 'static str = \"all_tasks\""));
    assert!(expanded.contains("pub (crate) fn __solverforge_get_worker_idx_typed"));
    assert!(expanded.contains("pub (crate) fn __solverforge_set_worker_idx_typed"));
    assert!(expanded.contains("const HAS_LIST_VARIABLE : bool = true"));
    assert!(expanded.contains("LIST_ELEMENT_SOURCE"));
    assert!(expanded.contains("fn __solverforge_list_metadata < Solution >"));
    assert!(expanded.contains("pub trait TaskUnassignedFilter"));
    assert!(expanded.contains("fn unassigned (self)"));
}

#[test]
fn entity_descriptor_preserves_mixed_variable_declaration_order() {
    let input = parse_quote! {
        struct Route {
            #[planning_id]
            id: String,
            #[planning_list_variable(element_collection = "all_tasks")]
            chain: Vec<usize>,
            #[planning_variable(allows_unassigned = true, value_range = "workers")]
            worker_idx: Option<usize>,
            #[planning_variable(allows_unassigned = true, value_range = "workers")]
            backup_idx: Option<usize>,
        }
    };

    let expanded = expand_derive(input)
        .expect("entity expansion should succeed")
        .to_string();

    let chain_pos = expanded
        .find("VariableDescriptor :: list (\"chain\")")
        .expect("list variable descriptor should exist");
    let worker_pos = expanded
        .find("VariableDescriptor :: genuine (\"worker_idx\")")
        .expect("worker variable descriptor should exist");
    let backup_pos = expanded
        .find("VariableDescriptor :: genuine (\"backup_idx\")")
        .expect("backup variable descriptor should exist");

    assert!(chain_pos < worker_pos);
    assert!(worker_pos < backup_pos);
}
