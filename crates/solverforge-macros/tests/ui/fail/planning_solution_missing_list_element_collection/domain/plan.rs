use solverforge::prelude::*;

use super::Route;

#[planning_solution]
#[shadow_variable_updates(list_owner = "routes", inverse_field = "route")]
pub struct Plan {
    #[planning_entity_collection]
    pub routes: Vec<Route>,

    pub all_visits: Vec<usize>,

    #[planning_score]
    pub score: Option<HardSoftScore>,
}
