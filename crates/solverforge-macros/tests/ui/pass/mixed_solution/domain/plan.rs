use solverforge::prelude::*;

use super::{Route, Visit, Worker};

#[planning_solution]
pub struct MixedPlan {
    #[problem_fact_collection]
    pub workers: Vec<Worker>,

    #[planning_entity_collection]
    pub routes: Vec<Route>,

    #[problem_fact_collection]
    pub visits: Vec<Visit>,

    #[planning_score]
    pub score: Option<HardSoftScore>,
}
