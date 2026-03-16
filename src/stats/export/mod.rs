use serde::Serialize;

mod ball_carry;
mod boost;
mod core;
mod demo;
mod dodge_reset;
mod movement;
mod positioning;
mod possession;
mod powerslide;

pub const LEGACY_STAT_VARIANT: &str = "legacy";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StatUnit {
    Seconds,
    Percent,
    UnrealUnits,
    UnrealUnitsPerSecond,
    Boost,
    BoostPerMinute,
    Count,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StatDescriptor {
    pub domain: &'static str,
    pub name: &'static str,
    pub variant: &'static str,
    pub unit: StatUnit,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "value_type", content = "value", rename_all = "snake_case")]
pub enum StatValue {
    Float(f32),
    Unsigned(u32),
    Signed(i32),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExportedStat {
    #[serde(flatten)]
    pub descriptor: StatDescriptor,
    pub value: StatValue,
}

impl ExportedStat {
    pub fn float(domain: &'static str, name: &'static str, unit: StatUnit, value: f32) -> Self {
        Self {
            descriptor: StatDescriptor {
                domain,
                name,
                variant: LEGACY_STAT_VARIANT,
                unit,
            },
            value: StatValue::Float(value),
        }
    }

    pub fn unsigned(domain: &'static str, name: &'static str, unit: StatUnit, value: u32) -> Self {
        Self {
            descriptor: StatDescriptor {
                domain,
                name,
                variant: LEGACY_STAT_VARIANT,
                unit,
            },
            value: StatValue::Unsigned(value),
        }
    }

    pub fn signed(domain: &'static str, name: &'static str, unit: StatUnit, value: i32) -> Self {
        Self {
            descriptor: StatDescriptor {
                domain,
                name,
                variant: LEGACY_STAT_VARIANT,
                unit,
            },
            value: StatValue::Signed(value),
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
