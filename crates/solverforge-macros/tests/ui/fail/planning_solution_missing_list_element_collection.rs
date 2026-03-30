use solverforge::prelude::*;

#[planning_entity]
struct Route {
    #[planning_id]
    id: usize,

    #[planning_list_variable(element_collection = "all_visits")]
    visits: Vec<usize>,
}

#[planning_solution]
#[shadow_variable_updates(list_owner = "routes", inverse_field = "route")]
struct Plan {
    #[planning_entity_collection]
    routes: Vec<Route>,

    all_visits: Vec<usize>,

    #[planning_score]
    score: Option<HardSoftScore>,
}

fn main() {}
