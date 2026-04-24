#[path = "mixed_solution/domain/mod.rs"]
mod domain;

use domain::*;

fn main() {
    let _ = MixedPlan {
        workers: Vec::new(),
        routes: Vec::new(),
        visits: Vec::new(),
        score: None,
    };
}
