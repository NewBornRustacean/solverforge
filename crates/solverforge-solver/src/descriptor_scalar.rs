mod bindings;
mod construction;
mod move_types;
mod selectors;

pub(crate) use bindings::scalar_work_remaining_with_frontier;
pub use bindings::{descriptor_has_bindings, scalar_target_matches, scalar_work_remaining};
pub use construction::{
    build_descriptor_construction, DescriptorConstruction, DescriptorEntityPlacer,
};
pub use move_types::{
    DescriptorChangeMove, DescriptorPillarChangeMove, DescriptorPillarSwapMove,
    DescriptorRuinRecreateMove, DescriptorScalarMoveUnion, DescriptorSwapMove,
};
pub use selectors::{
    build_descriptor_move_selector, DescriptorChangeMoveSelector, DescriptorFlatSelector,
    DescriptorLeafSelector, DescriptorSelector, DescriptorSelectorNode, DescriptorSwapMoveSelector,
};

#[cfg(test)]
mod tests;
