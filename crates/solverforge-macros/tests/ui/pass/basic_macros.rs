#[path = "basic_macros/domain/mod.rs"]
mod domain;

use domain::*;

fn main() {
    let _ = Plan {
        tasks: Vec::new(),
        workers: Vec::new(),
        score: None,
    };
}
