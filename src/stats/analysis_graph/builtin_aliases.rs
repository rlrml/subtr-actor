use serde::Serialize;

use super::builtin_names::builtin_analysis_node_names;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BuiltinAnalysisNodeAlias {
    pub alias: &'static str,
    pub node_name: &'static str,
}

pub const BUILTIN_ANALYSIS_NODE_ALIASES: &[BuiltinAnalysisNodeAlias] = &[
    BuiltinAnalysisNodeAlias {
        alias: "core",
        node_name: "match_stats",
    },
    BuiltinAnalysisNodeAlias {
        alias: "air_dribble",
        node_name: "ball_carry",
    },
];

pub fn builtin_analysis_node_aliases() -> &'static [BuiltinAnalysisNodeAlias] {
    BUILTIN_ANALYSIS_NODE_ALIASES
}

pub(super) fn canonical_builtin_analysis_node_name(name: &str) -> Option<&'static str> {
    builtin_analysis_node_aliases()
        .iter()
        .find_map(|alias| (alias.alias == name).then_some(alias.node_name))
        .or_else(|| {
            builtin_analysis_node_names()
                .iter()
                .copied()
                .find(|candidate| *candidate == name)
        })
}
