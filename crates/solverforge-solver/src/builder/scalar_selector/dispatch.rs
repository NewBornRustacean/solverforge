pub enum ScalarLeafSelector<S> {
    Change(ScalarChangeSelector<S>),
    Swap(ScalarSwapSelector<S>),
    NearbyChange(NearbyChangeLeafSelector<S>),
    NearbySwap(NearbySwapLeafSelector<S>),
    PillarChange(PillarChangeLeafSelector<S>),
    PillarSwap(PillarSwapLeafSelector<S>),
    RuinRecreate(RuinRecreateLeafSelector<S>),
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(clippy::large_enum_variant)] // Inline storage keeps selector assembly zero-erasure.
pub enum ScalarSelectorNode<S> {
    Leaf(ScalarLeafSelector<S>),
    Cartesian(ScalarCartesianSelector<S>),
}

pub enum ScalarSelectorCursor<S>
where
    S: PlanningSolution + 'static,
{
    Leaf(ArenaMoveCursor<S, ScalarMoveUnion<S, usize>>),
    Cartesian(CartesianProductCursor<S, ScalarMoveUnion<S, usize>>),
}

impl<S> MoveCursor<S, ScalarMoveUnion<S, usize>> for ScalarSelectorCursor<S>
where
    S: PlanningSolution + 'static,
{
    fn next_candidate(
        &mut self,
    ) -> Option<(usize, MoveCandidateRef<'_, S, ScalarMoveUnion<S, usize>>)> {
        match self {
            Self::Leaf(cursor) => cursor.next_candidate(),
            Self::Cartesian(cursor) => cursor.next_candidate(),
        }
    }

    fn candidate(
        &self,
        index: usize,
    ) -> Option<MoveCandidateRef<'_, S, ScalarMoveUnion<S, usize>>> {
        match self {
            Self::Leaf(cursor) => cursor.candidate(index),
            Self::Cartesian(cursor) => cursor.candidate(index),
        }
    }

    fn take_candidate(&mut self, index: usize) -> ScalarMoveUnion<S, usize> {
        match self {
            Self::Leaf(cursor) => cursor.take_candidate(index),
            Self::Cartesian(cursor) => cursor.take_candidate(index),
        }
    }
}

impl<S> Debug for ScalarSelectorNode<S>
where
    S: PlanningSolution + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Leaf(selector) => selector.fmt(f),
            Self::Cartesian(selector) => selector.fmt(f),
        }
    }
}

impl<S> MoveSelector<S, ScalarMoveUnion<S, usize>> for ScalarSelectorNode<S>
where
    S: PlanningSolution + 'static,
    S::Score: Score,
{
    type Cursor<'a>
        = ScalarSelectorCursor<S>
    where
        Self: 'a;

    fn open_cursor<'a, D: Director<S>>(&'a self, score_director: &D) -> Self::Cursor<'a> {
        match self {
            Self::Leaf(selector) => {
                ScalarSelectorCursor::Leaf(selector.open_cursor(score_director))
            }
            Self::Cartesian(selector) => {
                ScalarSelectorCursor::Cartesian(selector.open_cursor(score_director))
            }
        }
    }

    fn size<D: Director<S>>(&self, score_director: &D) -> usize {
        match self {
            Self::Leaf(selector) => selector.size(score_director),
            Self::Cartesian(selector) => selector.size(score_director),
        }
    }

    fn append_moves<D: Director<S>>(
        &self,
        score_director: &D,
        arena: &mut MoveArena<ScalarMoveUnion<S, usize>>,
    ) {
        match self {
            Self::Leaf(selector) => selector.append_moves(score_director, arena),
            Self::Cartesian(selector) => selector.append_moves(score_director, arena),
        }
    }
}

impl<S> Debug for ScalarLeafSelector<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Change(selector) => selector.fmt(f),
            Self::Swap(selector) => selector.fmt(f),
            Self::NearbyChange(selector) => selector.fmt(f),
            Self::NearbySwap(selector) => selector.fmt(f),
            Self::PillarChange(selector) => selector.fmt(f),
            Self::PillarSwap(selector) => selector.fmt(f),
            Self::RuinRecreate(selector) => selector.fmt(f),
        }
    }
}

impl<S> MoveSelector<S, ScalarMoveUnion<S, usize>> for ScalarLeafSelector<S>
where
    S: PlanningSolution + 'static,
    S::Score: Score,
{
    type Cursor<'a>
        = ArenaMoveCursor<S, ScalarMoveUnion<S, usize>>
    where
        Self: 'a;

    fn open_cursor<'a, D: Director<S>>(&'a self, score_director: &D) -> Self::Cursor<'a> {
        match self {
            Self::Change(selector) => ArenaMoveCursor::from_moves(
                selector
                    .iter_moves(score_director)
                    .map(ScalarMoveUnion::Change),
            ),
            Self::Swap(selector) => selector.open_cursor(score_director),
            Self::NearbyChange(selector) => {
                ArenaMoveCursor::from_moves(selector.iter_moves(score_director))
            }
            Self::NearbySwap(selector) => {
                ArenaMoveCursor::from_moves(selector.iter_moves(score_director))
            }
            Self::PillarChange(selector) => {
                ArenaMoveCursor::from_moves(selector.iter_moves(score_director))
            }
            Self::PillarSwap(selector) => {
                ArenaMoveCursor::from_moves(selector.iter_moves(score_director))
            }
            Self::RuinRecreate(selector) => {
                ArenaMoveCursor::from_moves(selector.iter_moves(score_director))
            }
        }
    }

    fn size<D: Director<S>>(&self, score_director: &D) -> usize {
        match self {
            Self::Change(selector) => selector.size(score_director),
            Self::Swap(selector) => selector.size(score_director),
            Self::NearbyChange(selector) => selector.size(score_director),
            Self::NearbySwap(selector) => selector.size(score_director),
            Self::PillarChange(selector) => selector.size(score_director),
            Self::PillarSwap(selector) => selector.size(score_director),
            Self::RuinRecreate(selector) => selector.size(score_director),
        }
    }

    fn append_moves<D: Director<S>>(
        &self,
        score_director: &D,
        arena: &mut MoveArena<ScalarMoveUnion<S, usize>>,
    ) {
        match self {
            Self::Change(selector) => arena.extend(
                selector
                    .open_cursor(score_director)
                    .map(ScalarMoveUnion::Change),
            ),
            Self::Swap(selector) => selector.append_moves(score_director, arena),
            Self::NearbyChange(selector) => selector.append_moves(score_director, arena),
            Self::NearbySwap(selector) => selector.append_moves(score_director, arena),
            Self::PillarChange(selector) => selector.append_moves(score_director, arena),
            Self::PillarSwap(selector) => selector.append_moves(score_director, arena),
            Self::RuinRecreate(selector) => selector.append_moves(score_director, arena),
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn wrap_scalar_composite<S>(
    mov: SequentialCompositeMove<S, ScalarMoveUnion<S, usize>>,
) -> ScalarMoveUnion<S, usize>
where
    S: PlanningSolution,
{
    ScalarMoveUnion::Composite(mov)
}

pub(super) fn build_scalar_flat_selector<S>(
    config: Option<&MoveSelectorConfig>,
    scalar_variables: &[ScalarVariableContext<S>],
    random_seed: Option<u64>,
) -> ScalarFlatSelector<S>
where
    S: PlanningSolution + 'static,
    S::Score: Score,
{
    let mut leaves = Vec::new();
    collect_scalar_leaf_selectors(config, scalar_variables, random_seed, &mut leaves);
    assert!(
        !leaves.is_empty(),
        "move selector configuration produced no scalar neighborhoods"
    );
    VecUnionSelector::new(leaves)
}

