#[path = "list_multi_module/domain/mod.rs"]
mod domain;

use domain::*;

fn main() {
    let _ = Plan {
        items: Vec::new(),
        containers: Vec::new(),
        score: None,
    };
}
