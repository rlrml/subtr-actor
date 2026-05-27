use std::fmt;

use super::team::TeamColor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::ballchasing::comparison) enum StatScope {
    Team(TeamColor),
    Player { team: TeamColor, name: String },
}

impl fmt::Display for StatScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Team(team) => write!(f, "team.{team}"),
            Self::Player { team, name } => write!(f, "player.{team}.{name}"),
        }
    }
}
