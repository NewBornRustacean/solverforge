#[derive(Clone)]
pub struct DescriptorChangeMoveSelector<S> {
    binding: VariableBinding,
    solution_descriptor: SolutionDescriptor,
    allows_unassigned: bool,
    _phantom: PhantomData<fn() -> S>,
}

impl<S> Debug for DescriptorChangeMoveSelector<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DescriptorChangeMoveSelector")
            .field("binding", &self.binding)
            .finish()
    }
}

impl<S> DescriptorChangeMoveSelector<S> {
    fn new(binding: VariableBinding, solution_descriptor: SolutionDescriptor) -> Self {
        let allows_unassigned = binding.allows_unassigned;
        Self {
            binding,
            solution_descriptor,
            allows_unassigned,
            _phantom: PhantomData,
        }
    }
}

impl<S> MoveSelector<S, DescriptorScalarMoveUnion<S>> for DescriptorChangeMoveSelector<S>
where
    S: PlanningSolution + 'static,
    S::Score: Score,
{
    type Cursor<'a>
        = ArenaMoveCursor<S, DescriptorScalarMoveUnion<S>>
    where
        Self: 'a;

    fn open_cursor<'a, D: Director<S>>(&'a self, score_director: &D) -> Self::Cursor<'a> {
        let count = score_director
            .entity_count(self.binding.descriptor_index)
            .unwrap_or(0);
        let descriptor = self.solution_descriptor.clone();
        let binding = self.binding.clone();
        let allows_unassigned = self.allows_unassigned;
        let solution = score_director.working_solution() as &dyn Any;
        let moves: Vec<_> = (0..count)
            .flat_map(move |entity_index| {
                let entity = descriptor
                    .get_entity(solution, binding.descriptor_index, entity_index)
                    .expect("entity lookup failed for change selector");
                let current_value = (binding.getter)(entity);
                let unassign_move = (allows_unassigned && current_value.is_some()).then({
                    let binding = binding.clone();
                    let descriptor = descriptor.clone();
                    move || {
                        DescriptorScalarMoveUnion::Change(DescriptorChangeMove::new(
                            binding.clone(),
                            entity_index,
                            None,
                            descriptor.clone(),
                        ))
                    }
                });
                binding
                    .values_for_entity(&descriptor, solution, entity)
                    .into_iter()
                    .map({
                        let binding = binding.clone();
                        let descriptor = descriptor.clone();
                        move |value| {
                            DescriptorScalarMoveUnion::Change(DescriptorChangeMove::new(
                                binding.clone(),
                                entity_index,
                                Some(value),
                                descriptor.clone(),
                            ))
                        }
                    })
                    .chain(unassign_move)
            })
            .collect();
        ArenaMoveCursor::from_moves(moves)
    }

    fn size<D: Director<S>>(&self, score_director: &D) -> usize {
        let count = score_director
            .entity_count(self.binding.descriptor_index)
            .unwrap_or(0);
        let mut total = 0;
        for entity_index in 0..count {
            let entity = self
                .solution_descriptor
                .get_entity(
                    score_director.working_solution() as &dyn Any,
                    self.binding.descriptor_index,
                    entity_index,
                )
                .expect("entity lookup failed for change selector");
            total += self
                .binding
                .values_for_entity(
                    &self.solution_descriptor,
                    score_director.working_solution() as &dyn Any,
                    entity,
                )
                .len()
                + usize::from(self.allows_unassigned && (self.binding.getter)(entity).is_some());
        }
        total
    }
}

#[derive(Clone)]
pub struct DescriptorSwapMoveSelector<S> {
    binding: VariableBinding,
    solution_descriptor: SolutionDescriptor,
    _phantom: PhantomData<fn() -> S>,
}

impl<S> Debug for DescriptorSwapMoveSelector<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DescriptorSwapMoveSelector")
            .field("binding", &self.binding)
            .finish()
    }
}

impl<S> DescriptorSwapMoveSelector<S> {
    fn new(binding: VariableBinding, solution_descriptor: SolutionDescriptor) -> Self {
        Self {
            binding,
            solution_descriptor,
            _phantom: PhantomData,
        }
    }
}

impl<S> MoveSelector<S, DescriptorScalarMoveUnion<S>> for DescriptorSwapMoveSelector<S>
where
    S: PlanningSolution + 'static,
    S::Score: Score,
{
    type Cursor<'a>
        = ArenaMoveCursor<S, DescriptorScalarMoveUnion<S>>
    where
        Self: 'a;

    fn open_cursor<'a, D: Director<S>>(&'a self, score_director: &D) -> Self::Cursor<'a> {
        let count = score_director
            .entity_count(self.binding.descriptor_index)
            .unwrap_or(0);
        let binding = self.binding.clone();
        let descriptor = self.solution_descriptor.clone();
        let solution = score_director.working_solution() as &dyn Any;
        let legality_index = SwapLegalityIndex::new(
            &binding,
            &descriptor,
            solution,
            count,
            "entity lookup failed for swap selector",
        );

        let mut moves = Vec::new();
        for left_entity_index in 0..count {
            for right_entity_index in (left_entity_index + 1)..count {
                if let Some((left_value, right_value)) =
                    legality_index.values_for_swap(left_entity_index, right_entity_index)
                {
                    moves.push(DescriptorScalarMoveUnion::Swap(
                        DescriptorSwapMove::new_validated(
                            binding.clone(),
                            left_entity_index,
                            left_value,
                            right_entity_index,
                            right_value,
                            descriptor.clone(),
                        ),
                    ));
                }
            }
        }
        ArenaMoveCursor::from_moves(moves)
    }

    fn size<D: Director<S>>(&self, score_director: &D) -> usize {
        let count = score_director
            .entity_count(self.binding.descriptor_index)
            .unwrap_or(0);
        let solution = score_director.working_solution() as &dyn Any;
        let legality_index = SwapLegalityIndex::new(
            &self.binding,
            &self.solution_descriptor,
            solution,
            count,
            "entity lookup failed for swap selector",
        );
        legality_index.count_legal_pairs()
    }
}

#[derive(Clone)]
pub struct DescriptorNearbyChangeMoveSelector<S> {
    binding: VariableBinding,
    solution_descriptor: SolutionDescriptor,
    max_nearby: usize,
    _phantom: PhantomData<fn() -> S>,
}

impl<S> Debug for DescriptorNearbyChangeMoveSelector<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DescriptorNearbyChangeMoveSelector")
            .field("binding", &self.binding)
            .field("max_nearby", &self.max_nearby)
            .finish()
    }
}

impl<S> MoveSelector<S, DescriptorScalarMoveUnion<S>> for DescriptorNearbyChangeMoveSelector<S>
where
    S: PlanningSolution + 'static,
    S::Score: Score,
{
    type Cursor<'a>
        = ArenaMoveCursor<S, DescriptorScalarMoveUnion<S>>
    where
        Self: 'a;

    fn open_cursor<'a, D: Director<S>>(&'a self, score_director: &D) -> Self::Cursor<'a> {
        let distance_meter = self
            .binding
            .nearby_value_distance_meter
            .expect("nearby change requires a nearby value distance meter");
        let solution = score_director.working_solution() as &dyn Any;
        let count = score_director
            .entity_count(self.binding.descriptor_index)
            .unwrap_or(0);
        let binding = self.binding.clone();
        let descriptor = self.solution_descriptor.clone();
        let max_nearby = self.max_nearby;
        let moves: Vec<_> = (0..count)
            .flat_map(move |entity_index| {
                let entity = descriptor
                    .get_entity(solution, binding.descriptor_index, entity_index)
                    .expect("entity lookup failed for nearby change selector");
                let current_value = (binding.getter)(entity);
                let current_assigned = current_value.is_some();
                let mut candidates: Vec<(usize, f64, usize)> = binding
                    .values_for_entity(&descriptor, solution, entity)
                    .into_iter()
                    .enumerate()
                    .filter_map(|(order, value)| {
                        if current_value == Some(value) {
                            return None;
                        }
                        let distance = distance_meter(solution, entity_index, value);
                        distance.is_finite().then_some((value, distance, order))
                    })
                    .collect();
                truncate_nearby_candidates(&mut candidates, max_nearby);

                let candidate_moves = candidates.into_iter().map({
                    let binding = binding.clone();
                    let descriptor = descriptor.clone();
                    move |(value, _, _)| {
                        DescriptorScalarMoveUnion::Change(DescriptorChangeMove::new(
                            binding.clone(),
                            entity_index,
                            Some(value),
                            descriptor.clone(),
                        ))
                    }
                });
                let unassign = (binding.allows_unassigned && current_assigned).then({
                    let binding = binding.clone();
                    let descriptor = descriptor.clone();
                    move || {
                        DescriptorScalarMoveUnion::Change(DescriptorChangeMove::new(
                            binding.clone(),
                            entity_index,
                            None,
                            descriptor.clone(),
                        ))
                    }
                });
                candidate_moves.chain(unassign)
            })
            .collect();
        ArenaMoveCursor::from_moves(moves)
    }

    fn size<D: Director<S>>(&self, score_director: &D) -> usize {
        self.open_cursor(score_director).count()
    }
}

#[derive(Clone)]
pub struct DescriptorNearbySwapMoveSelector<S> {
    binding: VariableBinding,
    solution_descriptor: SolutionDescriptor,
    max_nearby: usize,
    _phantom: PhantomData<fn() -> S>,
}

impl<S> Debug for DescriptorNearbySwapMoveSelector<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DescriptorNearbySwapMoveSelector")
            .field("binding", &self.binding)
            .field("max_nearby", &self.max_nearby)
            .finish()
    }
}

impl<S> MoveSelector<S, DescriptorScalarMoveUnion<S>> for DescriptorNearbySwapMoveSelector<S>
where
    S: PlanningSolution + 'static,
    S::Score: Score,
{
    type Cursor<'a>
        = ArenaMoveCursor<S, DescriptorScalarMoveUnion<S>>
    where
        Self: 'a;

    fn open_cursor<'a, D: Director<S>>(&'a self, score_director: &D) -> Self::Cursor<'a> {
        let distance_meter = self
            .binding
            .nearby_entity_distance_meter
            .expect("nearby swap requires a nearby entity distance meter");
        let solution = score_director.working_solution() as &dyn Any;
        let count = score_director
            .entity_count(self.binding.descriptor_index)
            .unwrap_or(0);
        let binding = self.binding.clone();
        let descriptor = self.solution_descriptor.clone();
        let max_nearby = self.max_nearby;
        let legality_index = SwapLegalityIndex::new(
            &binding,
            &descriptor,
            solution,
            count,
            "entity lookup failed for nearby swap selector",
        );
        let mut moves = Vec::new();
        for left_entity_index in 0..count {
            let mut candidates: Vec<(usize, f64, usize)> = ((left_entity_index + 1)..count)
                .enumerate()
                .filter_map(|(order, right_entity_index)| {
                    if !legality_index.can_swap(left_entity_index, right_entity_index) {
                        return None;
                    }
                    let distance = distance_meter(solution, left_entity_index, right_entity_index);
                    distance
                        .is_finite()
                        .then_some((right_entity_index, distance, order))
                })
                .collect();
            truncate_nearby_candidates(&mut candidates, max_nearby);
            for (right_entity_index, _, _) in candidates {
                let Some((left_value, right_value)) =
                    legality_index.values_for_swap(left_entity_index, right_entity_index)
                else {
                    continue;
                };
                moves.push(DescriptorScalarMoveUnion::Swap(
                    DescriptorSwapMove::new_validated(
                        binding.clone(),
                        left_entity_index,
                        left_value,
                        right_entity_index,
                        right_value,
                        descriptor.clone(),
                    ),
                ));
            }
        }
        ArenaMoveCursor::from_moves(moves)
    }

    fn size<D: Director<S>>(&self, score_director: &D) -> usize {
        self.open_cursor(score_director).count()
    }
}

