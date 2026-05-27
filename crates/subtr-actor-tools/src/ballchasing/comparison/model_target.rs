use std::fmt;

use super::{domain::StatDomain, scope::StatScope, stat_key::StatKey};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::ballchasing::comparison) struct ComparisonTarget {
    pub(in crate::ballchasing::comparison) scope: StatScope,
    pub(in crate::ballchasing::comparison) domain: StatDomain,
    pub(in crate::ballchasing::comparison) key: StatKey,
}

impl fmt::Display for ComparisonTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.scope, self.domain, self.key)
    }
}
