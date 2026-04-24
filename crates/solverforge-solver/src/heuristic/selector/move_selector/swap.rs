/// A swap move selector that generates `SwapMove` instances.
pub struct SwapMoveSelector<S, V, LES, RES> {
    left_entity_selector: LES,
    right_entity_selector: RES,
    getter: fn(&S, usize, usize) -> Option<V>,
    setter: fn(&mut S, usize, usize, Option<V>),
    descriptor_index: usize,
    variable_index: usize,
    variable_name: &'static str,
    _phantom: PhantomData<(fn() -> S, fn() -> V)>,
}

impl<S, V, LES: Debug, RES: Debug> Debug for SwapMoveSelector<S, V, LES, RES> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SwapMoveSelector")
            .field("left_entity_selector", &self.left_entity_selector)
            .field("right_entity_selector", &self.right_entity_selector)
            .field("descriptor_index", &self.descriptor_index)
            .field("variable_index", &self.variable_index)
            .field("variable_name", &self.variable_name)
            .finish()
    }
}

impl<S: PlanningSolution, V, LES, RES> SwapMoveSelector<S, V, LES, RES> {
    pub fn new(
        left_entity_selector: LES,
        right_entity_selector: RES,
        getter: fn(&S, usize, usize) -> Option<V>,
        setter: fn(&mut S, usize, usize, Option<V>),
        descriptor_index: usize,
        variable_index: usize,
        variable_name: &'static str,
    ) -> Self {
        Self {
            left_entity_selector,
            right_entity_selector,
            getter,
            setter,
            descriptor_index,
            variable_index,
            variable_name,
            _phantom: PhantomData,
        }
    }
}

impl<S: PlanningSolution, V>
    SwapMoveSelector<S, V, FromSolutionEntitySelector, FromSolutionEntitySelector>
{
    pub fn simple(
        getter: fn(&S, usize, usize) -> Option<V>,
        setter: fn(&mut S, usize, usize, Option<V>),
        descriptor_index: usize,
        variable_index: usize,
        variable_name: &'static str,
    ) -> Self {
        Self {
            left_entity_selector: FromSolutionEntitySelector::new(descriptor_index),
            right_entity_selector: FromSolutionEntitySelector::new(descriptor_index),
            getter,
            setter,
            descriptor_index,
            variable_index,
            variable_name,
            _phantom: PhantomData,
        }
    }
}

impl<S, V, LES, RES> MoveSelector<S, SwapMove<S, V>> for SwapMoveSelector<S, V, LES, RES>
where
    S: PlanningSolution,
    V: Clone + PartialEq + Send + Sync + Debug + 'static,
    LES: EntitySelector<S>,
    RES: EntitySelector<S>,
{
    type Cursor<'a>
        = ArenaMoveCursor<S, SwapMove<S, V>>
    where
        Self: 'a;

    fn open_cursor<'a, D: Director<S>>(&'a self, score_director: &D) -> Self::Cursor<'a> {
        let getter = self.getter;
        let setter = self.setter;
        let variable_index = self.variable_index;
        let variable_name = self.variable_name;
        let descriptor_index = self.descriptor_index;
        let right_entities: Vec<_> = self.right_entity_selector.iter(score_director).collect();
        let mut moves = Vec::new();
        for left_entity_ref in self.left_entity_selector.iter(score_director) {
            for right_entity_ref in &right_entities {
                if left_entity_ref.entity_index < right_entity_ref.entity_index {
                    moves.push(SwapMove::new(
                        left_entity_ref.entity_index,
                        right_entity_ref.entity_index,
                        getter,
                        setter,
                        variable_index,
                        variable_name,
                        descriptor_index,
                    ));
                }
            }
        }
        ArenaMoveCursor::from_moves(moves)
    }

    fn size<D: Director<S>>(&self, score_director: &D) -> usize {
        let left_count = self.left_entity_selector.iter(score_director).count();
        let right_count = self.right_entity_selector.iter(score_director).count();
        left_count.saturating_mul(right_count.saturating_sub(1)) / 2
    }
}
