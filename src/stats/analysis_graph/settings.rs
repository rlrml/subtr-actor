use super::graph::*;
use crate::stats::calculators::*;
use crate::*;

pub struct SettingsNode {
    calculator: SettingsCalculator,
}

impl SettingsNode {
    pub fn new() -> Self {
        Self {
            calculator: SettingsCalculator::new(),
        }
    }
}

impl_analysis_node! {
    node = SettingsNode,
    state = SettingsCalculator,
    name = "settings",
    dependencies = [],
    on_replay_meta = |node, meta| {
        node.calculator.apply_replay_meta(meta)
    },
    call = calculator.update,
}
