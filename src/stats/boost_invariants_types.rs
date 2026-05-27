#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoostInvariantKind {
    BucketAmounts,
    NominalPickupAmount,
    NominalStolenPickupAmount,
    CurrentAmount,
    UsedSplitAmounts,
}

impl BoostInvariantKind {
    pub const ALL: [Self; 5] = [
        Self::BucketAmounts,
        Self::NominalPickupAmount,
        Self::NominalStolenPickupAmount,
        Self::CurrentAmount,
        Self::UsedSplitAmounts,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::BucketAmounts => "bucket_amounts",
            Self::NominalPickupAmount => "nominal_pickup_amount",
            Self::NominalStolenPickupAmount => "nominal_stolen_pickup_amount",
            Self::CurrentAmount => "current_amount",
            Self::UsedSplitAmounts => "used_split_amounts",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoostInvariantViolation {
    pub kind: BoostInvariantKind,
    pub expected: f32,
    pub actual: f32,
    pub diff: f32,
    pub tolerance: f32,
}
