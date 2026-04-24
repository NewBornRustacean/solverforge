use solverforge::prelude::*;

use super::{Route, Visit};

#[planning_solution]
pub struct Plan {
    #[planning_entity_collection]
    pub routes: Vec<Route>,

    #[planning_entity_collection]
    pub visits: Vec<Visit>,

    #[planning_score]
    pub score: Option<HardSoftScore>,
}
