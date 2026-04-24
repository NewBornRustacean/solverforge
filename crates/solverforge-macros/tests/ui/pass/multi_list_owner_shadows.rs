#[path = "multi_list_owner_shadows/domain/mod.rs"]
mod domain;

use domain::*;

fn main() {
    let _ = Plan {
        routes: Vec::new(),
        shifts: Vec::new(),
        route_tasks: Vec::new(),
        shift_tasks: Vec::new(),
        score: None,
    };
}
