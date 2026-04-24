type LeafSelector<S, V, DM, IDM> =
    VecUnionSelector<S, NeighborhoodMove<S, V>, NeighborhoodLeaf<S, V, DM, IDM>>;

pub enum NeighborhoodMove<S, V> {
    Scalar(ScalarMoveUnion<S, usize>),
    List(ListMoveUnion<S, V>),
    Composite(SequentialCompositeMove<S, NeighborhoodMove<S, V>>),
}

impl<S, V> Clone for NeighborhoodMove<S, V>
where
    S: PlanningSolution + 'static,
    V: Clone + PartialEq + Send + Sync + Debug + 'static,
{
    fn clone(&self) -> Self {
        match self {
            Self::Scalar(m) => Self::Scalar(m.clone()),
            Self::List(m) => Self::List(m.clone()),
            Self::Composite(m) => Self::Composite(m.clone()),
        }
    }
}

impl<S, V> Debug for NeighborhoodMove<S, V>
where
    S: PlanningSolution + 'static,
    V: Clone + PartialEq + Send + Sync + Debug + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scalar(m) => write!(f, "NeighborhoodMove::Scalar({m:?})"),
            Self::List(m) => write!(f, "NeighborhoodMove::List({m:?})"),
            Self::Composite(m) => write!(f, "NeighborhoodMove::Composite({m:?})"),
        }
    }
}

impl<S, V> Move<S> for NeighborhoodMove<S, V>
where
    S: PlanningSolution + 'static,
    V: Clone + PartialEq + Send + Sync + Debug + 'static,
{
    fn is_doable<D: solverforge_scoring::Director<S>>(&self, score_director: &D) -> bool {
        match self {
            Self::Scalar(m) => m.is_doable(score_director),
            Self::List(m) => m.is_doable(score_director),
            Self::Composite(m) => m.is_doable(score_director),
        }
    }

    fn do_move<D: solverforge_scoring::Director<S>>(&self, score_director: &mut D) {
        match self {
            Self::Scalar(m) => m.do_move(score_director),
            Self::List(m) => m.do_move(score_director),
            Self::Composite(m) => m.do_move(score_director),
        }
    }

    fn descriptor_index(&self) -> usize {
        match self {
            Self::Scalar(m) => m.descriptor_index(),
            Self::List(m) => m.descriptor_index(),
            Self::Composite(m) => m.descriptor_index(),
        }
    }

    fn entity_indices(&self) -> &[usize] {
        match self {
            Self::Scalar(m) => m.entity_indices(),
            Self::List(m) => m.entity_indices(),
            Self::Composite(m) => m.entity_indices(),
        }
    }

    fn variable_name(&self) -> &str {
        match self {
            Self::Scalar(m) => m.variable_name(),
            Self::List(m) => m.variable_name(),
            Self::Composite(m) => m.variable_name(),
        }
    }

    fn tabu_signature<D: solverforge_scoring::Director<S>>(
        &self,
        score_director: &D,
    ) -> MoveTabuSignature {
        match self {
            Self::Scalar(m) => m.tabu_signature(score_director),
            Self::List(m) => m.tabu_signature(score_director),
            Self::Composite(m) => m.tabu_signature(score_director),
        }
    }
}

pub enum NeighborhoodLeaf<S, V, DM, IDM>
where
    S: PlanningSolution + 'static,
    V: Clone + PartialEq + Send + Sync + Debug + 'static,
    DM: CrossEntityDistanceMeter<S> + Clone,
    IDM: CrossEntityDistanceMeter<S> + Clone + 'static,
{
    Scalar(ScalarLeafSelector<S>),
    List(ListLeafSelector<S, V, DM, IDM>),
}

impl<S, V, DM, IDM> Debug for NeighborhoodLeaf<S, V, DM, IDM>
where
    S: PlanningSolution + 'static,
    V: Clone + PartialEq + Send + Sync + Debug + 'static,
    DM: CrossEntityDistanceMeter<S> + Clone + Debug,
    IDM: CrossEntityDistanceMeter<S> + Clone + Debug + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scalar(selector) => write!(f, "NeighborhoodLeaf::Scalar({selector:?})"),
            Self::List(selector) => write!(f, "NeighborhoodLeaf::List({selector:?})"),
        }
    }
}

impl<S, V, DM, IDM> MoveSelector<S, NeighborhoodMove<S, V>> for NeighborhoodLeaf<S, V, DM, IDM>
where
    S: PlanningSolution + 'static,
    V: Clone + PartialEq + Send + Sync + Debug + 'static,
    DM: CrossEntityDistanceMeter<S> + Clone + Debug + 'static,
    IDM: CrossEntityDistanceMeter<S> + Clone + Debug + 'static,
{
    type Cursor<'a>
        = ArenaMoveCursor<S, NeighborhoodMove<S, V>>
    where
        Self: 'a;

    fn open_cursor<'a, D: solverforge_scoring::Director<S>>(
        &'a self,
        score_director: &D,
    ) -> Self::Cursor<'a> {
        match self {
            Self::Scalar(selector) => ArenaMoveCursor::from_moves(
                selector
                    .iter_moves(score_director)
                    .map(NeighborhoodMove::Scalar),
            ),
            Self::List(selector) => ArenaMoveCursor::from_moves(
                selector
                    .iter_moves(score_director)
                    .map(NeighborhoodMove::List),
            ),
        }
    }

    fn size<D: solverforge_scoring::Director<S>>(&self, score_director: &D) -> usize {
        match self {
            Self::Scalar(selector) => selector.size(score_director),
            Self::List(selector) => selector.size(score_director),
        }
    }

    fn append_moves<D: solverforge_scoring::Director<S>>(
        &self,
        score_director: &D,
        arena: &mut MoveArena<NeighborhoodMove<S, V>>,
    ) {
        arena.extend(self.open_cursor(score_director));
    }
}

pub enum Neighborhood<S, V, DM, IDM>
where
    S: PlanningSolution + 'static,
    V: Clone + PartialEq + Send + Sync + Debug + 'static,
    DM: CrossEntityDistanceMeter<S> + Clone,
    IDM: CrossEntityDistanceMeter<S> + Clone + 'static,
{
    Flat(LeafSelector<S, V, DM, IDM>),
    Limited {
        selector: LeafSelector<S, V, DM, IDM>,
        selected_count_limit: usize,
    },
    Cartesian(CartesianNeighborhoodSelector<S, V, DM, IDM>),
}

pub enum CartesianChildSelector<S, V, DM, IDM>
where
    S: PlanningSolution + 'static,
    V: Clone + PartialEq + Send + Sync + Debug + 'static,
    DM: CrossEntityDistanceMeter<S> + Clone,
    IDM: CrossEntityDistanceMeter<S> + Clone + 'static,
{
    Flat(LeafSelector<S, V, DM, IDM>),
    Limited {
        selector: LeafSelector<S, V, DM, IDM>,
        selected_count_limit: usize,
    },
}

type CartesianNeighborhoodSelector<S, V, DM, IDM> = CartesianProductSelector<
    S,
    NeighborhoodMove<S, V>,
    CartesianChildSelector<S, V, DM, IDM>,
    CartesianChildSelector<S, V, DM, IDM>,
>;

pub enum CartesianChildCursor<S, V>
where
    S: PlanningSolution + 'static,
    V: Clone + PartialEq + Send + Sync + Debug + 'static,
{
    Flat(ArenaMoveCursor<S, NeighborhoodMove<S, V>>),
    Limited(ArenaMoveCursor<S, NeighborhoodMove<S, V>>),
}

impl<S, V> MoveCursor<S, NeighborhoodMove<S, V>> for CartesianChildCursor<S, V>
where
    S: PlanningSolution + 'static,
    V: Clone + PartialEq + Send + Sync + Debug + 'static,
{
    fn next_candidate(
        &mut self,
    ) -> Option<(usize, MoveCandidateRef<'_, S, NeighborhoodMove<S, V>>)> {
        match self {
            Self::Flat(cursor) => cursor.next_candidate(),
            Self::Limited(cursor) => cursor.next_candidate(),
        }
    }

    fn candidate(&self, index: usize) -> Option<MoveCandidateRef<'_, S, NeighborhoodMove<S, V>>> {
        match self {
            Self::Flat(cursor) => cursor.candidate(index),
            Self::Limited(cursor) => cursor.candidate(index),
        }
    }

    fn take_candidate(&mut self, index: usize) -> NeighborhoodMove<S, V> {
        match self {
            Self::Flat(cursor) => cursor.take_candidate(index),
            Self::Limited(cursor) => cursor.take_candidate(index),
        }
    }
}

pub enum NeighborhoodCursor<S, V>
where
    S: PlanningSolution + 'static,
    V: Clone + PartialEq + Send + Sync + Debug + 'static,
{
    Flat(ArenaMoveCursor<S, NeighborhoodMove<S, V>>),
    Limited(ArenaMoveCursor<S, NeighborhoodMove<S, V>>),
    Cartesian(CartesianProductCursor<S, NeighborhoodMove<S, V>>),
}

impl<S, V> MoveCursor<S, NeighborhoodMove<S, V>> for NeighborhoodCursor<S, V>
where
    S: PlanningSolution + 'static,
    V: Clone + PartialEq + Send + Sync + Debug + 'static,
{
    fn next_candidate(
        &mut self,
    ) -> Option<(usize, MoveCandidateRef<'_, S, NeighborhoodMove<S, V>>)> {
        match self {
            Self::Flat(cursor) => cursor.next_candidate(),
            Self::Limited(cursor) => cursor.next_candidate(),
            Self::Cartesian(cursor) => cursor.next_candidate(),
        }
    }

    fn candidate(&self, index: usize) -> Option<MoveCandidateRef<'_, S, NeighborhoodMove<S, V>>> {
        match self {
            Self::Flat(cursor) => cursor.candidate(index),
            Self::Limited(cursor) => cursor.candidate(index),
            Self::Cartesian(cursor) => cursor.candidate(index),
        }
    }

    fn take_candidate(&mut self, index: usize) -> NeighborhoodMove<S, V> {
        match self {
            Self::Flat(cursor) => cursor.take_candidate(index),
            Self::Limited(cursor) => cursor.take_candidate(index),
            Self::Cartesian(cursor) => cursor.take_candidate(index),
        }
    }
}

impl<S, V, DM, IDM> Debug for Neighborhood<S, V, DM, IDM>
where
    S: PlanningSolution + 'static,
    V: Clone + PartialEq + Send + Sync + Debug + 'static,
    DM: CrossEntityDistanceMeter<S> + Clone + Debug + 'static,
    IDM: CrossEntityDistanceMeter<S> + Clone + Debug + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Flat(selector) => write!(f, "Neighborhood::Flat({selector:?})"),
            Self::Limited {
                selector,
                selected_count_limit,
            } => f
                .debug_struct("Neighborhood::Limited")
                .field("selector", selector)
                .field("selected_count_limit", selected_count_limit)
                .finish(),
            Self::Cartesian(selector) => write!(f, "Neighborhood::Cartesian({selector:?})"),
        }
    }
}

impl<S, V, DM, IDM> MoveSelector<S, NeighborhoodMove<S, V>> for Neighborhood<S, V, DM, IDM>
where
    S: PlanningSolution + 'static,
    V: Clone + PartialEq + Send + Sync + Debug + 'static,
    DM: CrossEntityDistanceMeter<S> + Clone + Debug + 'static,
    IDM: CrossEntityDistanceMeter<S> + Clone + Debug + 'static,
{
    type Cursor<'a>
        = NeighborhoodCursor<S, V>
    where
        Self: 'a;

    fn open_cursor<'a, D: solverforge_scoring::Director<S>>(
        &'a self,
        score_director: &D,
    ) -> Self::Cursor<'a> {
        match self {
            Self::Flat(selector) => NeighborhoodCursor::Flat(ArenaMoveCursor::from_moves(
                selector.iter_moves(score_director),
            )),
            Self::Limited {
                selector,
                selected_count_limit,
            } => NeighborhoodCursor::Limited(ArenaMoveCursor::from_moves(
                selector
                    .iter_moves(score_director)
                    .take(*selected_count_limit),
            )),
            Self::Cartesian(selector) => {
                NeighborhoodCursor::Cartesian(selector.open_cursor(score_director))
            }
        }
    }

    fn size<D: solverforge_scoring::Director<S>>(&self, score_director: &D) -> usize {
        match self {
            Self::Flat(selector) => selector.size(score_director),
            Self::Limited {
                selector,
                selected_count_limit,
            } => selector.size(score_director).min(*selected_count_limit),
            Self::Cartesian(selector) => selector.size(score_director),
        }
    }
}

impl<S, V, DM, IDM> Debug for CartesianChildSelector<S, V, DM, IDM>
where
    S: PlanningSolution + 'static,
    V: Clone + PartialEq + Send + Sync + Debug + 'static,
    DM: CrossEntityDistanceMeter<S> + Clone + Debug,
    IDM: CrossEntityDistanceMeter<S> + Clone + Debug + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Flat(selector) => write!(f, "CartesianChildSelector::Flat({selector:?})"),
            Self::Limited {
                selector,
                selected_count_limit,
            } => f
                .debug_struct("CartesianChildSelector::Limited")
                .field("selector", selector)
                .field("selected_count_limit", selected_count_limit)
                .finish(),
        }
    }
}

impl<S, V, DM, IDM> MoveSelector<S, NeighborhoodMove<S, V>>
    for CartesianChildSelector<S, V, DM, IDM>
where
    S: PlanningSolution + 'static,
    V: Clone + PartialEq + Send + Sync + Debug + 'static,
    DM: CrossEntityDistanceMeter<S> + Clone + Debug + 'static,
    IDM: CrossEntityDistanceMeter<S> + Clone + Debug + 'static,
{
    type Cursor<'a>
        = CartesianChildCursor<S, V>
    where
        Self: 'a;

    fn open_cursor<'a, D: solverforge_scoring::Director<S>>(
        &'a self,
        score_director: &D,
    ) -> Self::Cursor<'a> {
        match self {
            Self::Flat(selector) => CartesianChildCursor::Flat(ArenaMoveCursor::from_moves(
                selector.iter_moves(score_director),
            )),
            Self::Limited {
                selector,
                selected_count_limit,
            } => CartesianChildCursor::Limited(ArenaMoveCursor::from_moves(
                selector
                    .iter_moves(score_director)
                    .take(*selected_count_limit),
            )),
        }
    }

    fn size<D: solverforge_scoring::Director<S>>(&self, score_director: &D) -> usize {
        match self {
            Self::Flat(selector) => selector.size(score_director),
            Self::Limited {
                selector,
                selected_count_limit,
            } => selector.size(score_director).min(*selected_count_limit),
        }
    }
}

