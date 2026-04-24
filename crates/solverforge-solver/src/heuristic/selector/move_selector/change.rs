/// A change move selector that generates `ChangeMove` instances.
pub struct ChangeMoveSelector<S, V, ES, VS> {
    entity_selector: ES,
    value_selector: VS,
    getter: fn(&S, usize, usize) -> Option<V>,
    setter: fn(&mut S, usize, usize, Option<V>),
    descriptor_index: usize,
    variable_index: usize,
    variable_name: &'static str,
    allows_unassigned: bool,
    _phantom: PhantomData<(fn() -> S, fn() -> V)>,
}

impl<S, V: Debug, ES: Debug, VS: Debug> Debug for ChangeMoveSelector<S, V, ES, VS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChangeMoveSelector")
            .field("entity_selector", &self.entity_selector)
            .field("value_selector", &self.value_selector)
            .field("descriptor_index", &self.descriptor_index)
            .field("variable_index", &self.variable_index)
            .field("variable_name", &self.variable_name)
            .field("allows_unassigned", &self.allows_unassigned)
            .finish()
    }
}

impl<S: PlanningSolution, V: Clone, ES, VS> ChangeMoveSelector<S, V, ES, VS> {
    pub fn new(
        entity_selector: ES,
        value_selector: VS,
        getter: fn(&S, usize, usize) -> Option<V>,
        setter: fn(&mut S, usize, usize, Option<V>),
        descriptor_index: usize,
        variable_index: usize,
        variable_name: &'static str,
    ) -> Self {
        Self {
            entity_selector,
            value_selector,
            getter,
            setter,
            descriptor_index,
            variable_index,
            variable_name,
            allows_unassigned: false,
            _phantom: PhantomData,
        }
    }

    pub fn with_allows_unassigned(mut self, allows_unassigned: bool) -> Self {
        self.allows_unassigned = allows_unassigned;
        self
    }
}

impl<S: PlanningSolution, V: Clone + Send + Sync + Debug + 'static>
    ChangeMoveSelector<S, V, FromSolutionEntitySelector, StaticValueSelector<S, V>>
{
    pub fn simple(
        getter: fn(&S, usize, usize) -> Option<V>,
        setter: fn(&mut S, usize, usize, Option<V>),
        descriptor_index: usize,
        variable_index: usize,
        variable_name: &'static str,
        values: Vec<V>,
    ) -> Self {
        Self {
            entity_selector: FromSolutionEntitySelector::new(descriptor_index),
            value_selector: StaticValueSelector::new(values),
            getter,
            setter,
            descriptor_index,
            variable_index,
            variable_name,
            allows_unassigned: false,
            _phantom: PhantomData,
        }
    }
}

impl<S, V, ES, VS> MoveSelector<S, ChangeMove<S, V>> for ChangeMoveSelector<S, V, ES, VS>
where
    S: PlanningSolution,
    V: Clone + PartialEq + Send + Sync + Debug + 'static,
    ES: EntitySelector<S>,
    VS: ValueSelector<S, V>,
{
    type Cursor<'a>
        = ArenaMoveCursor<S, ChangeMove<S, V>>
    where
        Self: 'a;

    fn open_cursor<'a, D: Director<S>>(&'a self, score_director: &D) -> Self::Cursor<'a> {
        let descriptor_index = self.descriptor_index;
        let variable_index = self.variable_index;
        let variable_name = self.variable_name;
        let getter = self.getter;
        let setter = self.setter;
        let allows_unassigned = self.allows_unassigned;
        let value_selector = &self.value_selector;
        let solution = score_director.working_solution();
        let entity_values: Vec<_> = self
            .entity_selector
            .iter(score_director)
            .map(|entity_ref| {
                let current_assigned =
                    getter(solution, entity_ref.entity_index, variable_index).is_some();
                let values = value_selector.iter(
                    score_director,
                    entity_ref.descriptor_index,
                    entity_ref.entity_index,
                );
                (entity_ref, values, current_assigned)
            })
            .collect();

        let iter =
            entity_values
                .into_iter()
                .flat_map(move |(entity_ref, values, current_assigned)| {
                    let to_none = (allows_unassigned && current_assigned).then(|| {
                        ChangeMove::new(
                            entity_ref.entity_index,
                            None,
                            getter,
                            setter,
                            variable_index,
                            variable_name,
                            descriptor_index,
                        )
                    });
                    values
                        .map(move |value| {
                            ChangeMove::new(
                                entity_ref.entity_index,
                                Some(value),
                                getter,
                                setter,
                                variable_index,
                                variable_name,
                                descriptor_index,
                            )
                        })
                        .chain(to_none)
                });
        ArenaMoveCursor::from_moves(iter)
    }

    fn size<D: Director<S>>(&self, score_director: &D) -> usize {
        self.entity_selector
            .iter(score_director)
            .map(|entity_ref| {
                self.value_selector.size(
                    score_director,
                    entity_ref.descriptor_index,
                    entity_ref.entity_index,
                ) + usize::from(
                    self.allows_unassigned
                        && (self.getter)(
                            score_director.working_solution(),
                            entity_ref.entity_index,
                            self.variable_index,
                        )
                        .is_some(),
                )
            })
            .sum()
    }
}

