use boxcars;

use crate::*;

/// A sampled ball state at a frame.
#[derive(Debug, Clone)]
pub struct BallSample {
    pub rigid_body: boxcars::RigidBody,
}

impl BallSample {
    pub fn position(&self) -> glam::Vec3 {
        vec_to_glam(&self.rigid_body.location)
    }

    pub fn velocity(&self) -> glam::Vec3 {
        self.rigid_body
            .linear_velocity
            .as_ref()
            .map(vec_to_glam)
            .unwrap_or(glam::Vec3::ZERO)
    }
}

/// A sampled player state at a frame.
#[derive(Debug, Clone)]
pub struct PlayerSample {
    pub player_id: PlayerId,
    pub is_team_0: bool,
    pub hitbox: CarHitbox,
    pub rigid_body: Option<boxcars::RigidBody>,
    pub boost_amount: Option<f32>,
    pub last_boost_amount: Option<f32>,
    pub boost_active: bool,
    pub dodge_active: bool,
    /// World-frame dodge torque of the player's most recent dodge, when
    /// available. This is the flip's rotation axis (a horizontal vector; its
    /// world-z is ~0), and combined with the travel direction at the dodge it
    /// recovers the dodge direction (forward/back vs side) the player input. See
    /// the flick detector's reverse/side classification. `None` on inputs that
    /// do not replicate dodge torque (e.g. the BakkesMod live path).
    pub dodge_torque: Option<glam::Vec3>,
    pub powerslide_active: bool,
    pub match_goals: Option<i32>,
    pub match_assists: Option<i32>,
    pub match_saves: Option<i32>,
    pub match_shots: Option<i32>,
    pub match_score: Option<i32>,
}

impl PlayerSample {
    pub fn position(&self) -> Option<glam::Vec3> {
        self.rigid_body.as_ref().map(|rb| vec_to_glam(&rb.location))
    }

    pub fn velocity(&self) -> Option<glam::Vec3> {
        self.rigid_body
            .as_ref()
            .and_then(|rb| rb.linear_velocity.as_ref().map(vec_to_glam))
    }

    pub fn speed(&self) -> Option<f32> {
        self.velocity().map(|velocity| velocity.length())
    }
}

/// A sampled demolition event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemoEventSample {
    pub attacker: PlayerId,
    pub victim: PlayerId,
}
