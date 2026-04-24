#[derive(Clone, Copy)]
pub struct SwapLeafSelector<S> {
    ctx: ScalarVariableContext<S>,
}

impl<S> Debug for SwapLeafSelector<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SwapLeafSelector")
            .field("descriptor_index", &self.ctx.descriptor_index)
            .field("variable_index", &self.ctx.variable_index)
            .field("variable_name", &self.ctx.variable_name)
            .finish()
    }
}

impl<S> MoveSelector<S, ScalarMoveUnion<S, usize>> for SwapLeafSelector<S>
where
    S: PlanningSolution + 'static,
{
    type Cursor<'a>
        = ArenaMoveCursor<S, ScalarMoveUnion<S, usize>>
    where
        Self: 'a;

    fn open_cursor<'a, D: Director<S>>(&'a self, score_director: &D) -> Self::Cursor<'a> {
        let solution = score_director.working_solution();
        let entity_count = (self.ctx.entity_count)(solution);
        let current_values: Vec<_> = (0..entity_count)
            .map(|entity_index| self.ctx.current_value(solution, entity_index))
            .collect();
        let legal_values: Vec<_> = (0..entity_count)
            .map(|entity_index| self.ctx.values_for_entity(solution, entity_index))
            .collect();
        let mut moves = Vec::new();

        for left_entity_index in 0..entity_count {
            let left_value = current_values[left_entity_index];
            for right_entity_index in (left_entity_index + 1)..entity_count {
                let right_value = current_values[right_entity_index];
                if left_value == right_value {
                    continue;
                }
                if !scalar_swap_is_legal(self.ctx, &legal_values[left_entity_index], right_value)
                    || !scalar_swap_is_legal(
                        self.ctx,
                        &legal_values[right_entity_index],
                        left_value,
                    )
                {
                    continue;
                }
                moves.push(ScalarMoveUnion::Swap(
                    crate::heuristic::r#move::SwapMove::new(
                        left_entity_index,
                        right_entity_index,
                        self.ctx.getter,
                        self.ctx.setter,
                        self.ctx.variable_index,
                        self.ctx.variable_name,
                        self.ctx.descriptor_index,
                    ),
                ));
            }
        }

        ArenaMoveCursor::from_moves(moves)
    }

    fn size<D: Director<S>>(&self, score_director: &D) -> usize {
        self.open_cursor(score_director).count()
    }

    fn append_moves<D: Director<S>>(
        &self,
        score_director: &D,
        arena: &mut MoveArena<ScalarMoveUnion<S, usize>>,
    ) {
        arena.extend(self.open_cursor(score_director));
    }
}

#[derive(Clone, Copy)]
pub struct NearbyChangeLeafSelector<S> {
    ctx: ScalarVariableContext<S>,
    max_nearby: usize,
}

impl<S> Debug for NearbyChangeLeafSelector<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NearbyChangeLeafSelector")
            .field("descriptor_index", &self.ctx.descriptor_index)
            .field("variable_name", &self.ctx.variable_name)
            .field("max_nearby", &self.max_nearby)
            .finish()
    }
}

impl<S> MoveSelector<S, ScalarMoveUnion<S, usize>> for NearbyChangeLeafSelector<S>
where
    S: PlanningSolution + 'static,
{
    type Cursor<'a>
        = ArenaMoveCursor<S, ScalarMoveUnion<S, usize>>
    where
        Self: 'a;

    fn open_cursor<'a, D: Director<S>>(&'a self, score_director: &D) -> Self::Cursor<'a> {
        let solution = score_director.working_solution();
        let distance_meter = self
            .ctx
            .nearby_value_distance_meter
            .expect("nearby change requires a nearby value distance meter");
        let value_selector = ScalarValueSelector::from_context(self.ctx);
        let mut moves = Vec::new();

        for entity_index in 0..(self.ctx.entity_count)(solution) {
            let current_value = self.ctx.current_value(solution, entity_index);
            let current_assigned = current_value.is_some();
            let mut candidates: Vec<(usize, f64, usize)> = value_selector
                .iter(score_director, self.ctx.descriptor_index, entity_index)
                .enumerate()
                .filter_map(|(order, value)| {
                    if current_value == Some(value) {
                        return None;
                    }
                    let distance =
                        distance_meter(solution, entity_index, self.ctx.variable_index, value)?;
                    distance.is_finite().then_some((value, distance, order))
                })
                .collect();

            truncate_nearby_candidates(&mut candidates, self.max_nearby);

            moves.extend(candidates.into_iter().map(|(value, _, _)| {
                ScalarMoveUnion::Change(crate::heuristic::r#move::ChangeMove::new(
                    entity_index,
                    Some(value),
                    self.ctx.getter,
                    self.ctx.setter,
                    self.ctx.variable_index,
                    self.ctx.variable_name,
                    self.ctx.descriptor_index,
                ))
            }));

            if self.ctx.allows_unassigned && current_assigned {
                moves.push(ScalarMoveUnion::Change(
                    crate::heuristic::r#move::ChangeMove::new(
                        entity_index,
                        None,
                        self.ctx.getter,
                        self.ctx.setter,
                        self.ctx.variable_index,
                        self.ctx.variable_name,
                        self.ctx.descriptor_index,
                    ),
                ));
            }
        }

        ArenaMoveCursor::from_moves(moves)
    }

    fn size<D: Director<S>>(&self, score_director: &D) -> usize {
        self.open_cursor(score_director).count()
    }

    fn append_moves<D: Director<S>>(
        &self,
        score_director: &D,
        arena: &mut MoveArena<ScalarMoveUnion<S, usize>>,
    ) {
        arena.extend(self.open_cursor(score_director));
    }
}

#[derive(Clone, Copy)]
pub struct NearbySwapLeafSelector<S> {
    ctx: ScalarVariableContext<S>,
    max_nearby: usize,
}

impl<S> Debug for NearbySwapLeafSelector<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NearbySwapLeafSelector")
            .field("descriptor_index", &self.ctx.descriptor_index)
            .field("variable_name", &self.ctx.variable_name)
            .field("max_nearby", &self.max_nearby)
            .finish()
    }
}

impl<S> MoveSelector<S, ScalarMoveUnion<S, usize>> for NearbySwapLeafSelector<S>
where
    S: PlanningSolution + 'static,
{
    type Cursor<'a>
        = ArenaMoveCursor<S, ScalarMoveUnion<S, usize>>
    where
        Self: 'a;

    fn open_cursor<'a, D: Director<S>>(&'a self, score_director: &D) -> Self::Cursor<'a> {
        let solution = score_director.working_solution();
        let distance_meter = self
            .ctx
            .nearby_entity_distance_meter
            .expect("nearby swap requires a nearby entity distance meter");
        let entity_count = (self.ctx.entity_count)(solution);
        let current_values: Vec<_> = (0..entity_count)
            .map(|entity_index| self.ctx.current_value(solution, entity_index))
            .collect();
        let legal_values: Vec<_> = (0..entity_count)
            .map(|entity_index| self.ctx.values_for_entity(solution, entity_index))
            .collect();
        let mut moves = Vec::new();

        for left_entity_index in 0..entity_count {
            let left_value = current_values[left_entity_index];
            let mut candidates: Vec<(usize, f64, usize)> = ((left_entity_index + 1)..entity_count)
                .enumerate()
                .filter_map(|(order, right_entity_index)| {
                    if left_value == current_values[right_entity_index] {
                        return None;
                    }
                    if !scalar_swap_is_legal(
                        self.ctx,
                        &legal_values[left_entity_index],
                        current_values[right_entity_index],
                    ) || !scalar_swap_is_legal(
                        self.ctx,
                        &legal_values[right_entity_index],
                        left_value,
                    ) {
                        return None;
                    }
                    let distance = distance_meter(
                        solution,
                        left_entity_index,
                        right_entity_index,
                        self.ctx.variable_index,
                    )?;
                    distance
                        .is_finite()
                        .then_some((right_entity_index, distance, order))
                })
                .collect();

            truncate_nearby_candidates(&mut candidates, self.max_nearby);

            moves.extend(candidates.into_iter().map(|(right_entity_index, _, _)| {
                ScalarMoveUnion::Swap(crate::heuristic::r#move::SwapMove::new(
                    left_entity_index,
                    right_entity_index,
                    self.ctx.getter,
                    self.ctx.setter,
                    self.ctx.variable_index,
                    self.ctx.variable_name,
                    self.ctx.descriptor_index,
                ))
            }));
        }

        ArenaMoveCursor::from_moves(moves)
    }

    fn size<D: Director<S>>(&self, score_director: &D) -> usize {
        self.open_cursor(score_director).count()
    }

    fn append_moves<D: Director<S>>(
        &self,
        score_director: &D,
        arena: &mut MoveArena<ScalarMoveUnion<S, usize>>,
    ) {
        arena.extend(self.open_cursor(score_director));
    }
}

