#[path = "list_alias_owner/domain/mod.rs"]
mod domain;

use domain::*;

fn main() {
    let mut plan = Plan {
        routes: Vec::new(),
        visits: Vec::new(),
        score: None,
    };

    let _ = Plan::list_len_static(&plan, 0);
    let _ = Plan::element_count(&plan);
    let _ = Plan::routes_list_len_static(&plan, 0);
    let _ = Plan::routes_element_count(&plan);
    let _ = Plan::routes_list_variable_descriptor_index();
    Plan::assign_element(&mut plan, 0, 0);
    Plan::routes_assign_element(&mut plan, 0, 0);
}
