use solverforge::prelude::*;

use super::{Task, Worker};

#[planning_solution]
pub struct Plan {
    #[planning_entity_collection]
    pub tasks: Vec<Task>,
    #[problem_fact_collection]
    pub workers: Vec<Worker>,
    #[planning_score]
    pub score: Option<HardSoftScore>,
}
