use serde::{Deserialize, Serialize};

use super::{
    StatDescriptor, StatLabel, StatUnit, StatValue, LABELED_STAT_VARIANT, LEGACY_STAT_VARIANT,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportedStat {
    #[serde(flatten)]
    pub descriptor: StatDescriptor,
    pub value: StatValue,
}

impl ExportedStat {
    pub fn float(domain: &'static str, name: &'static str, unit: StatUnit, value: f32) -> Self {
        Self {
            descriptor: legacy_descriptor(domain, name, unit),
            value: StatValue::Float(value),
        }
    }

    pub fn unsigned(domain: &'static str, name: &'static str, unit: StatUnit, value: u32) -> Self {
        Self {
            descriptor: legacy_descriptor(domain, name, unit),
            value: StatValue::Unsigned(value),
        }
    }

    pub fn signed(domain: &'static str, name: &'static str, unit: StatUnit, value: i32) -> Self {
        Self {
            descriptor: legacy_descriptor(domain, name, unit),
            value: StatValue::Signed(value),
        }
    }

    pub fn unsigned_labeled(
        domain: &'static str,
        name: &'static str,
        unit: StatUnit,
        labels: Vec<StatLabel>,
        value: u32,
    ) -> Self {
        Self {
            descriptor: labeled_descriptor(domain, name, unit, labels),
            value: StatValue::Unsigned(value),
        }
    }

    pub fn float_labeled(
        domain: &'static str,
        name: &'static str,
        unit: StatUnit,
        labels: Vec<StatLabel>,
        value: f32,
    ) -> Self {
        Self {
            descriptor: labeled_descriptor(domain, name, unit, labels),
            value: StatValue::Float(value),
        }
    }
}

pub trait StatFieldProvider {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat));

    fn stat_fields(&self) -> Vec<ExportedStat> {
        let mut fields = Vec::new();
        self.visit_stat_fields(&mut |field| fields.push(field));
        fields
    }
}

fn legacy_descriptor(domain: &'static str, name: &'static str, unit: StatUnit) -> StatDescriptor {
    StatDescriptor {
        domain,
        name,
        variant: LEGACY_STAT_VARIANT,
        unit,
        labels: Vec::new(),
    }
}

fn labeled_descriptor(
    domain: &'static str,
    name: &'static str,
    unit: StatUnit,
    labels: Vec<StatLabel>,
) -> StatDescriptor {
    StatDescriptor {
        domain,
        name,
        variant: LABELED_STAT_VARIANT,
        unit,
        labels,
    }
}
