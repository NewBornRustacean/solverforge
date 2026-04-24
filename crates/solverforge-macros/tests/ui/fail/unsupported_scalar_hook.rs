use solverforge::prelude::*;

#[planning_entity]
pub struct Task {
    #[planning_id]
    pub id: usize,

    #[planning_variable(
        value_range = "workers",
        nearby_value_distance_meter = "worker_distance"
    )]
    pub worker: Option<i64>,
}

fn main() {}
