pub use either::{ScalarChangeMoveSelector, ScalarSwapMoveSelector};

impl<S, M> Move<S> for &M
where
    S: PlanningSolution,
    M: Move<S> + ?Sized,
{
    fn is_doable<D: Director<S>>(&self, score_director: &D) -> bool {
        (**self).is_doable(score_director)
    }

    fn do_move<D: Director<S>>(&self, score_director: &mut D) {
        (**self).do_move(score_director)
    }

    fn descriptor_index(&self) -> usize {
        (**self).descriptor_index()
    }

    fn entity_indices(&self) -> &[usize] {
        (**self).entity_indices()
    }

    fn variable_name(&self) -> &str {
        (**self).variable_name()
    }

    fn tabu_signature<D: Director<S>>(
        &self,
        score_director: &D,
    ) -> crate::heuristic::r#move::MoveTabuSignature {
        (**self).tabu_signature(score_director)
    }
}

pub enum MoveCandidateRef<'a, S, M>
where
    S: PlanningSolution,
    M: Move<S>,
{
    Borrowed(&'a M),
    Sequential(SequentialCompositeMoveRef<'a, S, M>),
}

impl<S, M> Debug for MoveCandidateRef<'_, S, M>
where
    S: PlanningSolution,
    M: Move<S>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Borrowed(_) => write!(f, "MoveCandidateRef::Borrowed(..)"),
            Self::Sequential(mov) => write!(f, "MoveCandidateRef::Sequential({mov:?})"),
        }
    }
}

impl<S, M> Move<S> for MoveCandidateRef<'_, S, M>
where
    S: PlanningSolution,
    M: Move<S>,
{
    fn is_doable<D: Director<S>>(&self, score_director: &D) -> bool {
        match self {
            Self::Borrowed(mov) => mov.is_doable(score_director),
            Self::Sequential(mov) => mov.is_doable(score_director),
        }
    }

    fn do_move<D: Director<S>>(&self, score_director: &mut D) {
        match self {
            Self::Borrowed(mov) => mov.do_move(score_director),
            Self::Sequential(mov) => mov.do_move(score_director),
        }
    }

    fn descriptor_index(&self) -> usize {
        match self {
            Self::Borrowed(mov) => mov.descriptor_index(),
            Self::Sequential(mov) => mov.descriptor_index(),
        }
    }

    fn entity_indices(&self) -> &[usize] {
        match self {
            Self::Borrowed(mov) => mov.entity_indices(),
            Self::Sequential(mov) => mov.entity_indices(),
        }
    }

    fn variable_name(&self) -> &str {
        match self {
            Self::Borrowed(mov) => mov.variable_name(),
            Self::Sequential(mov) => mov.variable_name(),
        }
    }

    fn tabu_signature<D: Director<S>>(
        &self,
        score_director: &D,
    ) -> crate::heuristic::r#move::MoveTabuSignature {
        match self {
            Self::Borrowed(mov) => mov.tabu_signature(score_director),
            Self::Sequential(mov) => mov.tabu_signature(score_director),
        }
    }
}

pub trait MoveCursor<S: PlanningSolution, M: Move<S>> {
    fn next_candidate(&mut self) -> Option<(usize, MoveCandidateRef<'_, S, M>)>;

    fn candidate(&self, index: usize) -> Option<MoveCandidateRef<'_, S, M>>;

    fn take_candidate(&mut self, index: usize) -> M;
}

pub struct ArenaMoveCursor<S, M>
where
    S: PlanningSolution,
    M: Move<S>,
{
    moves: Vec<Option<M>>,
    next_index: usize,
    _phantom: PhantomData<fn() -> S>,
}

impl<S, M> ArenaMoveCursor<S, M>
where
    S: PlanningSolution,
    M: Move<S>,
{
    pub fn new() -> Self {
        Self {
            moves: Vec::new(),
            next_index: 0,
            _phantom: PhantomData,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            moves: Vec::with_capacity(capacity),
            next_index: 0,
            _phantom: PhantomData,
        }
    }

    pub fn from_moves<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = M>,
    {
        let mut cursor = Self::new();
        cursor.extend(iter);
        cursor
    }

    pub fn push(&mut self, mov: M) {
        self.moves.push(Some(mov));
    }

    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = M>,
    {
        self.moves.extend(iter.into_iter().map(Some));
    }
}

impl<S, M> Default for ArenaMoveCursor<S, M>
where
    S: PlanningSolution,
    M: Move<S>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S, M> Debug for ArenaMoveCursor<S, M>
where
    S: PlanningSolution,
    M: Move<S> + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArenaMoveCursor")
            .field("move_count", &self.moves.len())
            .field("next_index", &self.next_index)
            .finish()
    }
}

impl<S, M> MoveCursor<S, M> for ArenaMoveCursor<S, M>
where
    S: PlanningSolution,
    M: Move<S>,
{
    fn next_candidate(&mut self) -> Option<(usize, MoveCandidateRef<'_, S, M>)> {
        while self.next_index < self.moves.len() {
            let index = self.next_index;
            self.next_index += 1;
            if let Some(mov) = self.moves[index].as_ref() {
                return Some((index, MoveCandidateRef::Borrowed(mov)));
            }
        }
        None
    }

    fn candidate(&self, index: usize) -> Option<MoveCandidateRef<'_, S, M>> {
        self.moves
            .get(index)
            .and_then(|mov| mov.as_ref())
            .map(MoveCandidateRef::Borrowed)
    }

    fn take_candidate(&mut self, index: usize) -> M {
        self.moves
            .get_mut(index)
            .and_then(Option::take)
            .expect("move cursor candidate index must remain valid")
    }
}

impl<S, M> Iterator for ArenaMoveCursor<S, M>
where
    S: PlanningSolution,
    M: Move<S>,
{
    type Item = M;

    fn next(&mut self) -> Option<Self::Item> {
        let index = {
            let (index, _) = self.next_candidate()?;
            index
        };
        Some(self.take_candidate(index))
    }
}

pub(crate) fn collect_cursor_indices<S, M, C>(cursor: &mut C) -> Vec<usize>
where
    S: PlanningSolution,
    M: Move<S>,
    C: MoveCursor<S, M>,
{
    let mut indices = Vec::new();
    while let Some((index, _)) = cursor.next_candidate() {
        indices.push(index);
    }
    indices
}

