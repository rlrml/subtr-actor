use super::*;
use std::collections::HashSet;

impl BoostCalculator {
    fn warn_for_boost_invariant_violations(
        &mut self,
        scope: &str,
        frame_number: usize,
        time: f32,
        stats: &BoostStats,
        observed_boost_amount: Option<f32>,
    ) {
        let violations = boost_invariant_violations(stats, observed_boost_amount);
        let active_kinds: HashSet<BoostInvariantKind> =
            violations.iter().map(|violation| violation.kind).collect();

        for violation in violations {
            let key = BoostInvariantWarningKey {
                scope: scope.to_string(),
                kind: violation.kind,
            };
            if self.active_invariant_warnings.insert(key) {
                log::warn!(
                    "Boost invariant violation for {} at frame {} (t={:.3}): {}",
                    scope,
                    frame_number,
                    time,
                    violation.message(),
                );
            }
        }

        for kind in BoostInvariantKind::ALL {
            if active_kinds.contains(&kind) {
                continue;
            }
            self.active_invariant_warnings
                .remove(&BoostInvariantWarningKey {
                    scope: scope.to_string(),
                    kind,
                });
        }
    }

    pub(super) fn warn_for_sample_boost_invariants(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
    ) {
        let team_zero_stats = self.team_zero_stats.clone();
        let team_one_stats = self.team_one_stats.clone();
        let player_scopes: Vec<(PlayerId, Option<f32>, BoostStats)> = players
            .players
            .iter()
            .map(|player| {
                (
                    player.player_id.clone(),
                    player.boost_amount,
                    self.player_stats
                        .get(&player.player_id)
                        .cloned()
                        .unwrap_or_default(),
                )
            })
            .collect();

        self.warn_for_boost_invariant_violations(
            "team_zero",
            frame.frame_number,
            frame.time,
            &team_zero_stats,
            None,
        );
        self.warn_for_boost_invariant_violations(
            "team_one",
            frame.frame_number,
            frame.time,
            &team_one_stats,
            None,
        );
        for (player_id, observed_boost_amount, stats) in player_scopes {
            self.warn_for_boost_invariant_violations(
                &format!("player {player_id:?}"),
                frame.frame_number,
                frame.time,
                &stats,
                observed_boost_amount,
            );
        }
    }
}
