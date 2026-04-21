/* Construction heuristic phase

Builds an initial solution by assigning values to uninitialized
planning variables one at a time.
*/

mod config;
mod decision;
mod engine;
mod evaluation;
mod forager;
mod frontier;
mod phase;
mod placer;
mod slot;

pub use config::{ConstructionHeuristicConfig, ForagerType};
pub use forager::{
    BestFitForager, ConstructionChoice, ConstructionForager, FirstFeasibleForager, FirstFitForager,
    StrongestFitForager, WeakestFitForager,
};
pub(crate) use frontier::ConstructionFrontier;
pub use phase::ConstructionHeuristicPhase;
pub use placer::{EntityPlacer, Placement, QueuedEntityPlacer, SortedEntityPlacer};
pub(crate) use slot::{ConstructionListElementId, ConstructionSlotId};
pub(crate) use engine::solve_construction;
