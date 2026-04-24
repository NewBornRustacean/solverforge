#[path = "multi_list_owners/domain/mod.rs"]
mod domain;

use domain::*;

fn main() {
    let mut plan = Plan {
        routes: Vec::new(),
        shifts: Vec::new(),
        route_tasks: Vec::new(),
        shift_tasks: Vec::new(),
        score: None,
    };

    let _ = Plan::routes_list_len_static(&plan, 0);
    let _ = Plan::shifts_list_len_static(&plan, 0);
    let _ = Plan::routes_element_count(&plan);
    let _ = Plan::shifts_element_count(&plan);
    let _ = Plan::routes_list_variable_descriptor_index();
    let _ = Plan::shifts_list_variable_descriptor_index();
    Plan::routes_assign_element(&mut plan, 0, 0);
    Plan::shifts_assign_element(&mut plan, 0, 0);
}
