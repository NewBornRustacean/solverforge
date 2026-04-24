use solverforge::prelude::*;

#[planning_entity]
pub struct Shift {
    #[planning_id]
    pub id: usize,

    #[planning_list_variable(element_collection = "shift_tasks")]
    pub tasks: Vec<usize>,
}
