/* Solver phases for different solving strategies

Phases are the main building blocks of solving:
- ConstructionHeuristicPhase: Builds an initial solution
- LocalSearchPhase: Improves an existing solution
- ExhaustiveSearchPhase: Explores entire solution space
- PartitionedSearchPhase: Parallel solving via partitioning
- VndPhase: Variable Neighborhood Descent
*/

pub mod construction;
pub mod exhaustive;
pub mod localsearch;
pub mod partitioned;
pub mod sequence;
pub mod stock_vnd;
mod traits;
mod tuple_impl;
pub mod vnd;

pub use sequence::PhaseSequence;
pub use stock_vnd::StockVndPhase;
pub use traits::Phase;
