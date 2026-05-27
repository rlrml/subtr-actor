use std::collections::BTreeMap;

use serde::Serialize;

use super::super::super::model::TeamColor;
use super::{
    ComparableBoostStats, ComparableCoreStats, ComparableDemoStats, ComparableMovementStats,
    ComparablePositioningStats,
};

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub(crate) struct ComparablePlayerStats {
    pub(crate) core: ComparableCoreStats,
    pub(crate) boost: ComparableBoostStats,
    pub(crate) movement: ComparableMovementStats,
    pub(crate) positioning: ComparablePositioningStats,
    pub(crate) demo: ComparableDemoStats,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub(crate) struct ComparableTeamStats {
    pub(crate) core: ComparableCoreStats,
    pub(crate) boost: ComparableBoostStats,
    pub(crate) movement: ComparableMovementStats,
    pub(crate) demo: ComparableDemoStats,
    pub(crate) players: BTreeMap<String, ComparablePlayerStats>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub(crate) struct ComparableReplayStats {
    pub(crate) blue: ComparableTeamStats,
    pub(crate) orange: ComparableTeamStats,
}

impl ComparableReplayStats {
    pub(in crate::ballchasing::comparison) fn team(
        &self,
        color: TeamColor,
    ) -> &ComparableTeamStats {
        match color {
            TeamColor::Blue => &self.blue,
            TeamColor::Orange => &self.orange,
        }
    }

    pub(in crate::ballchasing::comparison) fn team_mut(
        &mut self,
        color: TeamColor,
    ) -> &mut ComparableTeamStats {
        match color {
            TeamColor::Blue => &mut self.blue,
            TeamColor::Orange => &mut self.orange,
        }
    }
}
