use solverforge::prelude::*;

#[problem_fact]
pub struct RouteTask {
    #[planning_id]
    pub id: usize,
    pub route: Option<usize>,
}
