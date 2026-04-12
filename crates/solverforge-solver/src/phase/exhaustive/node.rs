/* Exhaustive search node representation.

Each node represents a partial solution state in the search tree.
*/

use std::marker::PhantomData;

use crate::heuristic::Move;
use solverforge_core::domain::PlanningSolution;

/* A node in the exhaustive search tree.

Each node represents a partial solution state, containing:
- The depth in the search tree (number of variables assigned)
- The score at this node
- An optimistic bound (best possible score from this node)
- The move sequence to reach this node from the root
*/
#[derive(Debug, Clone)]
pub struct ExhaustiveSearchNode<S: PlanningSolution> {
    // Depth in the search tree (0 = root, number of assignments made).
    depth: usize,

    // The score at this node after applying all moves.
    score: S::Score,

    // Optimistic bound: best possible score achievable from this node.
    // Used for pruning branches that cannot improve on the best solution.
    optimistic_bound: Option<S::Score>,

    // Index of the entity being assigned at this node.
    entity_index: Option<usize>,

    // Index of the value assigned at this node.
    value_index: Option<usize>,

    // Parent node index in the node list (None for root).
    parent_index: Option<usize>,

    // Whether this node has been expanded.
    expanded: bool,
}

impl<S: PlanningSolution> ExhaustiveSearchNode<S> {
    pub fn root(score: S::Score) -> Self {
        Self {
            depth: 0,
            score,
            optimistic_bound: None,
            entity_index: None,
            value_index: None,
            parent_index: None,
            expanded: false,
        }
    }

    pub fn child(
        parent_index: usize,
        depth: usize,
        score: S::Score,
        entity_index: usize,
        value_index: usize,
    ) -> Self {
        Self {
            depth,
            score,
            optimistic_bound: None,
            entity_index: Some(entity_index),
            value_index: Some(value_index),
            parent_index: Some(parent_index),
            expanded: false,
        }
    }

    #[inline]
    pub fn depth(&self) -> usize {
        self.depth
    }

    #[inline]
    pub fn score(&self) -> &S::Score {
        &self.score
    }

    pub fn set_score(&mut self, score: S::Score) {
        self.score = score;
    }

    #[inline]
    pub fn optimistic_bound(&self) -> Option<&S::Score> {
        self.optimistic_bound.as_ref()
    }

    pub fn set_optimistic_bound(&mut self, bound: S::Score) {
        self.optimistic_bound = Some(bound);
    }

    #[inline]
    pub fn entity_index(&self) -> Option<usize> {
        self.entity_index
    }

    #[inline]
    pub fn value_index(&self) -> Option<usize> {
        self.value_index
    }

    #[inline]
    pub fn parent_index(&self) -> Option<usize> {
        self.parent_index
    }

    // Returns whether this node has been expanded.
    #[inline]
    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    /// Marks this node as expanded.
    pub fn mark_expanded(&mut self) {
        self.expanded = true;
    }

    pub fn is_leaf(&self, total_entities: usize) -> bool {
        self.depth >= total_entities
    }

    pub fn can_prune(&self, best_score: &S::Score) -> bool {
        match &self.optimistic_bound {
            Some(bound) => bound <= best_score,
            None => false,
        }
    }
}

/* Tracks the move sequence to reconstruct a solution path.

# Type Parameters
* `S` - The planning solution type
* `M` - The move type
*/
#[derive(Debug, Clone)]
pub struct MoveSequence<S, M>
where
    S: PlanningSolution,
    M: Move<S>,
{
    // The sequence of moves from root to current node.
    moves: Vec<M>,
    _phantom: PhantomData<fn() -> S>,
}

impl<S, M> MoveSequence<S, M>
where
    S: PlanningSolution,
    M: Move<S>,
{
    pub fn new() -> Self {
        Self {
            moves: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            moves: Vec::with_capacity(capacity),
            _phantom: PhantomData,
        }
    }

    pub fn push(&mut self, m: M) {
        self.moves.push(m);
    }

    /// Removes and returns the last move.
    pub fn pop(&mut self) -> Option<M> {
        self.moves.pop()
    }

    pub fn len(&self) -> usize {
        self.moves.len()
    }

    pub fn is_empty(&self) -> bool {
        self.moves.is_empty()
    }

    /// Returns an iterator over the moves.
    pub fn iter(&self) -> impl Iterator<Item = &M> {
        self.moves.iter()
    }

    /// Clears all moves from the sequence.
    pub fn clear(&mut self) {
        self.moves.clear();
    }
}

impl<S, M> Default for MoveSequence<S, M>
where
    S: PlanningSolution,
    M: Move<S>,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "node_tests.rs"]
mod tests;
