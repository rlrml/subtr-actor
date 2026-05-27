use super::*;
use boost_update_context::BoostUpdateContext;

#[allow(clippy::too_many_arguments)]
pub(super) fn allocate_used_boost(
    player: &PlayerSample,
    stats: &mut BoostStats,
    team_stats: &mut BoostStats,
    amount_used: f32,
    boost_amount: f32,
    boost_before: Option<f32>,
    frame: &FrameInfo,
    vertical_state: &PlayerVerticalState,
    previous_speed: Option<f32>,
    context: &BoostUpdateContext,
) -> Option<BoostLedgerEvent> {
    let split_amount = stats.amount_used_by_vertical_band();
    let amount_used_delta = (amount_used - split_amount).max(0.0);
    if amount_used_delta <= 0.0 {
        return None;
    }

    let used_while_supersonic = used_while_supersonic(
        player,
        previous_speed,
        context.boost_levels_resumed_this_sample,
    );
    let vertical_label = if vertical_state.is_grounded(&player.player_id) {
        vertical_state_label(false)
    } else {
        vertical_state_label(true)
    };
    let used_labels = [
        boost_transaction_label("used"),
        vertical_label,
        boost_supersonic_label(used_while_supersonic),
    ];
    stats.add_labeled_amount(used_labels.clone(), amount_used_delta);
    team_stats.add_labeled_amount(used_labels.clone(), amount_used_delta);
    record_used_vertical_band(
        stats,
        team_stats,
        vertical_state.is_grounded(&player.player_id),
        used_while_supersonic,
        amount_used_delta,
    );

    Some(BoostLedgerEvent {
        frame: frame.frame_number,
        time: frame.time,
        player_id: player.player_id.clone(),
        is_team_0: player.is_team_0,
        transaction: BoostLedgerTransactionKind::UsedAllocation,
        amount: amount_used_delta,
        count: 0,
        labels: used_labels.into_iter().collect(),
        boost_before,
        boost_after: Some(boost_amount),
    })
}

fn used_while_supersonic(
    player: &PlayerSample,
    previous_speed: Option<f32>,
    boost_levels_resumed_this_sample: bool,
) -> bool {
    let speed = player.speed();
    let previous_speed = if boost_levels_resumed_this_sample {
        speed
    } else {
        previous_speed.or(speed)
    };
    player.boost_active
        && speed.unwrap_or(0.0) >= SUPERSONIC_SPEED_THRESHOLD
        && previous_speed.unwrap_or(0.0) >= SUPERSONIC_SPEED_THRESHOLD
}

fn record_used_vertical_band(
    stats: &mut BoostStats,
    team_stats: &mut BoostStats,
    grounded: bool,
    supersonic: bool,
    amount: f32,
) {
    if grounded {
        stats.amount_used_while_grounded += amount;
        team_stats.amount_used_while_grounded += amount;
    } else {
        stats.amount_used_while_airborne += amount;
        team_stats.amount_used_while_airborne += amount;
    }
    if supersonic {
        stats.amount_used_while_supersonic += amount;
        team_stats.amount_used_while_supersonic += amount;
    }
}
