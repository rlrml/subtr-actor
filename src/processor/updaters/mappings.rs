use super::*;

#[path = "mappings_ball.rs"]
mod mappings_ball;
#[path = "mappings_bots.rs"]
mod mappings_bots;
#[path = "mappings_context.rs"]
mod mappings_context;
#[path = "mappings_update.rs"]
mod mappings_update;

pub(super) use mappings_context::synthetic_bot_player_id;
