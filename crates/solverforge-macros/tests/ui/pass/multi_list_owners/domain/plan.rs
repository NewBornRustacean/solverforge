use solverforge::prelude::*;

use super::{Route, RouteTask, Shift, ShiftTask};

#[planning_solution]
pub struct Plan {
    #[planning_entity_collection]
    pub routes: Vec<Route>,

    #[planning_entity_collection]
    pub shifts: Vec<Shift>,

    #[problem_fact_collection]
    pub route_tasks: Vec<RouteTask>,

    #[problem_fact_collection]
    pub shift_tasks: Vec<ShiftTask>,

    #[planning_score]
    pub score: Option<HardSoftScore>,
}
