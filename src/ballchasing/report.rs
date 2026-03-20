pub struct BallchasingComparisonReport {
    pub(super) mismatches: Vec<String>,
}

impl BallchasingComparisonReport {
    pub fn is_match(&self) -> bool {
        self.mismatches.is_empty()
    }

    pub fn mismatches(&self) -> &[String] {
        &self.mismatches
    }

    pub fn assert_matches(&self) {
        if self.is_match() {
            return;
        }

        panic!(
            "Ballchasing comparison failed:\n{}",
            self.mismatches.join("\n")
        );
    }
}
