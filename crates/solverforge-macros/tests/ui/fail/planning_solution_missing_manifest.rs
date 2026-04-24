use solverforge::prelude::*;

#[planning_entity]
struct Task {
    #[planning_id]
    id: usize,
}

#[planning_solution]
struct Plan {
    #[planning_entity_collection]
    tasks: Vec<Task>,

    #[planning_score]
    score: Option<HardSoftScore>,
}

fn main() {}
