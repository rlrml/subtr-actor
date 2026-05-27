use super::*;

impl BoostInvariantViolation {
    pub fn message(&self) -> String {
        match self.kind {
            BoostInvariantKind::BucketAmounts => format!(
                "amount_collected_big + amount_collected_small should match amount_collected \
                 (actual={:.1}, expected={:.1}, diff={:.1}, tolerance={:.1})",
                self.actual, self.expected, self.diff, self.tolerance
            ),
            BoostInvariantKind::NominalPickupAmount => format!(
                "amount_collected + overfill_total should match nominal pad value from pickup counts \
                 (actual={:.1}, expected={:.1}, diff={:.1}, tolerance={:.1})",
                self.actual, self.expected, self.diff, self.tolerance
            ),
            BoostInvariantKind::NominalStolenPickupAmount => format!(
                "amount_stolen + overfill_from_stolen should match nominal stolen pad value from pickup counts \
                 (actual={:.1}, expected={:.1}, diff={:.1}, tolerance={:.1})",
                self.actual, self.expected, self.diff, self.tolerance
            ),
            BoostInvariantKind::CurrentAmount => format!(
                "amount_obtained - amount_used should match observed current boost \
                 (actual={:.1}, expected={:.1}, diff={:.1}, tolerance={:.1})",
                self.actual, self.expected, self.diff, self.tolerance
            ),
            BoostInvariantKind::UsedSplitAmounts => format!(
                "amount_used_while_grounded + amount_used_while_airborne should match amount_used \
                 (actual={:.1}, expected={:.1}, diff={:.1}, tolerance={:.1})",
                self.actual, self.expected, self.diff, self.tolerance
            ),
        }
    }
}
