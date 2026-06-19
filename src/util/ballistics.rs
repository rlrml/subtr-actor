use crate::{apply_velocities_to_rigid_body, glam_to_vec, vec_to_glam};

/// Rocket League's standard physics tick rate in Hz.
pub const ROCKET_LEAGUE_PHYSICS_TICK_RATE_HZ: f32 = 120.0;

/// Standard Soccar gravity in unreal units per second squared.
pub const STANDARD_BALL_GRAVITY_Z: f32 = -650.0;

/// Standard Soccar ball speed cap in unreal units per second.
pub const STANDARD_BALL_MAX_SPEED: f32 = 6000.0;

/// Standard Soccar ball radius used by common ball-prediction models.
pub const STANDARD_BALL_RADIUS: f32 = 91.25;

/// Approximate ball restitution for wall/ground bounces.
pub const STANDARD_BALL_RESTITUTION: f32 = 0.6;

/// Tangential impulse coefficient from smish.dev's Rocket League ball model.
pub const STANDARD_BALL_TANGENTIAL_FRICTION: f32 = 0.285;

/// Tangential impulse scale from smish.dev's Rocket League ball model.
pub const STANDARD_BALL_TANGENTIAL_RATIO_SCALE: f32 = 2.0;

/// Angular velocity coupling from smish.dev's Rocket League ball model.
pub const STANDARD_BALL_ANGULAR_COUPLING: f32 = 0.0003;

/// Standard Soccar goal line Y coordinate.
pub const STANDARD_GOAL_LINE_Y: f32 = 5120.0;

/// Standard Soccar back-wall Y coordinate.
pub const STANDARD_ARENA_BACK_WALL_Y: f32 = STANDARD_GOAL_LINE_Y;

/// Standard Soccar goal center-to-post distance in unreal units.
pub const STANDARD_GOAL_MOUTH_HALF_WIDTH_X: f32 = 892.755;

/// Approximate goal mouth height in unreal units.
pub const STANDARD_GOAL_MOUTH_HEIGHT_Z: f32 = 642.775;

/// Approximate radius for goal posts and crossbars in the standard arena model.
pub const STANDARD_GOAL_FRAME_RADIUS: f32 = 75.0;

/// Approximate standard Soccar wall-bottom ramp radius. RLBot documents this as
/// roughly 256 uu, with the caveat that the real mesh is not a perfect circle.
pub const STANDARD_ARENA_WALL_BOTTOM_RAMP_RADIUS: f32 = 256.0;

/// Default tolerance for trajectory-to-goal-mouth checks.
pub const STANDARD_GOAL_MOUTH_TRAJECTORY_MARGIN: f32 = STANDARD_BALL_RADIUS * 1.5;

/// Standard Soccar side-wall X coordinate.
pub const STANDARD_ARENA_SIDE_WALL_X: f32 = 4096.0;

/// Standard Soccar ceiling Z coordinate.
pub const STANDARD_ARENA_CEILING_Z: f32 = 2044.0;

const MIN_TICK_RATE_HZ: f32 = 1.0;
const MAX_INTEGRATION_STEPS: usize = 1_000_000;
const MAX_COLLISIONS_PER_STEP: usize = 16;
const COLLISION_TIME_EPSILON: f32 = 0.000_001;
const ARENA_BOUND_EPSILON: f32 = 0.001;
const RESTING_CONTACT_NORMAL_SPEED_THRESHOLD: f32 = 50.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BallTrajectoryIntegration {
    /// Match Rocket League's fixed-step simulation style by applying acceleration
    /// to velocity before integrating position for each substep.
    #[default]
    SemiImplicitEuler,
    /// Use closed-form constant-acceleration displacement for each substep.
    ClosedForm,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BallTrajectoryConfig {
    pub gravity: glam::Vec3,
    pub max_speed: f32,
    pub tick_rate_hz: f32,
    pub integration: BallTrajectoryIntegration,
}

impl BallTrajectoryConfig {
    pub const STANDARD_SOCCAR: Self = Self {
        gravity: glam::Vec3::new(0.0, 0.0, STANDARD_BALL_GRAVITY_Z),
        max_speed: STANDARD_BALL_MAX_SPEED,
        tick_rate_hz: ROCKET_LEAGUE_PHYSICS_TICK_RATE_HZ,
        integration: BallTrajectoryIntegration::SemiImplicitEuler,
    };

    fn fixed_step_seconds(self) -> f32 {
        1.0 / self.tick_rate_hz.max(MIN_TICK_RATE_HZ)
    }
}

impl Default for BallTrajectoryConfig {
    fn default() -> Self {
        Self::STANDARD_SOCCAR
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BallBounceConfig {
    pub radius: f32,
    pub restitution: f32,
    pub tangential_friction: f32,
    pub tangential_ratio_scale: f32,
    pub angular_coupling: f32,
}

impl BallBounceConfig {
    pub const STANDARD_SOCCAR: Self = Self {
        radius: STANDARD_BALL_RADIUS,
        restitution: STANDARD_BALL_RESTITUTION,
        tangential_friction: STANDARD_BALL_TANGENTIAL_FRICTION,
        tangential_ratio_scale: STANDARD_BALL_TANGENTIAL_RATIO_SCALE,
        angular_coupling: STANDARD_BALL_ANGULAR_COUPLING,
    };
}

impl Default for BallBounceConfig {
    fn default() -> Self {
        Self::STANDARD_SOCCAR
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BallCollisionPlaneBounds {
    pub min: glam::Vec3,
    pub max: glam::Vec3,
}

impl BallCollisionPlaneBounds {
    pub const fn new(min: glam::Vec3, max: glam::Vec3) -> Self {
        Self { min, max }
    }

    pub fn contains(self, position: glam::Vec3) -> bool {
        position.x + ARENA_BOUND_EPSILON >= self.min.x
            && position.x - ARENA_BOUND_EPSILON <= self.max.x
            && position.y + ARENA_BOUND_EPSILON >= self.min.y
            && position.y - ARENA_BOUND_EPSILON <= self.max.y
            && position.z + ARENA_BOUND_EPSILON >= self.min.z
            && position.z - ARENA_BOUND_EPSILON <= self.max.z
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BallCollisionPlane {
    /// Unit normal pointing into the playable half-space.
    pub normal: glam::Vec3,
    /// Plane constant in `normal.dot(position) == distance_from_origin` form.
    pub distance_from_origin: f32,
    /// Optional axis-aligned bounds for finite wall sections. Bounds apply to
    /// the ball center at the impact point, not to the mesh contact point.
    pub bounds: Option<BallCollisionPlaneBounds>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BallCollisionCylinderAxis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BallCollisionCylinder {
    pub axis: BallCollisionCylinderAxis,
    pub center: glam::Vec3,
    pub radius: f32,
    pub min_axis: f32,
    pub max_axis: f32,
}

impl BallCollisionCylinder {
    pub const fn new(
        axis: BallCollisionCylinderAxis,
        center: glam::Vec3,
        radius: f32,
        min_axis: f32,
        max_axis: f32,
    ) -> Self {
        Self {
            axis,
            center,
            radius,
            min_axis,
            max_axis,
        }
    }

    pub const fn standard_positive_goal_left_post() -> Self {
        Self::standard_goal_post(
            -STANDARD_GOAL_MOUTH_HALF_WIDTH_X - STANDARD_GOAL_FRAME_RADIUS,
            STANDARD_GOAL_LINE_Y,
        )
    }

    pub const fn standard_positive_goal_right_post() -> Self {
        Self::standard_goal_post(
            STANDARD_GOAL_MOUTH_HALF_WIDTH_X + STANDARD_GOAL_FRAME_RADIUS,
            STANDARD_GOAL_LINE_Y,
        )
    }

    pub const fn standard_negative_goal_left_post() -> Self {
        Self::standard_goal_post(
            -STANDARD_GOAL_MOUTH_HALF_WIDTH_X - STANDARD_GOAL_FRAME_RADIUS,
            -STANDARD_GOAL_LINE_Y,
        )
    }

    pub const fn standard_negative_goal_right_post() -> Self {
        Self::standard_goal_post(
            STANDARD_GOAL_MOUTH_HALF_WIDTH_X + STANDARD_GOAL_FRAME_RADIUS,
            -STANDARD_GOAL_LINE_Y,
        )
    }

    pub const fn standard_positive_goal_crossbar() -> Self {
        Self::standard_goal_crossbar(STANDARD_GOAL_LINE_Y)
    }

    pub const fn standard_negative_goal_crossbar() -> Self {
        Self::standard_goal_crossbar(-STANDARD_GOAL_LINE_Y)
    }

    const fn standard_goal_post(x: f32, y: f32) -> Self {
        Self::new(
            BallCollisionCylinderAxis::Z,
            glam::Vec3::new(x, y, 0.0),
            STANDARD_GOAL_FRAME_RADIUS,
            0.0,
            STANDARD_GOAL_MOUTH_HEIGHT_Z + STANDARD_GOAL_FRAME_RADIUS,
        )
    }

    const fn standard_goal_crossbar(y: f32) -> Self {
        Self::new(
            BallCollisionCylinderAxis::X,
            glam::Vec3::new(
                0.0,
                y,
                STANDARD_GOAL_MOUTH_HEIGHT_Z + STANDARD_GOAL_FRAME_RADIUS,
            ),
            STANDARD_GOAL_FRAME_RADIUS,
            -STANDARD_GOAL_MOUTH_HALF_WIDTH_X - STANDARD_GOAL_FRAME_RADIUS,
            STANDARD_GOAL_MOUTH_HALF_WIDTH_X + STANDARD_GOAL_FRAME_RADIUS,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BallCollisionConcaveCylinder {
    pub axis: BallCollisionCylinderAxis,
    pub center: glam::Vec3,
    pub radius: f32,
    pub min_axis: f32,
    pub max_axis: f32,
    pub bounds: BallCollisionPlaneBounds,
}

impl BallCollisionConcaveCylinder {
    pub const fn new(
        axis: BallCollisionCylinderAxis,
        center: glam::Vec3,
        radius: f32,
        min_axis: f32,
        max_axis: f32,
        bounds: BallCollisionPlaneBounds,
    ) -> Self {
        Self {
            axis,
            center,
            radius,
            min_axis,
            max_axis,
            bounds,
        }
    }

    pub fn standard_positive_x_wall_bottom_ramp() -> Self {
        let center_x = STANDARD_ARENA_SIDE_WALL_X - STANDARD_ARENA_WALL_BOTTOM_RAMP_RADIUS;
        Self::new(
            BallCollisionCylinderAxis::Y,
            glam::Vec3::new(center_x, 0.0, STANDARD_ARENA_WALL_BOTTOM_RAMP_RADIUS),
            STANDARD_ARENA_WALL_BOTTOM_RAMP_RADIUS,
            -STANDARD_ARENA_BACK_WALL_Y,
            STANDARD_ARENA_BACK_WALL_Y,
            BallCollisionPlaneBounds::new(
                glam::Vec3::new(center_x, -STANDARD_ARENA_BACK_WALL_Y, 0.0),
                glam::Vec3::new(
                    STANDARD_ARENA_SIDE_WALL_X,
                    STANDARD_ARENA_BACK_WALL_Y,
                    STANDARD_ARENA_WALL_BOTTOM_RAMP_RADIUS,
                ),
            ),
        )
    }

    pub fn standard_negative_x_wall_bottom_ramp() -> Self {
        let center_x = -STANDARD_ARENA_SIDE_WALL_X + STANDARD_ARENA_WALL_BOTTOM_RAMP_RADIUS;
        Self::new(
            BallCollisionCylinderAxis::Y,
            glam::Vec3::new(center_x, 0.0, STANDARD_ARENA_WALL_BOTTOM_RAMP_RADIUS),
            STANDARD_ARENA_WALL_BOTTOM_RAMP_RADIUS,
            -STANDARD_ARENA_BACK_WALL_Y,
            STANDARD_ARENA_BACK_WALL_Y,
            BallCollisionPlaneBounds::new(
                glam::Vec3::new(
                    -STANDARD_ARENA_SIDE_WALL_X,
                    -STANDARD_ARENA_BACK_WALL_Y,
                    0.0,
                ),
                glam::Vec3::new(
                    center_x,
                    STANDARD_ARENA_BACK_WALL_Y,
                    STANDARD_ARENA_WALL_BOTTOM_RAMP_RADIUS,
                ),
            ),
        )
    }

    pub fn standard_positive_y_wall_bottom_ramp_left() -> Self {
        Self::standard_y_wall_bottom_ramp(
            STANDARD_ARENA_BACK_WALL_Y,
            -STANDARD_ARENA_SIDE_WALL_X,
            -STANDARD_GOAL_MOUTH_HALF_WIDTH_X - STANDARD_BALL_RADIUS,
        )
    }

    pub fn standard_positive_y_wall_bottom_ramp_right() -> Self {
        Self::standard_y_wall_bottom_ramp(
            STANDARD_ARENA_BACK_WALL_Y,
            STANDARD_GOAL_MOUTH_HALF_WIDTH_X + STANDARD_BALL_RADIUS,
            STANDARD_ARENA_SIDE_WALL_X,
        )
    }

    pub fn standard_negative_y_wall_bottom_ramp_left() -> Self {
        Self::standard_y_wall_bottom_ramp(
            -STANDARD_ARENA_BACK_WALL_Y,
            -STANDARD_ARENA_SIDE_WALL_X,
            -STANDARD_GOAL_MOUTH_HALF_WIDTH_X - STANDARD_BALL_RADIUS,
        )
    }

    pub fn standard_negative_y_wall_bottom_ramp_right() -> Self {
        Self::standard_y_wall_bottom_ramp(
            -STANDARD_ARENA_BACK_WALL_Y,
            STANDARD_GOAL_MOUTH_HALF_WIDTH_X + STANDARD_BALL_RADIUS,
            STANDARD_ARENA_SIDE_WALL_X,
        )
    }

    fn standard_y_wall_bottom_ramp(y: f32, min_x: f32, max_x: f32) -> Self {
        let center_y = if y > 0.0 {
            y - STANDARD_ARENA_WALL_BOTTOM_RAMP_RADIUS
        } else {
            y + STANDARD_ARENA_WALL_BOTTOM_RAMP_RADIUS
        };
        let min_y = if y > 0.0 { center_y } else { y };
        let max_y = if y > 0.0 { y } else { center_y };
        Self::new(
            BallCollisionCylinderAxis::X,
            glam::Vec3::new(0.0, center_y, STANDARD_ARENA_WALL_BOTTOM_RAMP_RADIUS),
            STANDARD_ARENA_WALL_BOTTOM_RAMP_RADIUS,
            min_x,
            max_x,
            BallCollisionPlaneBounds::new(
                glam::Vec3::new(min_x, min_y, 0.0),
                glam::Vec3::new(max_x, max_y, STANDARD_ARENA_WALL_BOTTOM_RAMP_RADIUS),
            ),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BallCollisionSurface {
    Plane(BallCollisionPlane),
    Cylinder(BallCollisionCylinder),
    ConcaveCylinder(BallCollisionConcaveCylinder),
}

impl From<BallCollisionPlane> for BallCollisionSurface {
    fn from(plane: BallCollisionPlane) -> Self {
        Self::Plane(plane)
    }
}

impl From<BallCollisionCylinder> for BallCollisionSurface {
    fn from(cylinder: BallCollisionCylinder) -> Self {
        Self::Cylinder(cylinder)
    }
}

impl From<BallCollisionConcaveCylinder> for BallCollisionSurface {
    fn from(cylinder: BallCollisionConcaveCylinder) -> Self {
        Self::ConcaveCylinder(cylinder)
    }
}

impl BallCollisionPlane {
    pub fn new(normal: glam::Vec3, distance_from_origin: f32) -> Option<Self> {
        if !normal.is_finite() || normal.length_squared() <= f32::EPSILON {
            return None;
        }
        Some(Self {
            normal: normal.normalize(),
            distance_from_origin,
            bounds: None,
        })
    }

    pub const fn from_unit_normal(normal: glam::Vec3, distance_from_origin: f32) -> Self {
        Self {
            normal,
            distance_from_origin,
            bounds: None,
        }
    }

    pub const fn with_bounds(mut self, bounds: BallCollisionPlaneBounds) -> Self {
        self.bounds = Some(bounds);
        self
    }

    pub fn contains_impact_point(self, position: glam::Vec3) -> bool {
        self.bounds.is_none_or(|bounds| bounds.contains(position))
    }

    pub const fn standard_ground() -> Self {
        Self::from_unit_normal(glam::Vec3::Z, 0.0)
    }

    pub const fn standard_positive_x_wall() -> Self {
        Self::from_unit_normal(glam::Vec3::NEG_X, -STANDARD_ARENA_SIDE_WALL_X)
    }

    pub const fn standard_negative_x_wall() -> Self {
        Self::from_unit_normal(glam::Vec3::X, -STANDARD_ARENA_SIDE_WALL_X)
    }

    pub const fn standard_positive_y_wall() -> Self {
        Self::from_unit_normal(glam::Vec3::NEG_Y, -STANDARD_ARENA_BACK_WALL_Y)
    }

    pub const fn standard_negative_y_wall() -> Self {
        Self::from_unit_normal(glam::Vec3::Y, -STANDARD_ARENA_BACK_WALL_Y)
    }

    pub const fn standard_ceiling() -> Self {
        Self::from_unit_normal(glam::Vec3::NEG_Z, -STANDARD_ARENA_CEILING_Z)
    }

    pub const fn standard_ground_bounded() -> Self {
        Self::standard_ground().with_bounds(Self::standard_full_arena_bounds())
    }

    pub const fn standard_positive_x_wall_bounded() -> Self {
        Self::standard_positive_x_wall().with_bounds(Self::standard_full_arena_bounds())
    }

    pub const fn standard_negative_x_wall_bounded() -> Self {
        Self::standard_negative_x_wall().with_bounds(Self::standard_full_arena_bounds())
    }

    pub const fn standard_ceiling_bounded() -> Self {
        Self::standard_ceiling().with_bounds(Self::standard_full_arena_bounds())
    }

    pub const fn standard_positive_y_wall_left_bounded() -> Self {
        Self::standard_positive_y_wall().with_bounds(BallCollisionPlaneBounds::new(
            glam::Vec3::new(
                -STANDARD_ARENA_SIDE_WALL_X,
                -STANDARD_ARENA_BACK_WALL_Y,
                0.0,
            ),
            glam::Vec3::new(
                -STANDARD_GOAL_MOUTH_HALF_WIDTH_X - STANDARD_BALL_RADIUS,
                STANDARD_ARENA_BACK_WALL_Y,
                STANDARD_ARENA_CEILING_Z,
            ),
        ))
    }

    pub const fn standard_positive_y_wall_right_bounded() -> Self {
        Self::standard_positive_y_wall().with_bounds(BallCollisionPlaneBounds::new(
            glam::Vec3::new(
                STANDARD_GOAL_MOUTH_HALF_WIDTH_X + STANDARD_BALL_RADIUS,
                -STANDARD_ARENA_BACK_WALL_Y,
                0.0,
            ),
            glam::Vec3::new(
                STANDARD_ARENA_SIDE_WALL_X,
                STANDARD_ARENA_BACK_WALL_Y,
                STANDARD_ARENA_CEILING_Z,
            ),
        ))
    }

    pub const fn standard_positive_y_wall_above_goal_bounded() -> Self {
        Self::standard_positive_y_wall().with_bounds(BallCollisionPlaneBounds::new(
            glam::Vec3::new(
                -STANDARD_GOAL_MOUTH_HALF_WIDTH_X - STANDARD_BALL_RADIUS,
                -STANDARD_ARENA_BACK_WALL_Y,
                STANDARD_GOAL_MOUTH_HEIGHT_Z + STANDARD_BALL_RADIUS,
            ),
            glam::Vec3::new(
                STANDARD_GOAL_MOUTH_HALF_WIDTH_X + STANDARD_BALL_RADIUS,
                STANDARD_ARENA_BACK_WALL_Y,
                STANDARD_ARENA_CEILING_Z,
            ),
        ))
    }

    pub const fn standard_negative_y_wall_left_bounded() -> Self {
        Self::standard_negative_y_wall().with_bounds(BallCollisionPlaneBounds::new(
            glam::Vec3::new(
                -STANDARD_ARENA_SIDE_WALL_X,
                -STANDARD_ARENA_BACK_WALL_Y,
                0.0,
            ),
            glam::Vec3::new(
                -STANDARD_GOAL_MOUTH_HALF_WIDTH_X - STANDARD_BALL_RADIUS,
                STANDARD_ARENA_BACK_WALL_Y,
                STANDARD_ARENA_CEILING_Z,
            ),
        ))
    }

    pub const fn standard_negative_y_wall_right_bounded() -> Self {
        Self::standard_negative_y_wall().with_bounds(BallCollisionPlaneBounds::new(
            glam::Vec3::new(
                STANDARD_GOAL_MOUTH_HALF_WIDTH_X + STANDARD_BALL_RADIUS,
                -STANDARD_ARENA_BACK_WALL_Y,
                0.0,
            ),
            glam::Vec3::new(
                STANDARD_ARENA_SIDE_WALL_X,
                STANDARD_ARENA_BACK_WALL_Y,
                STANDARD_ARENA_CEILING_Z,
            ),
        ))
    }

    pub const fn standard_negative_y_wall_above_goal_bounded() -> Self {
        Self::standard_negative_y_wall().with_bounds(BallCollisionPlaneBounds::new(
            glam::Vec3::new(
                -STANDARD_GOAL_MOUTH_HALF_WIDTH_X - STANDARD_BALL_RADIUS,
                -STANDARD_ARENA_BACK_WALL_Y,
                STANDARD_GOAL_MOUTH_HEIGHT_Z + STANDARD_BALL_RADIUS,
            ),
            glam::Vec3::new(
                STANDARD_GOAL_MOUTH_HALF_WIDTH_X + STANDARD_BALL_RADIUS,
                STANDARD_ARENA_BACK_WALL_Y,
                STANDARD_ARENA_CEILING_Z,
            ),
        ))
    }

    const fn standard_full_arena_bounds() -> BallCollisionPlaneBounds {
        BallCollisionPlaneBounds::new(
            glam::Vec3::new(
                -STANDARD_ARENA_SIDE_WALL_X,
                -STANDARD_ARENA_BACK_WALL_Y,
                0.0,
            ),
            glam::Vec3::new(
                STANDARD_ARENA_SIDE_WALL_X,
                STANDARD_ARENA_BACK_WALL_Y,
                STANDARD_ARENA_CEILING_Z,
            ),
        )
    }

    pub fn center_distance(self, position: glam::Vec3) -> f32 {
        self.normal.dot(position) - self.distance_from_origin
    }

    pub fn penetration_depth(self, position: glam::Vec3, radius: f32) -> f32 {
        radius - self.center_distance(position)
    }
}

/// Collision planes that matter before evaluating a standard Soccar goal-line
/// crossing. Back-wall planes are intentionally omitted because the target
/// crossing plane is the back goal line itself.
pub const fn standard_soccar_goal_line_prediction_planes() -> [BallCollisionPlane; 4] {
    [
        BallCollisionPlane::standard_ground_bounded(),
        BallCollisionPlane::standard_positive_x_wall_bounded(),
        BallCollisionPlane::standard_negative_x_wall_bounded(),
        BallCollisionPlane::standard_ceiling_bounded(),
    ]
}

/// Simple rectangular standard Soccar arena planes for replay-wide prediction
/// audits. This intentionally approximates the true arena and omits ramps,
/// curved corners, post geometry, and goal interiors.
pub const fn standard_soccar_prediction_planes() -> [BallCollisionPlane; 10] {
    [
        BallCollisionPlane::standard_ground_bounded(),
        BallCollisionPlane::standard_positive_x_wall_bounded(),
        BallCollisionPlane::standard_negative_x_wall_bounded(),
        BallCollisionPlane::standard_positive_y_wall_left_bounded(),
        BallCollisionPlane::standard_positive_y_wall_right_bounded(),
        BallCollisionPlane::standard_positive_y_wall_above_goal_bounded(),
        BallCollisionPlane::standard_negative_y_wall_left_bounded(),
        BallCollisionPlane::standard_negative_y_wall_right_bounded(),
        BallCollisionPlane::standard_negative_y_wall_above_goal_bounded(),
        BallCollisionPlane::standard_ceiling_bounded(),
    ]
}

pub fn standard_soccar_goal_frame_surfaces() -> [BallCollisionSurface; 6] {
    [
        BallCollisionSurface::Cylinder(BallCollisionCylinder::standard_positive_goal_left_post()),
        BallCollisionSurface::Cylinder(BallCollisionCylinder::standard_positive_goal_right_post()),
        BallCollisionSurface::Cylinder(BallCollisionCylinder::standard_positive_goal_crossbar()),
        BallCollisionSurface::Cylinder(BallCollisionCylinder::standard_negative_goal_left_post()),
        BallCollisionSurface::Cylinder(BallCollisionCylinder::standard_negative_goal_right_post()),
        BallCollisionSurface::Cylinder(BallCollisionCylinder::standard_negative_goal_crossbar()),
    ]
}

/// Collision surfaces before the goal line, excluding goal-frame cylinders.
///
/// This is useful when the desired answer is the counterfactual goal-line
/// crossing location itself, rather than whether the ball would hit a post or
/// crossbar before entering the goal.
pub const fn standard_soccar_goal_line_prediction_field_surfaces() -> [BallCollisionSurface; 4] {
    [
        BallCollisionSurface::Plane(BallCollisionPlane::standard_ground_bounded()),
        BallCollisionSurface::Plane(BallCollisionPlane::standard_positive_x_wall_bounded()),
        BallCollisionSurface::Plane(BallCollisionPlane::standard_negative_x_wall_bounded()),
        BallCollisionSurface::Plane(BallCollisionPlane::standard_ceiling_bounded()),
    ]
}

/// Collision surfaces that matter before evaluating a standard Soccar goal-line
/// crossing. Back-wall planes are still omitted because the target crossing
/// plane is the back goal line itself, but goal-frame cylinders are included so
/// projected post and crossbar bounces are not treated as unobstructed crosses.
pub fn standard_soccar_goal_line_prediction_surfaces() -> [BallCollisionSurface; 10] {
    [
        BallCollisionSurface::Plane(BallCollisionPlane::standard_ground_bounded()),
        BallCollisionSurface::Plane(BallCollisionPlane::standard_positive_x_wall_bounded()),
        BallCollisionSurface::Plane(BallCollisionPlane::standard_negative_x_wall_bounded()),
        BallCollisionSurface::Plane(BallCollisionPlane::standard_ceiling_bounded()),
        BallCollisionSurface::Cylinder(BallCollisionCylinder::standard_positive_goal_left_post()),
        BallCollisionSurface::Cylinder(BallCollisionCylinder::standard_positive_goal_right_post()),
        BallCollisionSurface::Cylinder(BallCollisionCylinder::standard_positive_goal_crossbar()),
        BallCollisionSurface::Cylinder(BallCollisionCylinder::standard_negative_goal_left_post()),
        BallCollisionSurface::Cylinder(BallCollisionCylinder::standard_negative_goal_right_post()),
        BallCollisionSurface::Cylinder(BallCollisionCylinder::standard_negative_goal_crossbar()),
    ]
}

/// Simple standard Soccar arena surfaces for replay-wide prediction audits. This
/// remains an approximation: ramps, curved corners, detailed goal interiors, and
/// exact mesh normals are not modeled.
pub fn standard_soccar_prediction_surfaces() -> [BallCollisionSurface; 16] {
    [
        BallCollisionSurface::Plane(BallCollisionPlane::standard_ground_bounded()),
        BallCollisionSurface::Plane(BallCollisionPlane::standard_positive_x_wall_bounded()),
        BallCollisionSurface::Plane(BallCollisionPlane::standard_negative_x_wall_bounded()),
        BallCollisionSurface::Plane(BallCollisionPlane::standard_positive_y_wall_left_bounded()),
        BallCollisionSurface::Plane(BallCollisionPlane::standard_positive_y_wall_right_bounded()),
        BallCollisionSurface::Plane(
            BallCollisionPlane::standard_positive_y_wall_above_goal_bounded(),
        ),
        BallCollisionSurface::Plane(BallCollisionPlane::standard_negative_y_wall_left_bounded()),
        BallCollisionSurface::Plane(BallCollisionPlane::standard_negative_y_wall_right_bounded()),
        BallCollisionSurface::Plane(
            BallCollisionPlane::standard_negative_y_wall_above_goal_bounded(),
        ),
        BallCollisionSurface::Plane(BallCollisionPlane::standard_ceiling_bounded()),
        BallCollisionSurface::Cylinder(BallCollisionCylinder::standard_positive_goal_left_post()),
        BallCollisionSurface::Cylinder(BallCollisionCylinder::standard_positive_goal_right_post()),
        BallCollisionSurface::Cylinder(BallCollisionCylinder::standard_positive_goal_crossbar()),
        BallCollisionSurface::Cylinder(BallCollisionCylinder::standard_negative_goal_left_post()),
        BallCollisionSurface::Cylinder(BallCollisionCylinder::standard_negative_goal_right_post()),
        BallCollisionSurface::Cylinder(BallCollisionCylinder::standard_negative_goal_crossbar()),
    ]
}

/// Collision surfaces used when asking where a shot would first reach the target
/// goal area. Field surfaces can redirect the ball before it arrives; target
/// back-wall sections and goal-frame cylinders are reported as hits.
pub fn standard_soccar_goal_target_prediction_surfaces() -> [BallCollisionSurface; 16] {
    standard_soccar_prediction_surfaces()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BallTrajectorySample {
    pub time: f32,
    pub rigid_body: boxcars::RigidBody,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BallGoalLineCrossing {
    pub time: f32,
    pub position: glam::Vec3,
    pub velocity: Option<glam::Vec3>,
    pub inside_goal_mouth: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BallGoalTargetHitKind {
    GoalLine,
    BackWall,
    GoalFrame,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BallGoalTargetHit {
    pub time: f32,
    pub position: glam::Vec3,
    pub velocity: Option<glam::Vec3>,
    pub hit_kind: BallGoalTargetHitKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BallGoalLineCrossingConfig {
    pub target_goal_y: f32,
    pub max_seconds: f32,
    pub goal_mouth_half_width_x: f32,
    pub goal_mouth_height_z: f32,
    pub goal_mouth_margin: f32,
}

impl BallGoalLineCrossingConfig {
    pub const fn team_zero_attacking_goal() -> Self {
        Self {
            target_goal_y: STANDARD_GOAL_LINE_Y,
            max_seconds: 6.0,
            goal_mouth_half_width_x: STANDARD_GOAL_MOUTH_HALF_WIDTH_X,
            goal_mouth_height_z: STANDARD_GOAL_MOUTH_HEIGHT_Z,
            goal_mouth_margin: STANDARD_GOAL_MOUTH_TRAJECTORY_MARGIN,
        }
    }

    pub const fn team_one_attacking_goal() -> Self {
        Self {
            target_goal_y: -STANDARD_GOAL_LINE_Y,
            max_seconds: 6.0,
            goal_mouth_half_width_x: STANDARD_GOAL_MOUTH_HALF_WIDTH_X,
            goal_mouth_height_z: STANDARD_GOAL_MOUTH_HEIGHT_Z,
            goal_mouth_margin: STANDARD_GOAL_MOUTH_TRAJECTORY_MARGIN,
        }
    }

    pub const fn attacking_goal(is_team_0: bool) -> Self {
        if is_team_0 {
            Self::team_zero_attacking_goal()
        } else {
            Self::team_one_attacking_goal()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BallTrajectoryError {
    pub sample_count: usize,
    pub max_position_error: f32,
    pub rms_position_error: f32,
    pub max_velocity_error: Option<f32>,
    pub rms_velocity_error: Option<f32>,
}

/// Advances a ball rigid body through free flight only.
///
/// This intentionally ignores cars, arena surfaces, gravity mutators, beach-ball
/// curve, heatseeker steering, and any other collision or externally applied
/// force. Angular velocity is retained for orientation, but it does not curve the
/// ball's center-of-mass path in standard Soccar.
pub fn advance_ball_free_flight(
    initial: &boxcars::RigidBody,
    duration_seconds: f32,
    config: BallTrajectoryConfig,
) -> boxcars::RigidBody {
    if duration_seconds <= 0.0 {
        return *initial;
    }

    let mut current = *initial;
    let mut remaining = duration_seconds;
    let fixed_step_seconds = config.fixed_step_seconds();
    let mut steps = 0usize;

    while remaining > f32::EPSILON && steps < MAX_INTEGRATION_STEPS {
        let step_seconds = remaining.min(fixed_step_seconds);
        current = advance_ball_free_flight_step(&current, step_seconds, config);
        remaining -= step_seconds;
        steps += 1;
    }

    current
}

/// Advances a ball through free flight and resolves bounces against simple
/// planes after each physics substep.
///
/// This is useful for ground/wall-style tests and rough predictions. Accurate
/// arena prediction should use the same bounce model with normals from Rocket
/// League's collision meshes.
pub fn advance_ball_with_plane_bounces(
    initial: &boxcars::RigidBody,
    duration_seconds: f32,
    trajectory_config: BallTrajectoryConfig,
    bounce_config: BallBounceConfig,
    planes: &[BallCollisionPlane],
) -> boxcars::RigidBody {
    if duration_seconds <= 0.0 {
        return *initial;
    }

    let mut current = *initial;
    let mut remaining = duration_seconds;
    let fixed_step_seconds = trajectory_config.fixed_step_seconds();
    let mut steps = 0usize;

    while remaining > f32::EPSILON && steps < MAX_INTEGRATION_STEPS {
        let step_seconds = remaining.min(fixed_step_seconds);
        current = advance_ball_with_plane_bounces_step(
            &current,
            step_seconds,
            trajectory_config,
            bounce_config,
            planes,
        );
        remaining -= step_seconds;
        steps += 1;
    }

    current
}

pub fn advance_ball_with_surface_bounces(
    initial: &boxcars::RigidBody,
    duration_seconds: f32,
    trajectory_config: BallTrajectoryConfig,
    bounce_config: BallBounceConfig,
    surfaces: &[BallCollisionSurface],
) -> boxcars::RigidBody {
    if duration_seconds <= 0.0 {
        return *initial;
    }

    let mut current = *initial;
    let mut remaining = duration_seconds;
    let fixed_step_seconds = trajectory_config.fixed_step_seconds();
    let mut steps = 0usize;

    while remaining > f32::EPSILON && steps < MAX_INTEGRATION_STEPS {
        let step_seconds = remaining.min(fixed_step_seconds);
        current = advance_ball_with_surface_bounces_step(
            &current,
            step_seconds,
            trajectory_config,
            bounce_config,
            surfaces,
        );
        remaining -= step_seconds;
        steps += 1;
    }

    current
}

fn advance_ball_free_flight_step(
    current: &boxcars::RigidBody,
    step_seconds: f32,
    config: BallTrajectoryConfig,
) -> boxcars::RigidBody {
    let mut advanced = *current;
    let position = vec_to_glam(&current.location);
    let velocity = current
        .linear_velocity
        .as_ref()
        .map(vec_to_glam)
        .unwrap_or(glam::Vec3::ZERO);

    let mut next_velocity = velocity + config.gravity * step_seconds;
    next_velocity = clamp_speed(next_velocity, config.max_speed);

    let next_position = match config.integration {
        BallTrajectoryIntegration::SemiImplicitEuler => position + next_velocity * step_seconds,
        BallTrajectoryIntegration::ClosedForm => {
            position + velocity * step_seconds + 0.5 * config.gravity * step_seconds.powi(2)
        }
    };

    advanced.location = glam_to_vec(&next_position);
    advanced.linear_velocity = Some(glam_to_vec(&next_velocity));
    advanced.rotation = apply_velocities_to_rigid_body(&advanced, step_seconds).rotation;
    advanced
}

fn advance_ball_with_plane_bounces_step(
    current: &boxcars::RigidBody,
    step_seconds: f32,
    trajectory_config: BallTrajectoryConfig,
    bounce_config: BallBounceConfig,
    planes: &[BallCollisionPlane],
) -> boxcars::RigidBody {
    ball_with_plane_bounces_step_segments(
        current,
        step_seconds,
        trajectory_config,
        bounce_config,
        planes,
    )
    .last()
    .map(|segment| segment.end)
    .unwrap_or(*current)
}

fn advance_ball_with_surface_bounces_step(
    current: &boxcars::RigidBody,
    step_seconds: f32,
    trajectory_config: BallTrajectoryConfig,
    bounce_config: BallBounceConfig,
    surfaces: &[BallCollisionSurface],
) -> boxcars::RigidBody {
    ball_with_surface_bounces_step_segments(
        current,
        step_seconds,
        trajectory_config,
        bounce_config,
        surfaces,
    )
    .last()
    .map(|segment| segment.end)
    .unwrap_or(*current)
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct BallTrajectorySegment {
    duration: f32,
    start: boxcars::RigidBody,
    end: boxcars::RigidBody,
}

fn ball_with_plane_bounces_step_segments(
    current: &boxcars::RigidBody,
    step_seconds: f32,
    trajectory_config: BallTrajectoryConfig,
    bounce_config: BallBounceConfig,
    planes: &[BallCollisionPlane],
) -> Vec<BallTrajectorySegment> {
    let mut current = *current;
    let mut remaining = step_seconds;
    let mut collisions = 0usize;
    let mut segments = Vec::new();

    while remaining > f32::EPSILON && collisions <= MAX_COLLISIONS_PER_STEP {
        let free_flight_next =
            advance_ball_free_flight_step(&current, remaining, trajectory_config);
        let Some(impact) =
            first_plane_impact(&current, &free_flight_next, bounce_config.radius, planes)
        else {
            segments.push(BallTrajectorySegment {
                duration: remaining,
                start: current,
                end: free_flight_next,
            });
            return segments;
        };

        let impact_time = remaining * impact.fraction;
        let mut impact_body =
            advance_ball_free_flight_step(&current, impact_time, trajectory_config);
        snap_ball_to_plane(&mut impact_body, impact.plane, bounce_config.radius);
        let bounced = bounce_ball_off_surface(
            &impact_body,
            impact.plane.normal,
            bounce_config,
            trajectory_config,
        );

        if impact_time <= COLLISION_TIME_EPSILON && bounced == impact_body {
            let resolved = resolve_ball_plane_collisions(
                &free_flight_next,
                bounce_config,
                trajectory_config,
                planes,
            );
            segments.push(BallTrajectorySegment {
                duration: remaining,
                start: current,
                end: resolved,
            });
            return segments;
        }

        segments.push(BallTrajectorySegment {
            duration: impact_time,
            start: current,
            end: impact_body,
        });
        current = bounced;
        remaining -= impact_time;
        collisions += 1;

        if impact_time <= COLLISION_TIME_EPSILON {
            remaining = (remaining - COLLISION_TIME_EPSILON).max(0.0);
        }
    }

    segments
}

fn ball_with_surface_bounces_step_segments(
    current: &boxcars::RigidBody,
    step_seconds: f32,
    trajectory_config: BallTrajectoryConfig,
    bounce_config: BallBounceConfig,
    surfaces: &[BallCollisionSurface],
) -> Vec<BallTrajectorySegment> {
    let mut current = *current;
    let mut remaining = step_seconds;
    let mut collisions = 0usize;
    let mut segments = Vec::new();

    while remaining > f32::EPSILON && collisions <= MAX_COLLISIONS_PER_STEP {
        let free_flight_next =
            advance_ball_free_flight_step(&current, remaining, trajectory_config);
        let Some(impact) =
            first_surface_impact(&current, &free_flight_next, bounce_config.radius, surfaces)
        else {
            segments.push(BallTrajectorySegment {
                duration: remaining,
                start: current,
                end: free_flight_next,
            });
            return segments;
        };

        let impact_time = remaining * impact.fraction;
        let mut impact_body =
            advance_ball_free_flight_step(&current, impact_time, trajectory_config);
        snap_ball_to_surface(
            &mut impact_body,
            impact.surface,
            impact.normal,
            bounce_config.radius,
        );
        let bounced = bounce_ball_off_surface(
            &impact_body,
            impact.normal,
            bounce_config,
            trajectory_config,
        );

        if impact_time <= COLLISION_TIME_EPSILON && bounced == impact_body {
            let resolved = resolve_ball_surface_collisions(
                &free_flight_next,
                bounce_config,
                trajectory_config,
                surfaces,
            );
            segments.push(BallTrajectorySegment {
                duration: remaining,
                start: current,
                end: resolved,
            });
            return segments;
        }

        segments.push(BallTrajectorySegment {
            duration: impact_time,
            start: current,
            end: impact_body,
        });
        current = bounced;
        remaining -= impact_time;
        collisions += 1;

        if impact_time <= COLLISION_TIME_EPSILON {
            remaining = (remaining - COLLISION_TIME_EPSILON).max(0.0);
        }
    }

    segments
}

pub fn bounce_ball_off_surface(
    rigid_body: &boxcars::RigidBody,
    surface_normal: glam::Vec3,
    bounce_config: BallBounceConfig,
    trajectory_config: BallTrajectoryConfig,
) -> boxcars::RigidBody {
    if !surface_normal.is_finite() || surface_normal.length_squared() <= f32::EPSILON {
        return *rigid_body;
    }

    let normal = surface_normal.normalize();
    let velocity = rigid_body
        .linear_velocity
        .as_ref()
        .map(vec_to_glam)
        .unwrap_or(glam::Vec3::ZERO);
    if velocity.dot(normal) >= 0.0 {
        return *rigid_body;
    }

    let angular_velocity = rigid_body
        .angular_velocity
        .as_ref()
        .map(vec_to_glam)
        .unwrap_or(glam::Vec3::ZERO);
    let perpendicular_velocity = velocity.dot(normal) * normal;
    let parallel_velocity = velocity - perpendicular_velocity;
    if perpendicular_velocity.length() <= RESTING_CONTACT_NORMAL_SPEED_THRESHOLD {
        let mut rested = *rigid_body;
        rested.linear_velocity = Some(glam_to_vec(&parallel_velocity));
        return rested;
    }

    let spin_velocity = bounce_config.radius * normal.cross(angular_velocity);
    let slip_velocity = parallel_velocity + spin_velocity;

    let delta_perpendicular_velocity = -(1.0 + bounce_config.restitution) * perpendicular_velocity;
    let delta_parallel_velocity = if slip_velocity.length_squared() <= f32::EPSILON {
        glam::Vec3::ZERO
    } else {
        let ratio = perpendicular_velocity.length() / slip_velocity.length();
        let impulse_fraction = 1.0f32.min(bounce_config.tangential_ratio_scale * ratio);
        -impulse_fraction * bounce_config.tangential_friction * slip_velocity
    };

    let next_velocity = clamp_speed(
        velocity + delta_perpendicular_velocity + delta_parallel_velocity,
        trajectory_config.max_speed,
    );
    let next_angular_velocity = angular_velocity
        + bounce_config.angular_coupling
            * bounce_config.radius
            * delta_parallel_velocity.cross(normal);

    let mut bounced = *rigid_body;
    bounced.linear_velocity = Some(glam_to_vec(&next_velocity));
    bounced.angular_velocity = Some(glam_to_vec(&next_angular_velocity));
    bounced
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PlaneImpact {
    plane: BallCollisionPlane,
    fraction: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SurfaceImpact {
    surface: BallCollisionSurface,
    normal: glam::Vec3,
    fraction: f32,
}

fn first_plane_impact(
    start: &boxcars::RigidBody,
    end: &boxcars::RigidBody,
    radius: f32,
    planes: &[BallCollisionPlane],
) -> Option<PlaneImpact> {
    let start_position = vec_to_glam(&start.location);
    let end_position = vec_to_glam(&end.location);
    let displacement = end_position - start_position;
    let mut first_impact: Option<PlaneImpact> = None;

    for &plane in planes {
        let movement_toward_plane = displacement.dot(plane.normal);
        if movement_toward_plane >= -f32::EPSILON {
            continue;
        }

        let start_distance = plane.center_distance(start_position);
        let end_distance = plane.center_distance(end_position);
        if start_distance < radius - f32::EPSILON {
            if plane.contains_impact_point(start_position) {
                return Some(PlaneImpact {
                    plane,
                    fraction: 0.0,
                });
            }
            continue;
        }
        if end_distance >= radius {
            continue;
        }

        let distance_delta = end_distance - start_distance;
        if distance_delta >= -f32::EPSILON {
            continue;
        }

        let fraction = ((radius - start_distance) / distance_delta).clamp(0.0, 1.0);
        let impact_position = start_position + displacement * fraction;
        if !plane.contains_impact_point(impact_position) {
            continue;
        }
        if first_impact.is_none_or(|impact| fraction < impact.fraction) {
            first_impact = Some(PlaneImpact { plane, fraction });
        }
    }

    first_impact
}

fn first_surface_impact(
    start: &boxcars::RigidBody,
    end: &boxcars::RigidBody,
    radius: f32,
    surfaces: &[BallCollisionSurface],
) -> Option<SurfaceImpact> {
    let mut first_impact: Option<SurfaceImpact> = None;

    for &surface in surfaces {
        let impact = match surface {
            BallCollisionSurface::Plane(plane) => {
                plane_impact(start, end, radius, plane).map(|impact| SurfaceImpact {
                    surface,
                    normal: plane.normal,
                    fraction: impact.fraction,
                })
            }
            BallCollisionSurface::Cylinder(cylinder) => {
                cylinder_impact(start, end, radius, cylinder).map(|(fraction, normal)| {
                    SurfaceImpact {
                        surface,
                        normal,
                        fraction,
                    }
                })
            }
            BallCollisionSurface::ConcaveCylinder(cylinder) => {
                concave_cylinder_impact(start, end, radius, cylinder).map(|(fraction, normal)| {
                    SurfaceImpact {
                        surface,
                        normal,
                        fraction,
                    }
                })
            }
        };

        if let Some(impact) = impact
            && first_impact.is_none_or(|first| impact.fraction < first.fraction)
        {
            first_impact = Some(impact);
        }
    }

    first_impact
}

fn plane_impact(
    start: &boxcars::RigidBody,
    end: &boxcars::RigidBody,
    radius: f32,
    plane: BallCollisionPlane,
) -> Option<PlaneImpact> {
    let start_position = vec_to_glam(&start.location);
    let end_position = vec_to_glam(&end.location);
    let displacement = end_position - start_position;
    let movement_toward_plane = displacement.dot(plane.normal);
    if movement_toward_plane >= -f32::EPSILON {
        return None;
    }

    let start_distance = plane.center_distance(start_position);
    let end_distance = plane.center_distance(end_position);
    if start_distance < radius - f32::EPSILON {
        return plane
            .contains_impact_point(start_position)
            .then_some(PlaneImpact {
                plane,
                fraction: 0.0,
            });
    }
    if end_distance >= radius {
        return None;
    }

    let distance_delta = end_distance - start_distance;
    if distance_delta >= -f32::EPSILON {
        return None;
    }

    let fraction = ((radius - start_distance) / distance_delta).clamp(0.0, 1.0);
    let impact_position = start_position + displacement * fraction;
    plane
        .contains_impact_point(impact_position)
        .then_some(PlaneImpact { plane, fraction })
}

fn cylinder_impact(
    start: &boxcars::RigidBody,
    end: &boxcars::RigidBody,
    ball_radius: f32,
    cylinder: BallCollisionCylinder,
) -> Option<(f32, glam::Vec3)> {
    if cylinder.radius <= 0.0 {
        return None;
    }

    let start_position = vec_to_glam(&start.location);
    let end_position = vec_to_glam(&end.location);
    let start_perp = cylinder_perpendicular(cylinder.axis, start_position);
    let end_perp = cylinder_perpendicular(cylinder.axis, end_position);
    let center_perp = cylinder_perpendicular(cylinder.axis, cylinder.center);
    let displacement = end_perp - start_perp;
    let from_center = start_perp - center_perp;
    let contact_radius = cylinder.radius + ball_radius;
    let start_axis = cylinder_axis_value(cylinder.axis, start_position);
    let end_axis = cylinder_axis_value(cylinder.axis, end_position);
    let start_distance_sq = from_center.length_squared();

    if start_distance_sq < contact_radius.powi(2) - f32::EPSILON {
        return cylinder_axis_value_is_in_bounds(cylinder, start_axis, ball_radius).then(|| {
            let normal = cylinder_normal(cylinder.axis, start_perp, center_perp, displacement);
            (0.0, normal)
        });
    }

    let a = displacement.dot(displacement);
    if a <= f32::EPSILON {
        return None;
    }

    let b = 2.0 * from_center.dot(displacement);
    if b >= 0.0 {
        return None;
    }

    let c = start_distance_sq - contact_radius.powi(2);
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }

    let fraction = (-b - discriminant.sqrt()) / (2.0 * a);
    if !(-f32::EPSILON..=1.0 + f32::EPSILON).contains(&fraction) {
        return None;
    }
    let fraction = fraction.clamp(0.0, 1.0);
    let impact_axis = start_axis + (end_axis - start_axis) * fraction;
    if !cylinder_axis_value_is_in_bounds(cylinder, impact_axis, ball_radius) {
        return None;
    }

    let impact_perp = start_perp + displacement * fraction;
    let normal = cylinder_normal(cylinder.axis, impact_perp, center_perp, displacement);
    Some((fraction, normal))
}

fn concave_cylinder_impact(
    start: &boxcars::RigidBody,
    end: &boxcars::RigidBody,
    ball_radius: f32,
    cylinder: BallCollisionConcaveCylinder,
) -> Option<(f32, glam::Vec3)> {
    let contact_radius = cylinder.radius - ball_radius;
    if contact_radius <= 0.0 {
        return None;
    }

    let start_position = vec_to_glam(&start.location);
    let end_position = vec_to_glam(&end.location);
    let start_perp = cylinder_perpendicular(cylinder.axis, start_position);
    let end_perp = cylinder_perpendicular(cylinder.axis, end_position);
    let center_perp = cylinder_perpendicular(cylinder.axis, cylinder.center);
    let displacement = end_perp - start_perp;
    let from_center = start_perp - center_perp;
    let start_axis = cylinder_axis_value(cylinder.axis, start_position);
    let end_axis = cylinder_axis_value(cylinder.axis, end_position);
    let start_distance_sq = from_center.length_squared();

    if start_distance_sq > contact_radius.powi(2) + f32::EPSILON {
        return concave_cylinder_contains_center_position(cylinder, start_position, ball_radius)
            .then(|| {
                let normal =
                    concave_cylinder_normal(cylinder.axis, start_perp, center_perp, displacement);
                (0.0, normal)
            });
    }

    let a = displacement.dot(displacement);
    if a <= f32::EPSILON {
        return None;
    }

    let b = 2.0 * from_center.dot(displacement);
    if b <= 0.0 {
        return None;
    }

    let c = start_distance_sq - contact_radius.powi(2);
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }

    let fraction = (-b + discriminant.sqrt()) / (2.0 * a);
    if !(-f32::EPSILON..=1.0 + f32::EPSILON).contains(&fraction) {
        return None;
    }
    let fraction = fraction.clamp(0.0, 1.0);
    let impact_axis = start_axis + (end_axis - start_axis) * fraction;
    if !concave_cylinder_axis_value_is_in_bounds(cylinder, impact_axis, ball_radius) {
        return None;
    }

    let impact_position = vec_to_glam(&start.location) + (end_position - start_position) * fraction;
    if !concave_cylinder_contains_center_position(cylinder, impact_position, ball_radius) {
        return None;
    }

    let impact_perp = start_perp + displacement * fraction;
    let normal = concave_cylinder_normal(cylinder.axis, impact_perp, center_perp, displacement);
    Some((fraction, normal))
}

fn cylinder_perpendicular(axis: BallCollisionCylinderAxis, position: glam::Vec3) -> glam::Vec2 {
    match axis {
        BallCollisionCylinderAxis::X => glam::Vec2::new(position.y, position.z),
        BallCollisionCylinderAxis::Y => glam::Vec2::new(position.x, position.z),
        BallCollisionCylinderAxis::Z => glam::Vec2::new(position.x, position.y),
    }
}

fn cylinder_axis_value(axis: BallCollisionCylinderAxis, position: glam::Vec3) -> f32 {
    match axis {
        BallCollisionCylinderAxis::X => position.x,
        BallCollisionCylinderAxis::Y => position.y,
        BallCollisionCylinderAxis::Z => position.z,
    }
}

fn cylinder_axis_value_is_in_bounds(
    cylinder: BallCollisionCylinder,
    axis_value: f32,
    ball_radius: f32,
) -> bool {
    axis_value + ball_radius + ARENA_BOUND_EPSILON >= cylinder.min_axis
        && axis_value - ball_radius - ARENA_BOUND_EPSILON <= cylinder.max_axis
}

fn concave_cylinder_axis_value_is_in_bounds(
    cylinder: BallCollisionConcaveCylinder,
    axis_value: f32,
    ball_radius: f32,
) -> bool {
    axis_value + ball_radius + ARENA_BOUND_EPSILON >= cylinder.min_axis
        && axis_value - ball_radius - ARENA_BOUND_EPSILON <= cylinder.max_axis
}

fn concave_cylinder_contains_center_position(
    cylinder: BallCollisionConcaveCylinder,
    position: glam::Vec3,
    ball_radius: f32,
) -> bool {
    concave_cylinder_axis_value_is_in_bounds(
        cylinder,
        cylinder_axis_value(cylinder.axis, position),
        ball_radius,
    ) && cylinder.bounds.contains(position)
}

fn cylinder_normal(
    axis: BallCollisionCylinderAxis,
    impact_perp: glam::Vec2,
    center_perp: glam::Vec2,
    displacement: glam::Vec2,
) -> glam::Vec3 {
    let normal_perp = (impact_perp - center_perp).normalize_or_zero();
    let normal_perp = if normal_perp.length_squared() <= f32::EPSILON {
        -displacement.normalize_or_zero()
    } else {
        normal_perp
    };

    match axis {
        BallCollisionCylinderAxis::X => glam::Vec3::new(0.0, normal_perp.x, normal_perp.y),
        BallCollisionCylinderAxis::Y => glam::Vec3::new(normal_perp.x, 0.0, normal_perp.y),
        BallCollisionCylinderAxis::Z => glam::Vec3::new(normal_perp.x, normal_perp.y, 0.0),
    }
    .normalize_or_zero()
}

fn concave_cylinder_normal(
    axis: BallCollisionCylinderAxis,
    impact_perp: glam::Vec2,
    center_perp: glam::Vec2,
    displacement: glam::Vec2,
) -> glam::Vec3 {
    let normal_perp = (center_perp - impact_perp).normalize_or_zero();
    let normal_perp = if normal_perp.length_squared() <= f32::EPSILON {
        -displacement.normalize_or_zero()
    } else {
        normal_perp
    };

    match axis {
        BallCollisionCylinderAxis::X => glam::Vec3::new(0.0, normal_perp.x, normal_perp.y),
        BallCollisionCylinderAxis::Y => glam::Vec3::new(normal_perp.x, 0.0, normal_perp.y),
        BallCollisionCylinderAxis::Z => glam::Vec3::new(normal_perp.x, normal_perp.y, 0.0),
    }
    .normalize_or_zero()
}

fn snap_ball_to_plane(rigid_body: &mut boxcars::RigidBody, plane: BallCollisionPlane, radius: f32) {
    let position = vec_to_glam(&rigid_body.location);
    let center_distance = plane.center_distance(position);
    rigid_body.location = glam_to_vec(&(position + plane.normal * (radius - center_distance)));
}

fn snap_ball_to_surface(
    rigid_body: &mut boxcars::RigidBody,
    surface: BallCollisionSurface,
    normal: glam::Vec3,
    ball_radius: f32,
) {
    match surface {
        BallCollisionSurface::Plane(plane) => snap_ball_to_plane(rigid_body, plane, ball_radius),
        BallCollisionSurface::Cylinder(cylinder) => {
            let position = vec_to_glam(&rigid_body.location);
            let center_perp = cylinder_perpendicular(cylinder.axis, cylinder.center);
            let current_perp = cylinder_perpendicular(cylinder.axis, position);
            let normal_perp = cylinder_perpendicular(cylinder.axis, normal).normalize_or_zero();
            let normal_perp = if normal_perp.length_squared() <= f32::EPSILON {
                (current_perp - center_perp).normalize_or_zero()
            } else {
                normal_perp
            };
            let snapped_perp = center_perp + normal_perp * (ball_radius + cylinder.radius);
            let snapped_position =
                with_cylinder_perpendicular(cylinder.axis, position, snapped_perp);
            rigid_body.location = glam_to_vec(&snapped_position);
        }
        BallCollisionSurface::ConcaveCylinder(cylinder) => {
            let position = vec_to_glam(&rigid_body.location);
            let center_perp = cylinder_perpendicular(cylinder.axis, cylinder.center);
            let current_perp = cylinder_perpendicular(cylinder.axis, position);
            let normal_perp = cylinder_perpendicular(cylinder.axis, normal).normalize_or_zero();
            let normal_perp = if normal_perp.length_squared() <= f32::EPSILON {
                (center_perp - current_perp).normalize_or_zero()
            } else {
                normal_perp
            };
            let contact_radius = (cylinder.radius - ball_radius).max(0.0);
            let snapped_perp = center_perp - normal_perp * contact_radius;
            let snapped_position =
                with_cylinder_perpendicular(cylinder.axis, position, snapped_perp);
            rigid_body.location = glam_to_vec(&snapped_position);
        }
    }
}

fn with_cylinder_perpendicular(
    axis: BallCollisionCylinderAxis,
    position: glam::Vec3,
    perpendicular: glam::Vec2,
) -> glam::Vec3 {
    match axis {
        BallCollisionCylinderAxis::X => {
            glam::Vec3::new(position.x, perpendicular.x, perpendicular.y)
        }
        BallCollisionCylinderAxis::Y => {
            glam::Vec3::new(perpendicular.x, position.y, perpendicular.y)
        }
        BallCollisionCylinderAxis::Z => {
            glam::Vec3::new(perpendicular.x, perpendicular.y, position.z)
        }
    }
}

fn resolve_ball_plane_collisions(
    rigid_body: &boxcars::RigidBody,
    bounce_config: BallBounceConfig,
    trajectory_config: BallTrajectoryConfig,
    planes: &[BallCollisionPlane],
) -> boxcars::RigidBody {
    let mut resolved = *rigid_body;
    for plane in planes {
        let position = vec_to_glam(&resolved.location);
        let penetration_depth = plane.penetration_depth(position, bounce_config.radius);
        if penetration_depth <= 0.0 || !plane.contains_impact_point(position) {
            continue;
        }

        snap_ball_to_plane(&mut resolved, *plane, bounce_config.radius);
        resolved =
            bounce_ball_off_surface(&resolved, plane.normal, bounce_config, trajectory_config);
    }

    resolved
}

fn resolve_ball_surface_collisions(
    rigid_body: &boxcars::RigidBody,
    bounce_config: BallBounceConfig,
    trajectory_config: BallTrajectoryConfig,
    surfaces: &[BallCollisionSurface],
) -> boxcars::RigidBody {
    let mut resolved = *rigid_body;
    for surface in surfaces {
        let Some(normal) = surface_penetration_normal(&resolved, bounce_config.radius, *surface)
        else {
            continue;
        };

        snap_ball_to_surface(&mut resolved, *surface, normal, bounce_config.radius);
        resolved = bounce_ball_off_surface(&resolved, normal, bounce_config, trajectory_config);
    }

    resolved
}

fn surface_penetration_normal(
    rigid_body: &boxcars::RigidBody,
    ball_radius: f32,
    surface: BallCollisionSurface,
) -> Option<glam::Vec3> {
    let position = vec_to_glam(&rigid_body.location);
    match surface {
        BallCollisionSurface::Plane(plane) => {
            let penetration_depth = plane.penetration_depth(position, ball_radius);
            (penetration_depth > 0.0 && plane.contains_impact_point(position))
                .then_some(plane.normal)
        }
        BallCollisionSurface::Cylinder(cylinder) => {
            let axis_value = cylinder_axis_value(cylinder.axis, position);
            if !cylinder_axis_value_is_in_bounds(cylinder, axis_value, ball_radius) {
                return None;
            }
            let center_perp = cylinder_perpendicular(cylinder.axis, cylinder.center);
            let current_perp = cylinder_perpendicular(cylinder.axis, position);
            let offset = current_perp - center_perp;
            let expanded_radius = ball_radius + cylinder.radius;
            if offset.length_squared() >= expanded_radius.powi(2) {
                return None;
            }
            Some(cylinder_normal(
                cylinder.axis,
                current_perp,
                center_perp,
                glam::Vec2::ZERO,
            ))
        }
        BallCollisionSurface::ConcaveCylinder(cylinder) => {
            if !concave_cylinder_contains_center_position(cylinder, position, ball_radius) {
                return None;
            }
            let contact_radius = cylinder.radius - ball_radius;
            if contact_radius <= 0.0 {
                return None;
            }
            let center_perp = cylinder_perpendicular(cylinder.axis, cylinder.center);
            let current_perp = cylinder_perpendicular(cylinder.axis, position);
            let offset = current_perp - center_perp;
            if offset.length_squared() <= contact_radius.powi(2) {
                return None;
            }
            Some(concave_cylinder_normal(
                cylinder.axis,
                current_perp,
                center_perp,
                glam::Vec2::ZERO,
            ))
        }
    }
}

fn clamp_speed(velocity: glam::Vec3, max_speed: f32) -> glam::Vec3 {
    if !max_speed.is_finite() || max_speed <= 0.0 {
        return velocity;
    }
    let speed = velocity.length();
    if speed > max_speed {
        velocity * (max_speed / speed)
    } else {
        velocity
    }
}

/// Produces regularly sampled free-flight predictions, including the initial
/// sample at `time == 0.0` and the exact requested endpoint.
pub fn predict_ball_free_flight_trajectory(
    initial: &boxcars::RigidBody,
    duration_seconds: f32,
    sample_interval_seconds: f32,
    config: BallTrajectoryConfig,
) -> Vec<BallTrajectorySample> {
    if duration_seconds < 0.0 || sample_interval_seconds <= 0.0 {
        return Vec::new();
    }

    let mut samples = vec![BallTrajectorySample {
        time: 0.0,
        rigid_body: *initial,
    }];
    if duration_seconds == 0.0 {
        return samples;
    }

    let mut elapsed = 0.0;
    let mut current = *initial;
    while elapsed < duration_seconds {
        let step = (duration_seconds - elapsed).min(sample_interval_seconds);
        current = advance_ball_free_flight(&current, step, config);
        elapsed += step;
        samples.push(BallTrajectorySample {
            time: elapsed.min(duration_seconds),
            rigid_body: current,
        });
    }

    samples
}

pub fn predict_ball_with_plane_bounces_trajectory(
    initial: &boxcars::RigidBody,
    duration_seconds: f32,
    sample_interval_seconds: f32,
    trajectory_config: BallTrajectoryConfig,
    bounce_config: BallBounceConfig,
    planes: &[BallCollisionPlane],
) -> Vec<BallTrajectorySample> {
    if duration_seconds < 0.0 || sample_interval_seconds <= 0.0 {
        return Vec::new();
    }

    let mut samples = vec![BallTrajectorySample {
        time: 0.0,
        rigid_body: *initial,
    }];
    if duration_seconds == 0.0 {
        return samples;
    }

    let mut elapsed = 0.0;
    let mut current = *initial;
    while elapsed < duration_seconds {
        let step = (duration_seconds - elapsed).min(sample_interval_seconds);
        current = advance_ball_with_plane_bounces(
            &current,
            step,
            trajectory_config,
            bounce_config,
            planes,
        );
        elapsed += step;
        samples.push(BallTrajectorySample {
            time: elapsed.min(duration_seconds),
            rigid_body: current,
        });
    }

    samples
}

pub fn predict_ball_with_surface_bounces_trajectory(
    initial: &boxcars::RigidBody,
    duration_seconds: f32,
    sample_interval_seconds: f32,
    trajectory_config: BallTrajectoryConfig,
    bounce_config: BallBounceConfig,
    surfaces: &[BallCollisionSurface],
) -> Vec<BallTrajectorySample> {
    if duration_seconds < 0.0 || sample_interval_seconds <= 0.0 {
        return Vec::new();
    }

    let mut samples = vec![BallTrajectorySample {
        time: 0.0,
        rigid_body: *initial,
    }];
    if duration_seconds == 0.0 {
        return samples;
    }

    let mut elapsed = 0.0;
    let mut current = *initial;
    while elapsed < duration_seconds {
        let step = (duration_seconds - elapsed).min(sample_interval_seconds);
        current = advance_ball_with_surface_bounces(
            &current,
            step,
            trajectory_config,
            bounce_config,
            surfaces,
        );
        elapsed += step;
        samples.push(BallTrajectorySample {
            time: elapsed.min(duration_seconds),
            rigid_body: current,
        });
    }

    samples
}

/// Predicts where the ball center crosses a goal line under free-flight physics.
///
/// This ignores later touches and arena collisions. It is intended for the common
/// saved-shot question: "where was this shot projected to cross the goal line
/// before a defender intervened?"
pub fn predict_free_flight_goal_line_crossing(
    initial: &boxcars::RigidBody,
    crossing_config: BallGoalLineCrossingConfig,
    trajectory_config: BallTrajectoryConfig,
) -> Option<BallGoalLineCrossing> {
    if initial.linear_velocity.is_none()
        || crossing_config.max_seconds < 0.0
        || crossing_config.target_goal_y.abs() <= f32::EPSILON
    {
        return None;
    }

    let direction = crossing_config.target_goal_y.signum();
    let fixed_step_seconds = trajectory_config.fixed_step_seconds();
    let mut current = *initial;
    let mut elapsed = 0.0f32;
    let mut steps = 0usize;

    while elapsed < crossing_config.max_seconds && steps < MAX_INTEGRATION_STEPS {
        let step_seconds = (crossing_config.max_seconds - elapsed).min(fixed_step_seconds);
        let next = advance_ball_free_flight_step(&current, step_seconds, trajectory_config);
        if let Some(crossing) = goal_line_crossing_between(
            elapsed,
            step_seconds,
            &current,
            &next,
            direction,
            crossing_config,
        ) {
            return Some(crossing);
        }

        current = next;
        elapsed += step_seconds;
        steps += 1;
    }

    None
}

/// Predicts where the ball center crosses a goal line while resolving bounces
/// against caller-provided planes.
pub fn predict_ball_with_plane_bounces_goal_line_crossing(
    initial: &boxcars::RigidBody,
    crossing_config: BallGoalLineCrossingConfig,
    trajectory_config: BallTrajectoryConfig,
    bounce_config: BallBounceConfig,
    planes: &[BallCollisionPlane],
) -> Option<BallGoalLineCrossing> {
    if initial.linear_velocity.is_none()
        || crossing_config.max_seconds < 0.0
        || crossing_config.target_goal_y.abs() <= f32::EPSILON
    {
        return None;
    }

    let direction = crossing_config.target_goal_y.signum();
    let fixed_step_seconds = trajectory_config.fixed_step_seconds();
    let mut current = *initial;
    let mut elapsed = 0.0f32;
    let mut steps = 0usize;

    while elapsed < crossing_config.max_seconds && steps < MAX_INTEGRATION_STEPS {
        let step_seconds = (crossing_config.max_seconds - elapsed).min(fixed_step_seconds);
        let segments = ball_with_plane_bounces_step_segments(
            &current,
            step_seconds,
            trajectory_config,
            bounce_config,
            planes,
        );
        let mut segment_start_time = elapsed;
        for segment in &segments {
            if let Some(crossing) = goal_line_crossing_between(
                segment_start_time,
                segment.duration,
                &segment.start,
                &segment.end,
                direction,
                crossing_config,
            ) {
                return Some(crossing);
            }
            segment_start_time += segment.duration;
        }

        current = segments
            .last()
            .map(|segment| segment.end)
            .unwrap_or(current);
        elapsed += step_seconds;
        steps += 1;
    }

    None
}

pub fn predict_ball_with_surface_bounces_goal_line_crossing(
    initial: &boxcars::RigidBody,
    crossing_config: BallGoalLineCrossingConfig,
    trajectory_config: BallTrajectoryConfig,
    bounce_config: BallBounceConfig,
    surfaces: &[BallCollisionSurface],
) -> Option<BallGoalLineCrossing> {
    if initial.linear_velocity.is_none()
        || crossing_config.max_seconds < 0.0
        || crossing_config.target_goal_y.abs() <= f32::EPSILON
    {
        return None;
    }

    let direction = crossing_config.target_goal_y.signum();
    let fixed_step_seconds = trajectory_config.fixed_step_seconds();
    let mut current = *initial;
    let mut elapsed = 0.0f32;
    let mut steps = 0usize;

    while elapsed < crossing_config.max_seconds && steps < MAX_INTEGRATION_STEPS {
        let step_seconds = (crossing_config.max_seconds - elapsed).min(fixed_step_seconds);
        let segments = ball_with_surface_bounces_step_segments(
            &current,
            step_seconds,
            trajectory_config,
            bounce_config,
            surfaces,
        );
        let mut segment_start_time = elapsed;
        for segment in &segments {
            if let Some(crossing) = goal_line_crossing_between(
                segment_start_time,
                segment.duration,
                &segment.start,
                &segment.end,
                direction,
                crossing_config,
            ) {
                return Some(crossing);
            }
            segment_start_time += segment.duration;
        }

        current = segments
            .last()
            .map(|segment| segment.end)
            .unwrap_or(current);
        elapsed += step_seconds;
        steps += 1;
    }

    None
}

pub fn predict_ball_with_surface_bounces_goal_target_hit(
    initial: &boxcars::RigidBody,
    crossing_config: BallGoalLineCrossingConfig,
    trajectory_config: BallTrajectoryConfig,
    bounce_config: BallBounceConfig,
    surfaces: &[BallCollisionSurface],
) -> Option<BallGoalTargetHit> {
    if initial.linear_velocity.is_none()
        || crossing_config.max_seconds < 0.0
        || crossing_config.target_goal_y.abs() <= f32::EPSILON
    {
        return None;
    }

    let direction = crossing_config.target_goal_y.signum();
    let fixed_step_seconds = trajectory_config.fixed_step_seconds();
    let mut current = *initial;
    let mut elapsed = 0.0f32;
    let mut steps = 0usize;

    while elapsed < crossing_config.max_seconds && steps < MAX_INTEGRATION_STEPS {
        let step_seconds = (crossing_config.max_seconds - elapsed).min(fixed_step_seconds);
        let free_flight_next =
            advance_ball_free_flight_step(&current, step_seconds, trajectory_config);
        let impact =
            first_surface_impact(&current, &free_flight_next, bounce_config.radius, surfaces);

        let (segment_end, segment_duration) = if let Some(impact) = impact {
            let impact_time = step_seconds * impact.fraction;
            let mut impact_body =
                advance_ball_free_flight_step(&current, impact_time, trajectory_config);
            snap_ball_to_surface(
                &mut impact_body,
                impact.surface,
                impact.normal,
                bounce_config.radius,
            );
            if let Some(hit_kind) = goal_target_surface_hit_kind(impact.surface, crossing_config) {
                return Some(BallGoalTargetHit {
                    time: elapsed + impact_time,
                    position: goal_target_surface_contact_position(
                        impact_body,
                        impact.normal,
                        bounce_config.radius,
                    ),
                    velocity: impact_body.linear_velocity.as_ref().map(vec_to_glam),
                    hit_kind,
                });
            }
            (impact_body, impact_time)
        } else {
            (free_flight_next, step_seconds)
        };

        if let Some(crossing) = goal_line_crossing_between(
            elapsed,
            segment_duration,
            &current,
            &segment_end,
            direction,
            crossing_config,
        ) && crossing.inside_goal_mouth
        {
            return Some(BallGoalTargetHit {
                time: crossing.time,
                position: crossing.position,
                velocity: crossing.velocity,
                hit_kind: BallGoalTargetHitKind::GoalLine,
            });
        }

        if let Some(impact) = impact {
            let bounced = bounce_ball_off_surface(
                &segment_end,
                impact.normal,
                bounce_config,
                trajectory_config,
            );
            current = if segment_duration <= COLLISION_TIME_EPSILON && bounced == segment_end {
                resolve_ball_surface_collisions(
                    &free_flight_next,
                    bounce_config,
                    trajectory_config,
                    surfaces,
                )
            } else {
                bounced
            };
        } else {
            current = free_flight_next;
        }
        elapsed += step_seconds;
        steps += 1;
    }

    None
}

fn goal_target_surface_hit_kind(
    surface: BallCollisionSurface,
    crossing_config: BallGoalLineCrossingConfig,
) -> Option<BallGoalTargetHitKind> {
    match surface {
        BallCollisionSurface::Plane(plane) => {
            let plane_y = (plane.normal.y.abs() > f32::EPSILON)
                .then_some(plane.distance_from_origin / plane.normal.y)?;
            ((plane_y - crossing_config.target_goal_y).abs() <= ARENA_BOUND_EPSILON)
                .then_some(BallGoalTargetHitKind::BackWall)
        }
        BallCollisionSurface::Cylinder(cylinder) => {
            ((cylinder.center.y - crossing_config.target_goal_y).abs() <= ARENA_BOUND_EPSILON)
                .then_some(BallGoalTargetHitKind::GoalFrame)
        }
        BallCollisionSurface::ConcaveCylinder(_) => None,
    }
}

fn goal_target_surface_contact_position(
    impact_body: boxcars::RigidBody,
    impact_normal: glam::Vec3,
    ball_radius: f32,
) -> glam::Vec3 {
    let position = vec_to_glam(&impact_body.location);
    if impact_normal.length_squared() <= f32::EPSILON {
        position
    } else {
        position - impact_normal.normalize() * ball_radius
    }
}

fn goal_line_crossing_between(
    start_time: f32,
    step_seconds: f32,
    start: &boxcars::RigidBody,
    end: &boxcars::RigidBody,
    direction: f32,
    crossing_config: BallGoalLineCrossingConfig,
) -> Option<BallGoalLineCrossing> {
    let start_position = vec_to_glam(&start.location);
    let end_position = vec_to_glam(&end.location);
    let start_signed_distance = direction * (start_position.y - crossing_config.target_goal_y);
    let end_signed_distance = direction * (end_position.y - crossing_config.target_goal_y);
    if start_signed_distance > 0.0 || end_signed_distance < 0.0 {
        return None;
    }

    let delta_y = end_position.y - start_position.y;
    if direction * delta_y <= f32::EPSILON {
        return None;
    }

    let fraction = ((crossing_config.target_goal_y - start_position.y) / delta_y).clamp(0.0, 1.0);
    let position = start_position.lerp(end_position, fraction);
    let velocity = match (
        start.linear_velocity.as_ref().map(vec_to_glam),
        end.linear_velocity.as_ref().map(vec_to_glam),
    ) {
        (Some(start_velocity), Some(end_velocity)) => {
            Some(start_velocity.lerp(end_velocity, fraction))
        }
        (Some(velocity), None) | (None, Some(velocity)) => Some(velocity),
        (None, None) => None,
    };

    Some(BallGoalLineCrossing {
        time: start_time + step_seconds * fraction,
        position: glam::Vec3::new(position.x, crossing_config.target_goal_y, position.z),
        velocity,
        inside_goal_mouth: goal_line_crossing_is_inside_mouth(position, crossing_config),
    })
}

fn goal_line_crossing_is_inside_mouth(
    position: glam::Vec3,
    crossing_config: BallGoalLineCrossingConfig,
) -> bool {
    position.x.abs() <= crossing_config.goal_mouth_half_width_x + crossing_config.goal_mouth_margin
        && position.z >= STANDARD_BALL_RADIUS - crossing_config.goal_mouth_margin
        && position.z <= crossing_config.goal_mouth_height_z + crossing_config.goal_mouth_margin
}

/// Compares observed replay samples to free-flight predictions from the same
/// initial state. Observed times are relative to `initial`.
pub fn ball_free_flight_prediction_error(
    initial: &boxcars::RigidBody,
    observed: &[(f32, boxcars::RigidBody)],
    config: BallTrajectoryConfig,
) -> Option<BallTrajectoryError> {
    ball_prediction_error(initial, observed, |initial, time| {
        advance_ball_free_flight(initial, time, config)
    })
}

/// Compares observed replay samples to predictions that include bounces against
/// caller-provided planes. Observed times are relative to `initial`.
pub fn ball_plane_bounce_prediction_error(
    initial: &boxcars::RigidBody,
    observed: &[(f32, boxcars::RigidBody)],
    trajectory_config: BallTrajectoryConfig,
    bounce_config: BallBounceConfig,
    planes: &[BallCollisionPlane],
) -> Option<BallTrajectoryError> {
    ball_prediction_error(initial, observed, |initial, time| {
        advance_ball_with_plane_bounces(initial, time, trajectory_config, bounce_config, planes)
    })
}

pub fn ball_surface_bounce_prediction_error(
    initial: &boxcars::RigidBody,
    observed: &[(f32, boxcars::RigidBody)],
    trajectory_config: BallTrajectoryConfig,
    bounce_config: BallBounceConfig,
    surfaces: &[BallCollisionSurface],
) -> Option<BallTrajectoryError> {
    ball_prediction_error(initial, observed, |initial, time| {
        advance_ball_with_surface_bounces(initial, time, trajectory_config, bounce_config, surfaces)
    })
}

fn ball_prediction_error(
    initial: &boxcars::RigidBody,
    observed: &[(f32, boxcars::RigidBody)],
    predict: impl Fn(&boxcars::RigidBody, f32) -> boxcars::RigidBody,
) -> Option<BallTrajectoryError> {
    if observed.is_empty() {
        return None;
    }

    let mut max_position_error = 0.0f32;
    let mut position_error_sum_sq = 0.0f32;
    let mut max_velocity_error = 0.0f32;
    let mut velocity_error_sum_sq = 0.0f32;
    let mut velocity_sample_count = 0usize;

    for (time, observed_body) in observed {
        let predicted = predict(initial, *time);
        let position_error =
            vec_to_glam(&predicted.location).distance(vec_to_glam(&observed_body.location));
        max_position_error = max_position_error.max(position_error);
        position_error_sum_sq += position_error.powi(2);

        if let (Some(predicted_velocity), Some(observed_velocity)) = (
            predicted.linear_velocity.as_ref(),
            observed_body.linear_velocity.as_ref(),
        ) {
            let velocity_error =
                vec_to_glam(predicted_velocity).distance(vec_to_glam(observed_velocity));
            max_velocity_error = max_velocity_error.max(velocity_error);
            velocity_error_sum_sq += velocity_error.powi(2);
            velocity_sample_count += 1;
        }
    }

    Some(BallTrajectoryError {
        sample_count: observed.len(),
        max_position_error,
        rms_position_error: (position_error_sum_sq / observed.len() as f32).sqrt(),
        max_velocity_error: (velocity_sample_count > 0).then_some(max_velocity_error),
        rms_velocity_error: (velocity_sample_count > 0)
            .then_some((velocity_error_sum_sq / velocity_sample_count as f32).sqrt()),
    })
}

#[cfg(test)]
#[path = "ballistics_tests.rs"]
mod tests;
