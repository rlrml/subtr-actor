use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::ballchasing::comparison) enum TeamColor {
    Blue,
    Orange,
}

impl TeamColor {
    pub(in crate::ballchasing::comparison) fn team_key(self) -> &'static str {
        match self {
            Self::Blue => "blue",
            Self::Orange => "orange",
        }
    }
}

impl fmt::Display for TeamColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blue => write!(f, "blue"),
            Self::Orange => write!(f, "orange"),
        }
    }
}
