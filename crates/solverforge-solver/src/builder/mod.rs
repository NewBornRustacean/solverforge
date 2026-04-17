/* Builder module for constructing solver components from configuration.

Provides wiring between `SolverConfig` and the actual solver types.
All builders return concrete monomorphized enums — no `Box<dyn Trait>`.
*/

pub mod acceptor;
pub mod context;
pub mod forager;
pub mod list_selector;
pub mod standard_context;
pub mod standard_selector;

pub use acceptor::{AcceptorBuilder, AnyAcceptor};
pub use context::{IntraDistanceAdapter, ListContext};
pub use forager::{AnyForager, ForagerBuilder};
pub use list_selector::{ListLeafSelector, ListMoveSelectorBuilder};
pub use standard_context::{StandardContext, StandardValueSource, StandardVariableContext};
pub use standard_selector::{build_standard_move_selector, StandardLeafSelector, StandardSelector};
