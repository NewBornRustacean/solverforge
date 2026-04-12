use std::any::Any;
use std::fmt::{self, Debug};
use std::marker::PhantomData;

use solverforge_core::domain::{PlanningSolution, SolutionDescriptor};
use solverforge_scoring::Director;

use crate::heuristic::r#move::Move;

use super::bindings::VariableBinding;

#[derive(Clone)]
pub struct DescriptorChangeMove<S> {
    binding: VariableBinding,
    entity_index: usize,
    to_value: Option<usize>,
    solution_descriptor: SolutionDescriptor,
    _phantom: PhantomData<fn() -> S>,
}

impl<S> Debug for DescriptorChangeMove<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DescriptorChangeMove")
            .field("descriptor_index", &self.binding.descriptor_index)
            .field("entity_index", &self.entity_index)
            .field("variable_name", &self.binding.variable_name)
            .field("to_value", &self.to_value)
            .finish()
    }
}

impl<S: 'static> DescriptorChangeMove<S> {
    pub(crate) fn new(
        binding: VariableBinding,
        entity_index: usize,
        to_value: Option<usize>,
        solution_descriptor: SolutionDescriptor,
    ) -> Self {
        Self {
            binding,
            entity_index,
            to_value,
            solution_descriptor,
            _phantom: PhantomData,
        }
    }

    fn current_value(&self, solution: &S) -> Option<usize> {
        let entity = self
            .solution_descriptor
            .get_entity(
                solution as &dyn Any,
                self.binding.descriptor_index,
                self.entity_index,
            )
            .expect("entity lookup failed for descriptor change move");
        (self.binding.getter)(entity)
    }
}

impl<S> Move<S> for DescriptorChangeMove<S>
where
    S: PlanningSolution + 'static,
{
    fn is_doable<D: Director<S>>(&self, score_director: &D) -> bool {
        self.current_value(score_director.working_solution()) != self.to_value
    }

    fn do_move<D: Director<S>>(&self, score_director: &mut D) {
        let old_value = self.current_value(score_director.working_solution());
        score_director.before_variable_changed(self.binding.descriptor_index, self.entity_index);
        let entity = self
            .solution_descriptor
            .get_entity_mut(
                score_director.working_solution_mut() as &mut dyn Any,
                self.binding.descriptor_index,
                self.entity_index,
            )
            .expect("entity lookup failed for descriptor change move");
        (self.binding.setter)(entity, self.to_value);
        score_director.after_variable_changed(self.binding.descriptor_index, self.entity_index);

        let descriptor = self.solution_descriptor.clone();
        let binding = self.binding.clone();
        let entity_index = self.entity_index;
        score_director.register_undo(Box::new(move |solution: &mut S| {
            let entity = descriptor
                .get_entity_mut(
                    solution as &mut dyn Any,
                    binding.descriptor_index,
                    entity_index,
                )
                .expect("entity lookup failed for descriptor change undo");
            (binding.setter)(entity, old_value);
        }));
    }

    fn descriptor_index(&self) -> usize {
        self.binding.descriptor_index
    }

    fn entity_indices(&self) -> &[usize] {
        std::slice::from_ref(&self.entity_index)
    }

    fn variable_name(&self) -> &str {
        self.binding.variable_name
    }
}

#[derive(Clone)]
pub struct DescriptorSwapMove<S> {
    binding: VariableBinding,
    left_entity_index: usize,
    right_entity_index: usize,
    indices: [usize; 2],
    solution_descriptor: SolutionDescriptor,
    _phantom: PhantomData<fn() -> S>,
}

impl<S> Debug for DescriptorSwapMove<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DescriptorSwapMove")
            .field("descriptor_index", &self.binding.descriptor_index)
            .field("left_entity_index", &self.left_entity_index)
            .field("right_entity_index", &self.right_entity_index)
            .field("variable_name", &self.binding.variable_name)
            .finish()
    }
}

impl<S: 'static> DescriptorSwapMove<S> {
    pub(crate) fn new(
        binding: VariableBinding,
        left_entity_index: usize,
        right_entity_index: usize,
        solution_descriptor: SolutionDescriptor,
    ) -> Self {
        Self {
            binding,
            left_entity_index,
            right_entity_index,
            indices: [left_entity_index, right_entity_index],
            solution_descriptor,
            _phantom: PhantomData,
        }
    }

    fn current_value(&self, solution: &S, entity_index: usize) -> Option<usize> {
        let entity = self
            .solution_descriptor
            .get_entity(
                solution as &dyn Any,
                self.binding.descriptor_index,
                entity_index,
            )
            .expect("entity lookup failed for descriptor swap move");
        (self.binding.getter)(entity)
    }
}

impl<S> Move<S> for DescriptorSwapMove<S>
where
    S: PlanningSolution + 'static,
{
    fn is_doable<D: Director<S>>(&self, score_director: &D) -> bool {
        self.left_entity_index != self.right_entity_index
            && self.current_value(score_director.working_solution(), self.left_entity_index)
                != self.current_value(score_director.working_solution(), self.right_entity_index)
    }

    fn do_move<D: Director<S>>(&self, score_director: &mut D) {
        let left_value =
            self.current_value(score_director.working_solution(), self.left_entity_index);
        let right_value =
            self.current_value(score_director.working_solution(), self.right_entity_index);

        score_director
            .before_variable_changed(self.binding.descriptor_index, self.left_entity_index);
        score_director
            .before_variable_changed(self.binding.descriptor_index, self.right_entity_index);

        let left_entity = self
            .solution_descriptor
            .get_entity_mut(
                score_director.working_solution_mut() as &mut dyn Any,
                self.binding.descriptor_index,
                self.left_entity_index,
            )
            .expect("entity lookup failed for descriptor swap move");
        (self.binding.setter)(left_entity, right_value);

        let right_entity = self
            .solution_descriptor
            .get_entity_mut(
                score_director.working_solution_mut() as &mut dyn Any,
                self.binding.descriptor_index,
                self.right_entity_index,
            )
            .expect("entity lookup failed for descriptor swap move");
        (self.binding.setter)(right_entity, left_value);

        score_director
            .after_variable_changed(self.binding.descriptor_index, self.left_entity_index);
        score_director
            .after_variable_changed(self.binding.descriptor_index, self.right_entity_index);

        let descriptor = self.solution_descriptor.clone();
        let binding = self.binding.clone();
        let left_entity_index = self.left_entity_index;
        let right_entity_index = self.right_entity_index;
        score_director.register_undo(Box::new(move |solution: &mut S| {
            let left_entity = descriptor
                .get_entity_mut(
                    solution as &mut dyn Any,
                    binding.descriptor_index,
                    left_entity_index,
                )
                .expect("entity lookup failed for descriptor swap undo");
            (binding.setter)(left_entity, left_value);
            let right_entity = descriptor
                .get_entity_mut(
                    solution as &mut dyn Any,
                    binding.descriptor_index,
                    right_entity_index,
                )
                .expect("entity lookup failed for descriptor swap undo");
            (binding.setter)(right_entity, right_value);
        }));
    }

    fn descriptor_index(&self) -> usize {
        self.binding.descriptor_index
    }

    fn entity_indices(&self) -> &[usize] {
        &self.indices
    }

    fn variable_name(&self) -> &str {
        self.binding.variable_name
    }
}

#[derive(Clone)]
pub enum DescriptorEitherMove<S> {
    Change(DescriptorChangeMove<S>),
    Swap(DescriptorSwapMove<S>),
}

impl<S> Debug for DescriptorEitherMove<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Change(m) => m.fmt(f),
            Self::Swap(m) => m.fmt(f),
        }
    }
}

impl<S> Move<S> for DescriptorEitherMove<S>
where
    S: PlanningSolution + 'static,
{
    fn is_doable<D: Director<S>>(&self, score_director: &D) -> bool {
        match self {
            Self::Change(m) => m.is_doable(score_director),
            Self::Swap(m) => m.is_doable(score_director),
        }
    }

    fn do_move<D: Director<S>>(&self, score_director: &mut D) {
        match self {
            Self::Change(m) => m.do_move(score_director),
            Self::Swap(m) => m.do_move(score_director),
        }
    }

    fn descriptor_index(&self) -> usize {
        match self {
            Self::Change(m) => m.descriptor_index(),
            Self::Swap(m) => m.descriptor_index(),
        }
    }

    fn entity_indices(&self) -> &[usize] {
        match self {
            Self::Change(m) => m.entity_indices(),
            Self::Swap(m) => m.entity_indices(),
        }
    }

    fn variable_name(&self) -> &str {
        match self {
            Self::Change(m) => m.variable_name(),
            Self::Swap(m) => m.variable_name(),
        }
    }
}
