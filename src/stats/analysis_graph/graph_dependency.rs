use std::any::{type_name, TypeId};

use super::AnalysisNodeDyn;

#[derive(Clone, Copy)]
pub struct AnalysisDependency {
    state_type_id: TypeId,
    state_type_name: &'static str,
    source: AnalysisDependencySource,
}

#[derive(Clone, Copy)]
enum AnalysisDependencySource {
    DefaultFactory(fn() -> Box<dyn AnalysisNodeDyn>),
    External,
}

impl AnalysisDependency {
    pub fn required<T: 'static>() -> Self {
        Self {
            state_type_id: TypeId::of::<T>(),
            state_type_name: type_name::<T>(),
            source: AnalysisDependencySource::External,
        }
    }

    pub fn with_default<T: 'static>(default_factory: fn() -> Box<dyn AnalysisNodeDyn>) -> Self {
        Self {
            state_type_id: TypeId::of::<T>(),
            state_type_name: type_name::<T>(),
            source: AnalysisDependencySource::DefaultFactory(default_factory),
        }
    }

    pub fn state_type_id(&self) -> TypeId {
        self.state_type_id
    }

    pub fn state_type_name(&self) -> &'static str {
        self.state_type_name
    }

    pub(super) fn default_factory(&self) -> fn() -> Box<dyn AnalysisNodeDyn> {
        match self.source {
            AnalysisDependencySource::DefaultFactory(default_factory) => default_factory,
            AnalysisDependencySource::External => panic!(
                "analysis dependency for {} has no default factory",
                self.state_type_name
            ),
        }
    }

    pub(super) fn is_external(&self) -> bool {
        matches!(self.source, AnalysisDependencySource::External)
    }
}
