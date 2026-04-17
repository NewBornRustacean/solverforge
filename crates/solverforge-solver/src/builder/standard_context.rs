use std::fmt;
use std::marker::PhantomData;

pub enum StandardValueSource<S> {
    Empty,
    CountableRange {
        from: usize,
        to: usize,
    },
    SolutionCount {
        count_fn: fn(&S) -> usize,
    },
    EntitySlice {
        values_for_entity: for<'a> fn(&'a S, usize) -> &'a [usize],
    },
}

impl<S> Clone for StandardValueSource<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for StandardValueSource<S> {}

impl<S> fmt::Debug for StandardValueSource<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "StandardValueSource::Empty"),
            Self::CountableRange { from, to } => {
                write!(f, "StandardValueSource::CountableRange({from}..{to})")
            }
            Self::SolutionCount { .. } => write!(f, "StandardValueSource::SolutionCount(..)"),
            Self::EntitySlice { .. } => write!(f, "StandardValueSource::EntitySlice(..)"),
        }
    }
}

pub struct StandardVariableContext<S> {
    pub descriptor_index: usize,
    pub entity_type_name: &'static str,
    pub entity_count: fn(&S) -> usize,
    pub variable_name: &'static str,
    pub getter: fn(&S, usize) -> Option<usize>,
    pub setter: fn(&mut S, usize, Option<usize>),
    pub value_source: StandardValueSource<S>,
    pub allows_unassigned: bool,
}

impl<S> Clone for StandardVariableContext<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for StandardVariableContext<S> {}

impl<S> StandardVariableContext<S> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        descriptor_index: usize,
        entity_type_name: &'static str,
        entity_count: fn(&S) -> usize,
        variable_name: &'static str,
        getter: fn(&S, usize) -> Option<usize>,
        setter: fn(&mut S, usize, Option<usize>),
        value_source: StandardValueSource<S>,
        allows_unassigned: bool,
    ) -> Self {
        Self {
            descriptor_index,
            entity_type_name,
            entity_count,
            variable_name,
            getter,
            setter,
            value_source,
            allows_unassigned,
        }
    }

    pub fn matches_target(&self, entity_class: Option<&str>, variable_name: Option<&str>) -> bool {
        entity_class.is_none_or(|name| name == self.entity_type_name)
            && variable_name.is_none_or(|name| name == self.variable_name)
    }
}

impl<S> fmt::Debug for StandardVariableContext<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StandardVariableContext")
            .field("descriptor_index", &self.descriptor_index)
            .field("entity_type_name", &self.entity_type_name)
            .field("variable_name", &self.variable_name)
            .field("value_source", &self.value_source)
            .field("allows_unassigned", &self.allows_unassigned)
            .finish()
    }
}

pub struct StandardContext<S> {
    variables: Vec<StandardVariableContext<S>>,
    _phantom: PhantomData<fn() -> S>,
}

impl<S> StandardContext<S> {
    pub fn new(variables: Vec<StandardVariableContext<S>>) -> Self {
        Self {
            variables,
            _phantom: PhantomData,
        }
    }

    pub fn variables(&self) -> &[StandardVariableContext<S>] {
        &self.variables
    }

    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }
}

impl<S> fmt::Debug for StandardContext<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StandardContext")
            .field("variables", &self.variables)
            .finish()
    }
}
