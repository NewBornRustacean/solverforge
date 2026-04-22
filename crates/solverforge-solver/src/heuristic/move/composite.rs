/* CompositeMove - applies two moves in sequence by arena indices.

This move stores indices into two arenas. The moves themselves
live in their respective arenas - CompositeMove just references them.

# Zero-Erasure Design

No cloning, no boxing - just typed arena indices.
*/

use std::fmt::Debug;
use std::marker::PhantomData;

use smallvec::SmallVec;
use solverforge_core::domain::PlanningSolution;
use solverforge_scoring::Director;

use super::{Move, MoveArena, MoveTabuSignature};

/// A move that applies two moves in sequence via arena indices.
///
/// The moves live in separate arenas. CompositeMove stores the indices
/// and arena references needed to execute both moves.
///
/// # Type Parameters
/// * `S` - The planning solution type
/// * `M1` - The first move type
/// * `M2` - The second move type
pub struct CompositeMove<S, M1, M2>
where
    S: PlanningSolution,
    M1: Move<S>,
    M2: Move<S>,
{
    index_1: usize,
    index_2: usize,
    _phantom: PhantomData<(fn() -> S, fn() -> M1, fn() -> M2)>,
}

impl<S, M1, M2> CompositeMove<S, M1, M2>
where
    S: PlanningSolution,
    M1: Move<S>,
    M2: Move<S>,
{
    pub fn new(index_1: usize, index_2: usize) -> Self {
        Self {
            index_1,
            index_2,
            _phantom: PhantomData,
        }
    }

    pub fn index_1(&self) -> usize {
        self.index_1
    }

    pub fn index_2(&self) -> usize {
        self.index_2
    }

    pub fn is_doable_with_arenas<D: Director<S>>(
        &self,
        arena_1: &MoveArena<M1>,
        arena_2: &MoveArena<M2>,
        score_director: &D,
    ) -> bool {
        let m1 = arena_1.get(self.index_1);
        let m2 = arena_2.get(self.index_2);

        match (m1, m2) {
            (Some(m1), Some(m2)) => m1.is_doable(score_director) || m2.is_doable(score_director),
            _ => false,
        }
    }

    /// Executes both moves using the arenas.
    pub fn do_move_with_arenas<D: Director<S>>(
        &self,
        arena_1: &MoveArena<M1>,
        arena_2: &MoveArena<M2>,
        score_director: &mut D,
    ) {
        if let Some(m1) = arena_1.get(self.index_1) {
            m1.do_move(score_director);
        }
        if let Some(m2) = arena_2.get(self.index_2) {
            m2.do_move(score_director);
        }
    }
}

impl<S, M1, M2> Clone for CompositeMove<S, M1, M2>
where
    S: PlanningSolution,
    M1: Move<S>,
    M2: Move<S>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<S, M1, M2> Copy for CompositeMove<S, M1, M2>
where
    S: PlanningSolution,
    M1: Move<S>,
    M2: Move<S>,
{
}

impl<S, M1, M2> Debug for CompositeMove<S, M1, M2>
where
    S: PlanningSolution,
    M1: Move<S>,
    M2: Move<S>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompositeMove")
            .field("index_1", &self.index_1)
            .field("index_2", &self.index_2)
            .finish()
    }
}

/// A cached sequential composite that executes two moves in order.
///
/// The underlying moves stay in selector-owned arenas. This move stores raw
/// pointers into those arenas plus the precomputed metadata needed by the
/// canonical local-search path.
pub struct SequentialCompositeMove<S, M> {
    first_index: usize,
    second_index: usize,
    first_arena_addr: usize,
    second_arena_addr: usize,
    first_doable: bool,
    second_doable: bool,
    descriptor_index: usize,
    entity_indices: SmallVec<[usize; 8]>,
    variable_name: &'static str,
    tabu_signature: MoveTabuSignature,
    _phantom: PhantomData<(fn() -> S, fn() -> M)>,
}

// SAFETY: the move only stores immutable arena pointers plus cached metadata.
// The pointed-to arenas live in the selector for the duration of a step, and
// candidate evaluation never mutates them.
unsafe impl<S, M> Send for SequentialCompositeMove<S, M> {}

// SAFETY: see the `Send` impl above; the raw pointers are only dereferenced
// immutably during move execution and metadata access.
unsafe impl<S, M> Sync for SequentialCompositeMove<S, M> {}

impl<S, M> SequentialCompositeMove<S, M> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        first_index: usize,
        second_index: usize,
        first_arena_addr: usize,
        second_arena_addr: usize,
        first_doable: bool,
        second_doable: bool,
        descriptor_index: usize,
        entity_indices: SmallVec<[usize; 8]>,
        variable_name: &'static str,
        tabu_signature: MoveTabuSignature,
    ) -> Self {
        Self {
            first_index,
            second_index,
            first_arena_addr,
            second_arena_addr,
            first_doable,
            second_doable,
            descriptor_index,
            entity_indices,
            variable_name,
            tabu_signature,
            _phantom: PhantomData,
        }
    }

    pub fn first_index(&self) -> usize {
        self.first_index
    }

    pub fn second_index(&self) -> usize {
        self.second_index
    }

    fn first_move(&self) -> Option<&M> {
        let arena = self.first_arena_addr as *const MoveArena<M>;
        unsafe { arena.as_ref() }.and_then(|arena| arena.get(self.first_index))
    }

    fn second_move(&self) -> Option<&M> {
        let arena = self.second_arena_addr as *const MoveArena<M>;
        unsafe { arena.as_ref() }.and_then(|arena| arena.get(self.second_index))
    }
}

impl<S, M> Clone for SequentialCompositeMove<S, M>
where
    S: PlanningSolution,
    M: Move<S>,
{
    fn clone(&self) -> Self {
        Self {
            first_index: self.first_index,
            second_index: self.second_index,
            first_arena_addr: self.first_arena_addr,
            second_arena_addr: self.second_arena_addr,
            first_doable: self.first_doable,
            second_doable: self.second_doable,
            descriptor_index: self.descriptor_index,
            entity_indices: self.entity_indices.clone(),
            variable_name: self.variable_name,
            tabu_signature: self.tabu_signature.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<S, M> Debug for SequentialCompositeMove<S, M>
where
    S: PlanningSolution,
    M: Move<S>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SequentialCompositeMove")
            .field("first_index", &self.first_index)
            .field("second_index", &self.second_index)
            .field("first_doable", &self.first_doable)
            .field("second_doable", &self.second_doable)
            .field("descriptor_index", &self.descriptor_index)
            .field("variable_name", &self.variable_name)
            .field("entity_indices", &self.entity_indices)
            .finish()
    }
}

impl<S, M> Move<S> for SequentialCompositeMove<S, M>
where
    S: PlanningSolution,
    M: Move<S>,
{
    fn is_doable<D: Director<S>>(&self, _score_director: &D) -> bool {
        self.first_doable || self.second_doable
    }

    fn do_move<D: Director<S>>(&self, score_director: &mut D) {
        if self.first_doable {
            if let Some(first) = self.first_move() {
                first.do_move(score_director);
            }
        }
        if self.second_doable {
            if let Some(second) = self.second_move() {
                second.do_move(score_director);
            }
        }
    }

    fn descriptor_index(&self) -> usize {
        self.descriptor_index
    }

    fn entity_indices(&self) -> &[usize] {
        &self.entity_indices
    }

    fn variable_name(&self) -> &str {
        self.variable_name
    }

    fn tabu_signature<D: Director<S>>(&self, _score_director: &D) -> MoveTabuSignature {
        self.tabu_signature.clone()
    }
}
