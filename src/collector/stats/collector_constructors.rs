use crate::SubtrActorResult;

use super::{BuiltinModuleSelection, IdentityFrameTransform, StatsCollector, StatsSnapshotFrame};

impl Default for StatsCollector<StatsSnapshotFrame, IdentityFrameTransform> {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsCollector<StatsSnapshotFrame, IdentityFrameTransform> {
    pub fn new() -> Self {
        Self::with_selection_and_frame_transform(
            BuiltinModuleSelection::all(),
            IdentityFrameTransform,
        )
        .expect("builtin stats modules should resolve without conflicts")
    }

    pub fn only_modules<I>(modules: I) -> Self
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        Self::try_only_modules(modules).expect("builtin stats module names should be valid")
    }

    pub fn try_only_modules<I>(modules: I) -> SubtrActorResult<Self>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        Self::with_builtin_module_names(modules)
    }

    pub fn with_builtin_module_names<I, S>(module_names: I) -> SubtrActorResult<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self::with_selection_and_frame_transform(
            BuiltinModuleSelection::from_names(module_names)?,
            IdentityFrameTransform,
        )
    }
}
