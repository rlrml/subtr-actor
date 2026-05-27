use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::ballchasing::comparison) enum StatDomain {
    Core,
    Boost,
    Movement,
    Positioning,
    Demo,
}

impl fmt::Display for StatDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Core => write!(f, "core"),
            Self::Boost => write!(f, "boost"),
            Self::Movement => write!(f, "movement"),
            Self::Positioning => write!(f, "positioning"),
            Self::Demo => write!(f, "demo"),
        }
    }
}
