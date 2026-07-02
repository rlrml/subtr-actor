use crate::{SubtrActorError, SubtrActorErrorVariant, SubtrActorResult};

pub fn vec_to_glam(v: &boxcars::Vector3f) -> glam::f32::Vec3 {
    glam::f32::Vec3::new(v.x, v.y, v.z)
}

pub fn glam_to_vec(v: &glam::f32::Vec3) -> boxcars::Vector3f {
    boxcars::Vector3f {
        x: v.x,
        y: v.y,
        z: v.z,
    }
}

pub fn quat_to_glam(q: &boxcars::Quaternion) -> glam::Quat {
    let rotation = glam::Quat::from_xyzw(q.x, q.y, q.z, q.w);
    if rotation.x.is_finite()
        && rotation.y.is_finite()
        && rotation.z.is_finite()
        && rotation.w.is_finite()
        && rotation.length_squared() > 0.0
    {
        rotation.normalize()
    } else {
        glam::Quat::IDENTITY
    }
}

pub fn glam_to_quat(rotation: &glam::Quat) -> boxcars::Quaternion {
    boxcars::Quaternion {
        x: rotation.x,
        y: rotation.y,
        z: rotation.z,
        w: rotation.w,
    }
}

pub fn apply_velocities_to_rigid_body(
    rigid_body: &boxcars::RigidBody,
    time_delta: f32,
) -> boxcars::RigidBody {
    let mut interpolated = *rigid_body;
    if time_delta == 0.0 {
        return interpolated;
    }
    let linear_velocity = interpolated.linear_velocity.unwrap_or(boxcars::Vector3f {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    });
    let location = vec_to_glam(&rigid_body.location) + (time_delta * vec_to_glam(&linear_velocity));
    interpolated.location = glam_to_vec(&location);
    interpolated.rotation = apply_angular_velocity(rigid_body, time_delta);
    interpolated
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarHitboxFamily {
    Breakout,
    Dominus,
    Hybrid,
    Merc,
    Octane,
    Plank,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CarHitbox {
    pub family: CarHitboxFamily,
    pub length: f32,
    pub width: f32,
    pub height: f32,
    pub angle: f32,
    pub front_height: f32,
    pub back_height: f32,
    pub offset: f32,
    pub elevation: f32,
}

impl CarHitbox {
    const DEFAULT_OFFSET: f32 = 13.88;
    const DEFAULT_ELEVATION: f32 = 17.05;

    const fn from_preset(
        family: CarHitboxFamily,
        length: f32,
        width: f32,
        height: f32,
        angle: f32,
        front_height: f32,
        back_height: f32,
    ) -> Self {
        Self {
            family,
            length,
            width,
            height,
            angle,
            front_height,
            back_height,
            offset: Self::DEFAULT_OFFSET,
            elevation: Self::DEFAULT_ELEVATION,
        }
    }

    pub const fn breakout() -> Self {
        Self::from_preset(
            CarHitboxFamily::Breakout,
            131.4924,
            80.521,
            30.3,
            -0.9795,
            43.8976,
            46.1454,
        )
    }

    pub const fn dominus() -> Self {
        Self::from_preset(
            CarHitboxFamily::Dominus,
            127.9268,
            83.27995,
            31.3,
            -0.9635,
            47.2238,
            49.3749,
        )
    }

    pub const fn hybrid() -> Self {
        Self::from_preset(
            CarHitboxFamily::Hybrid,
            127.0192,
            82.18787,
            34.15907,
            -0.5499,
            54.0982,
            55.3173,
        )
    }

    pub const fn merc() -> Self {
        Self::from_preset(
            CarHitboxFamily::Merc,
            120.72,
            76.71,
            41.66,
            0.28,
            60.76,
            61.35,
        )
    }

    pub const fn octane() -> Self {
        Self::from_preset(
            CarHitboxFamily::Octane,
            118.0074,
            84.19941,
            36.15907,
            -0.5518,
            55.1449,
            56.2814,
        )
    }

    pub const fn plank() -> Self {
        Self::from_preset(
            CarHitboxFamily::Plank,
            128.8198,
            84.67036,
            29.3944,
            -0.3447,
            44.998,
            45.773,
        )
    }

    pub const fn for_family(family: CarHitboxFamily) -> Self {
        match family {
            CarHitboxFamily::Breakout => Self::breakout(),
            CarHitboxFamily::Dominus => Self::dominus(),
            CarHitboxFamily::Hybrid => Self::hybrid(),
            CarHitboxFamily::Merc => Self::merc(),
            CarHitboxFamily::Octane => Self::octane(),
            CarHitboxFamily::Plank => Self::plank(),
        }
    }
}

pub fn default_car_hitbox() -> CarHitbox {
    CarHitbox::octane()
}

pub const BALL_COLLISION_RADIUS: f32 = 92.75;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TouchCandidateScoring {
    pub strict_contact_gap_threshold: f32,
    pub relaxed_contact_gap_threshold: f32,
    pub strict_contact_min_position_deviation: f32,
    pub relaxed_contact_min_position_deviation: f32,
    pub strict_contact_min_velocity_deviation: f32,
    pub relaxed_contact_min_velocity_deviation: f32,
    pub relaxed_contact_score_penalty: f32,
    pub dodge_contact_score_bonus: f32,
    pub simultaneous_touch_score_margin: f32,
    pub contested_touch_score_margin: f32,
}

impl TouchCandidateScoring {
    pub const DEFAULT: Self = Self {
        strict_contact_gap_threshold: 5.0,
        relaxed_contact_gap_threshold: 25.0,
        strict_contact_min_position_deviation: 25.0,
        relaxed_contact_min_position_deviation: 500.0,
        strict_contact_min_velocity_deviation: 15.0,
        relaxed_contact_min_velocity_deviation: 1000.0,
        relaxed_contact_score_penalty: 100.0,
        dodge_contact_score_bonus: 1.0,
        simultaneous_touch_score_margin: 5.0,
        contested_touch_score_margin: 5.0,
    };

    pub fn accepts_contact_gap(
        self,
        closest_contact_gap: f32,
        position_deviation: f32,
        velocity_deviation: f32,
    ) -> bool {
        (closest_contact_gap <= self.strict_contact_gap_threshold
            && (position_deviation >= self.strict_contact_min_position_deviation
                || velocity_deviation >= self.strict_contact_min_velocity_deviation))
            || (closest_contact_gap <= self.relaxed_contact_gap_threshold
                && (position_deviation >= self.relaxed_contact_min_position_deviation
                    || velocity_deviation >= self.relaxed_contact_min_velocity_deviation))
    }

    pub fn score_contact_gap(self, closest_contact_gap: f32, dodge_contact: bool) -> f32 {
        let relaxed_penalty = if closest_contact_gap > self.strict_contact_gap_threshold {
            self.relaxed_contact_score_penalty
        } else {
            0.0
        };
        closest_contact_gap + relaxed_penalty
            - if dodge_contact {
                self.dodge_contact_score_bonus
            } else {
                0.0
            }
    }
}

pub fn car_hitbox_for_body_name(body_name: &str) -> Option<CarHitbox> {
    hitbox_family_for_body_name(body_name).map(CarHitbox::for_family)
}

pub fn car_hitbox_for_body_id(body_id: u32) -> Option<CarHitbox> {
    hitbox_family_for_body_id(body_id).map(CarHitbox::for_family)
}

pub fn car_hitbox_for_body_id_or_name(
    body_id: Option<u32>,
    body_name: Option<&str>,
) -> Option<CarHitbox> {
    hitbox_family_for_body_id_or_name(body_id, body_name).map(CarHitbox::for_family)
}

pub fn hitbox_family_for_body_id_or_name(
    body_id: Option<u32>,
    body_name: Option<&str>,
) -> Option<CarHitboxFamily> {
    body_id
        .and_then(hitbox_family_for_body_id)
        .or_else(|| body_name.and_then(hitbox_family_for_body_name))
}

pub fn hitbox_family_for_body_id(body_id: u32) -> Option<CarHitboxFamily> {
    match body_id {
        22 | 1416 | 1894 | 1932 | 3031 | 3311 | 6243 | 6489 | 7651 | 7696 | 7890 | 7901 | 8006
        | 8360 | 8361 | 8565 | 8566 | 8669 | 9357 | 10697 | 10698 | 10817 | 10822 | 11038
        | 11394 | 11505 | 11677 | 11800 | 11933 | 11949 | 12173 | 12315 | 12361 | 12484 => {
            Some(CarHitboxFamily::Breakout)
        }
        29 | 403 | 597 | 600 | 1018 | 1171 | 1286 | 1675 | 1689 | 1883 | 2070 | 2268 | 2666
        | 2950 | 2951 | 3155 | 3156 | 3157 | 3265 | 3426 | 3875 | 3879 | 3880 | 4014 | 4155
        | 4367 | 4472 | 4473 | 4745 | 4770 | 4781 | 4861 | 4864 | 5709 | 5773 | 5823 | 5858
        | 5964 | 5979 | 6122 | 6244 | 6247 | 6260 | 6836 | 7211 | 7337 | 7338 | 7341 | 7343
        | 7415 | 7512 | 7532 | 7593 | 7772 | 8454 | 9053 | 9088 | 9089 | 9140 | 9388 | 9894
        | 10094 | 10440 | 10441 | 10694 | 10695 | 11016 | 11095 | 11315 | 11336 | 11534 | 11941
        | 11996 | 12106 | 12142 | 12262 | 12286 | 12325 | 12382 | 12563 | 12669 => {
            Some(CarHitboxFamily::Dominus)
        }
        28 | 31 | 1159 | 1317 | 1624 | 1856 | 2269 | 3451 | 3582 | 3702 | 5470 | 5488 | 5879
        | 7012 | 9084 | 9085 | 9427 | 10044 | 10805 | 11138 | 11141 | 11379 | 11932 | 12569
        | 12652 => Some(CarHitboxFamily::Hybrid),
        30 | 4780 | 7336 | 7477 | 7815 | 7979 | 10689 | 11098 | 11736 | 11905 | 11950 | 12318
        | 12335 => Some(CarHitboxFamily::Merc),
        21 | 23 | 25 | 26 | 27 | 402 | 404 | 523 | 607 | 625 | 723 | 1172 | 1295 | 1300 | 1475
        | 1478 | 1533 | 1568 | 1623 | 2665 | 2853 | 2919 | 2949 | 4284 | 4318 | 4319 | 4320
        | 4782 | 4906 | 5020 | 5039 | 5188 | 5361 | 5547 | 5713 | 5837 | 5951 | 6939 | 7947
        | 7948 | 8383 | 8806 | 8807 | 10896 | 10897 | 10900 | 10901 | 11314 | 11603 | 12104
        | 12105 => Some(CarHitboxFamily::Octane),
        24 | 803 | 1603 | 1691 | 1919 | 3594 | 3614 | 3622 | 4268 | 5265 | 7052 | 8524 => {
            Some(CarHitboxFamily::Plank)
        }
        _ => None,
    }
}

pub fn hitbox_family_for_body_name(body_name: &str) -> Option<CarHitboxFamily> {
    let normalized = normalized_car_body_name(body_name);
    if normalized.is_empty() {
        return None;
    }

    if normalized_body_name_matches(&normalized, BREAKOUT_HITBOX_BODIES) {
        Some(CarHitboxFamily::Breakout)
    } else if normalized_body_name_matches(&normalized, DOMINUS_HITBOX_BODIES) {
        Some(CarHitboxFamily::Dominus)
    } else if normalized_body_name_matches(&normalized, HYBRID_HITBOX_BODIES) {
        Some(CarHitboxFamily::Hybrid)
    } else if normalized_body_name_matches(&normalized, MERC_HITBOX_BODIES) {
        Some(CarHitboxFamily::Merc)
    } else if normalized_body_name_matches(&normalized, OCTANE_HITBOX_BODIES) {
        Some(CarHitboxFamily::Octane)
    } else if normalized_body_name_matches(&normalized, PLANK_HITBOX_BODIES) {
        Some(CarHitboxFamily::Plank)
    } else {
        None
    }
}

fn normalized_body_name_matches(normalized: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| normalized_car_body_name(candidate) == normalized)
}

fn normalized_car_body_name(body_name: &str) -> String {
    body_name
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

// Body-to-family mapping follows Epic's Rocket League Car Hitboxes support
// article when available. Newer body IDs are Rocket League product IDs surfaced
// in ClientLoadouts and cross-checked against a BakkesMod body product dump.
// Preset dimensions follow rlhitboxes.com/stats.json.

const BREAKOUT_HITBOX_BODIES: &[&str] = &[
    "1966 Cadillac DeVille",
    "Ace",
    "Animus GP",
    "Aston Martin Valhalla",
    "Azura",
    "Breakout",
    "Breakout Type-S",
    "Breakout X",
    "Cyberpunk Quadra",
    "Cyclone",
    "Diesel",
    "Emperor",
    "Emperor II",
    "Emperor II: Frozen",
    "Emperor II: Scorched",
    "Fast & Furious Mazda RX-7",
    "Ferrari F40",
    "Fast and Furious Mazda-RX7",
    "Fuse",
    "Havoc",
    "Chevrolet Corvette Stingray",
    "Chevrolet Corvette ZR1",
    "Komodo",
    "Mako",
    "McLaren Senna",
    "Megastar",
    "Nexus",
    "Nexus SC",
    "Pontiac Firebird",
    "Porsche 918 Spyder",
    "Quadra Turbo-R",
    "Redline",
    "Revolver",
    "Samurai",
    "The Incredibile",
    "Whiplash",
];

const DOMINUS_HITBOX_BODIES: &[&str] = &[
    "'89 Batmobile",
    "007's Aston Martin DBS",
    "007's Aston Martin Valhalla",
    "Admiral",
    "Aftershock",
    "Back To The Future Time Machine",
    "Batmobile (1989)",
    "Batmobile (2022)",
    "Bumblebee Car",
    "Bumblebee",
    "BMW M3 (E30)",
    "BMW M2 Racing",
    "BMW M4 GT3 EVO",
    "BMW M240i",
    "Chikara",
    "Chikara G1",
    "Chikara GXT",
    "DeLorean Time Machine",
    "Diestro",
    "Dodge Charger Daytona Scat Pack",
    "Dodger Charger Daytona Scat Pack",
    "Dominus",
    "Dominus: Neon",
    "Dominus GT",
    "Ecto-1",
    "Ecto-1 (Ghostbusters)",
    "Fast & Furious Dodge Charger",
    "Fast and Furious Dodge Charger",
    "Fast & Furious Dodge Charger SRT Hellcat",
    "Ferrari 296 GTB",
    "Ford Mustang Shelby GT350R RLE",
    "Ford Mustang Shelby GT500",
    "Ford Mustang GTD",
    "Gazella GT",
    "Gazella GT (Hot Wheels)",
    "Guardian",
    "Guardian G1",
    "Guardian GXT",
    "Homer's Car",
    "Hotshot",
    "Ice Charger",
    "Imperator DT5",
    "K.I.T.T.",
    "K.I.T.T. (Knight Rider)",
    "Lamborghini Countach LPI 800-4",
    "Lamborghini Huracan STO",
    "Lamborghini Huracán STO",
    "Lightning McQueen",
    "Lightning McQueen Car",
    "Lockjaw",
    "Maestro",
    "Magnifique",
    "Magnifique GXT",
    "Mamba",
    "Masamune",
    "Maven",
    "Maverick",
    "Maverick G1",
    "Maverick GXT",
    "McLaren 570S",
    "McLaren 765LT",
    "McLaren P1",
    "Mercedes-AMG GT 63 S",
    "Mercedes-Benz CLA",
    "MR11",
    "MR11 (Hot Wheels)",
    "NASCAR Chevrolet Camaro",
    "NASCAR Ford Mustang",
    "NASCAR Toyota Camry",
    "NASCAR Next Gen Chevrolet Camaro",
    "NASCAR Next Gen Chevrolet Camaro (2022)",
    "NASCAR Next Gen Ford Mustang",
    "NASCAR Next Gen Ford Mustang (2022)",
    "NASCAR Next Gen Toyota Camry",
    "NASCAR Next Gen Toyota Camry (2022)",
    "Nemesis",
    "Nissan 350Z",
    "Nissan Fairlady Z",
    "Nissan Fairlady Z RLE",
    "Nissan Z Performance",
    "Nissan Z Performance Car",
    "Peregrine TT",
    "Perigrine TT",
    "Porsche 911 GT3 RS",
    "Porsche 911 Turbo",
    "Porsche 911 Turbo RLE",
    "Ripper",
    "Ronin",
    "Ronin G1",
    "Ronin GXT",
    "Samus' Gunship",
    "Samus' Gunship (Nintendo Exclusive)",
    "Scorpion",
    "Tyranno",
    "Tyranno GXT",
    "Werewolf",
    "Zefira",
];

const HYBRID_HITBOX_BODIES: &[&str] = &[
    "Beskar",
    "Chrysler Pacifica",
    "Endo",
    "Esper",
    "Fast & Furious Nissan Skyline",
    "Fast and Furious Nissan Skyline",
    "Fast & Furious Pontiac Fiero",
    "Fast and Furious Pontiac Fiero",
    "Hearse",
    "Insidio",
    "Jager 619",
    "Jäger 619",
    "Jäger 619 RS",
    "Lamborghini Urus",
    "Lamborghini Urus SE",
    "Nimbus",
    "Nissan Silvia",
    "Nissan Silvia RLE",
    "Nissan Skyline GT-R",
    "Nissan Skyline GT-R (R32)",
    "Primo",
    "R3MX",
    "R3MX GXT",
    "RAM 1500 RHO",
    "Rivian R1S",
    "Tesla Cybertruck",
    "Tygris",
    "Venom",
    "Void Burn",
    "X-Devil",
    "X-Devil MK2",
];

const MERC_HITBOX_BODIES: &[&str] = &[
    "Battle Bus",
    "Behemoth",
    "Chevrolet Astro",
    "Defender D7X-R",
    "Ford Bronco Raptor RLE",
    "Merc",
    "The Mystery Machine",
    "Nomad",
    "Nomad GXT",
    "Pizza Planet Delivery Truck",
    "Recoil AV",
    "Stampede",
    "Turtle Van",
];

const OCTANE_HITBOX_BODIES: &[&str] = &[
    "007's Aston Martin DB5",
    "Armadillo",
    "Armadillo (Xbox Exclusive)",
    "Backfire",
    "BMW 1 Series",
    "BMW 1 Series RLE",
    "Bone Shaker",
    "Corlay",
    "Dingo",
    "Fast 4WD",
    "Fast 4WD (Hot Wheels)",
    "Fennec",
    "Fennec ZR-F",
    "Ford F-150 RLE",
    "Ford Mustang Mach-E RLE",
    "Gizmo",
    "Grog",
    "Harbinger",
    "Harbinger GXT",
    "Hogsticker",
    "Hogsticker (Xbox Exclusive)",
    "Honda Civic Type R",
    "Honda Civic Type R-LE",
    "Jackal",
    "Jurassic Jeep Wrangler",
    "Jeep Wrangler Rubicon",
    "Marauder",
    "Mario NSR",
    "Luigi NSR",
    "Mudcat",
    "Mudcat G1",
    "Mudcat GXT",
    "Octane",
    "Octane ZSR",
    "Outlaw",
    "Outlaw GXT",
    "Patty Wagon",
    "Proteus",
    "Psyclops",
    "Road Hog",
    "Road Hog XL",
    "Scarab",
    "Shokunin",
    "Shokunin GXT",
    "Sweet Tooth",
    "Sweet Tooth (PlayStation Exclusive)",
    "Takumi",
    "Takumi RX-T",
    "The Dark Knight's Tumbler",
    "The Dark Knight Tumbler",
    "Triton",
    "Twinzer",
    "Volkswagen Golf GTI",
    "Volkswagen Golf GTI RLE",
    "Vulcan",
    "Xentari",
    "Zippy",
];

const PLANK_HITBOX_BODIES: &[&str] = &[
    "'16 Batmobile",
    "Batmobile (2016)",
    "Bugatti Centodieci",
    "Artemis",
    "Artemis G1",
    "Artemis GXT",
    "Centio",
    "Centio V17",
    "Formula 1 2021",
    "Formula 1 2022",
    "Mantis",
    "Paladin",
    "Sentinel",
    "Twin Mill III",
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct CarHitboxContactEstimate {
    pub distance: f32,
    pub local_ball_position: glam::Vec3,
    pub local_contact_point: glam::Vec3,
}

pub(crate) fn car_hitbox_contact_estimate(
    ball_position: glam::Vec3,
    player_body: &boxcars::RigidBody,
    hitbox: CarHitbox,
) -> Option<CarHitboxContactEstimate> {
    let car_local_ball_position = quat_to_glam(&player_body.rotation).inverse()
        * (ball_position - vec_to_glam(&player_body.location));
    let hitbox_center = glam::Vec3::new(hitbox.offset, 0.0, hitbox.elevation);
    let hitbox_rotation = glam::Quat::from_rotation_y(hitbox.angle.to_radians());
    let local_ball_position = hitbox_rotation.inverse() * (car_local_ball_position - hitbox_center);
    if !local_ball_position.is_finite() {
        return None;
    }

    let x_min = -hitbox.length / 2.0;
    let x_max = hitbox.length / 2.0;
    let y_min = -hitbox.width / 2.0;
    let y_max = hitbox.width / 2.0;
    let z_min = -hitbox.height / 2.0;
    let z_max = hitbox.height / 2.0;
    let local_contact_point = glam::Vec3::new(
        local_ball_position.x.clamp(x_min, x_max),
        local_ball_position.y.clamp(y_min, y_max),
        local_ball_position.z.clamp(z_min, z_max),
    );
    let distance = (local_ball_position - local_contact_point).length();
    if !distance.is_finite() {
        return None;
    }

    Some(CarHitboxContactEstimate {
        distance,
        local_ball_position,
        local_contact_point,
    })
}

pub(crate) fn car_hitbox_distance(
    ball_position: glam::Vec3,
    player_body: &boxcars::RigidBody,
    hitbox: CarHitbox,
) -> Option<f32> {
    car_hitbox_contact_estimate(ball_position, player_body, hitbox)
        .map(|estimate| estimate.distance)
}

pub fn car_hitbox_ball_contact_gap(
    ball_position: glam::Vec3,
    player_body: &boxcars::RigidBody,
    hitbox: CarHitbox,
) -> Option<f32> {
    car_hitbox_distance(ball_position, player_body, hitbox)
        .map(|center_distance| (center_distance - BALL_COLLISION_RADIUS).max(0.0))
}

#[derive(Debug, Clone, Copy)]
struct OrientedCarHitbox {
    center: glam::Vec3,
    axes: [glam::Vec3; 3],
    half_extents: glam::Vec3,
}

impl OrientedCarHitbox {
    fn corners(self) -> [glam::Vec3; 8] {
        let x = self.axes[0] * self.half_extents.x;
        let y = self.axes[1] * self.half_extents.y;
        let z = self.axes[2] * self.half_extents.z;

        [
            self.center - x - y - z,
            self.center - x - y + z,
            self.center - x + y - z,
            self.center - x + y + z,
            self.center + x - y - z,
            self.center + x - y + z,
            self.center + x + y - z,
            self.center + x + y + z,
        ]
    }

    fn edge_segments(self) -> [(glam::Vec3, glam::Vec3); 12] {
        let corners = self.corners();
        [
            (corners[0], corners[1]),
            (corners[0], corners[2]),
            (corners[0], corners[4]),
            (corners[3], corners[1]),
            (corners[3], corners[2]),
            (corners[3], corners[7]),
            (corners[5], corners[1]),
            (corners[5], corners[4]),
            (corners[5], corners[7]),
            (corners[6], corners[2]),
            (corners[6], corners[4]),
            (corners[6], corners[7]),
        ]
    }
}

fn oriented_car_hitbox(
    player_body: &boxcars::RigidBody,
    hitbox: CarHitbox,
) -> Option<OrientedCarHitbox> {
    let car_position = vec_to_glam(&player_body.location);
    let car_rotation = quat_to_glam(&player_body.rotation);
    let hitbox_center = glam::Vec3::new(hitbox.offset, 0.0, hitbox.elevation);
    let hitbox_rotation = glam::Quat::from_rotation_y(hitbox.angle.to_radians());
    let rotation = car_rotation * hitbox_rotation;
    let center = car_position + car_rotation * hitbox_center;
    let axes = [
        rotation * glam::Vec3::X,
        rotation * glam::Vec3::Y,
        rotation * glam::Vec3::Z,
    ];
    let half_extents =
        glam::Vec3::new(hitbox.length / 2.0, hitbox.width / 2.0, hitbox.height / 2.0);

    if center.is_finite() && axes.iter().all(|axis| axis.is_finite()) && half_extents.is_finite() {
        Some(OrientedCarHitbox {
            center,
            axes,
            half_extents,
        })
    } else {
        None
    }
}

fn point_oriented_box_distance(point: glam::Vec3, hitbox: OrientedCarHitbox) -> f32 {
    let delta = point - hitbox.center;
    let mut closest = hitbox.center;
    for (axis, half_extent) in hitbox.axes.into_iter().zip([
        hitbox.half_extents.x,
        hitbox.half_extents.y,
        hitbox.half_extents.z,
    ]) {
        let distance_on_axis = delta.dot(axis).clamp(-half_extent, half_extent);
        closest += axis * distance_on_axis;
    }
    (point - closest).length()
}

fn segment_segment_distance(
    left_start: glam::Vec3,
    left_end: glam::Vec3,
    right_start: glam::Vec3,
    right_end: glam::Vec3,
) -> f32 {
    let left_delta = left_end - left_start;
    let right_delta = right_end - right_start;
    let offset = left_start - right_start;
    let left_length_sq = left_delta.length_squared();
    let right_length_sq = right_delta.length_squared();
    let left_right_dot = left_delta.dot(right_delta);
    let left_offset_dot = left_delta.dot(offset);
    let right_offset_dot = right_delta.dot(offset);
    let denominator = left_length_sq * right_length_sq - left_right_dot * left_right_dot;

    let mut left_t;
    let mut right_t;
    if denominator.abs() > f32::EPSILON {
        left_t = ((left_right_dot * right_offset_dot - right_length_sq * left_offset_dot)
            / denominator)
            .clamp(0.0, 1.0);
    } else {
        left_t = 0.0;
    }

    right_t = (left_right_dot * left_t + right_offset_dot) / right_length_sq;
    if right_t < 0.0 {
        right_t = 0.0;
        left_t = (-left_offset_dot / left_length_sq).clamp(0.0, 1.0);
    } else if right_t > 1.0 {
        right_t = 1.0;
        left_t = ((left_right_dot - left_offset_dot) / left_length_sq).clamp(0.0, 1.0);
    }

    let left_closest = left_start + left_delta * left_t;
    let right_closest = right_start + right_delta * right_t;
    (left_closest - right_closest).length()
}

fn projected_interval(points: &[glam::Vec3; 8], axis: glam::Vec3) -> (f32, f32) {
    points
        .iter()
        .map(|point| point.dot(axis))
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), value| {
            (min.min(value), max.max(value))
        })
}

fn separated_on_axis(
    left_corners: &[glam::Vec3; 8],
    right_corners: &[glam::Vec3; 8],
    axis: glam::Vec3,
) -> bool {
    if axis.length_squared() <= f32::EPSILON {
        return false;
    }
    let axis = axis.normalize();
    let (left_min, left_max) = projected_interval(left_corners, axis);
    let (right_min, right_max) = projected_interval(right_corners, axis);
    left_max < right_min || right_max < left_min
}

fn oriented_boxes_intersect(left: OrientedCarHitbox, right: OrientedCarHitbox) -> bool {
    let left_corners = left.corners();
    let right_corners = right.corners();
    let face_axes = left.axes.into_iter().chain(right.axes);
    let edge_axes = left
        .axes
        .into_iter()
        .flat_map(|left_axis| right.axes.map(|right_axis| left_axis.cross(right_axis)));

    face_axes
        .chain(edge_axes)
        .all(|axis| !separated_on_axis(&left_corners, &right_corners, axis))
}

fn oriented_box_distance(left: OrientedCarHitbox, right: OrientedCarHitbox) -> f32 {
    if oriented_boxes_intersect(left, right) {
        return 0.0;
    }

    let left_corners = left.corners();
    let right_corners = right.corners();
    let mut distance = f32::INFINITY;

    for corner in left_corners {
        distance = distance.min(point_oriented_box_distance(corner, right));
    }
    for corner in right_corners {
        distance = distance.min(point_oriented_box_distance(corner, left));
    }
    for (left_start, left_end) in left.edge_segments() {
        for (right_start, right_end) in right.edge_segments() {
            distance = distance.min(segment_segment_distance(
                left_start,
                left_end,
                right_start,
                right_end,
            ));
        }
    }

    distance
}

pub fn car_hitbox_pair_contact_gap(
    left_body: &boxcars::RigidBody,
    left_hitbox: CarHitbox,
    right_body: &boxcars::RigidBody,
    right_hitbox: CarHitbox,
) -> Option<f32> {
    let left = oriented_car_hitbox(left_body, left_hitbox)?;
    let right = oriented_car_hitbox(right_body, right_hitbox)?;
    let distance = oriented_box_distance(left, right);
    distance.is_finite().then_some(distance)
}

pub fn car_hitbox_min_world_z(player_body: &boxcars::RigidBody, hitbox: CarHitbox) -> Option<f32> {
    let car_position = vec_to_glam(&player_body.location);
    let car_rotation = quat_to_glam(&player_body.rotation);
    let hitbox_center = glam::Vec3::new(hitbox.offset, 0.0, hitbox.elevation);
    let hitbox_rotation = glam::Quat::from_rotation_y(hitbox.angle.to_radians());
    let mut min_z: Option<f32> = None;

    for x in [-hitbox.length / 2.0, hitbox.length / 2.0] {
        for y in [-hitbox.width / 2.0, hitbox.width / 2.0] {
            for z in [-hitbox.height / 2.0, hitbox.height / 2.0] {
                let local_corner = glam::Vec3::new(x, y, z);
                let world_corner =
                    car_position + car_rotation * (hitbox_center + hitbox_rotation * local_corner);
                if !world_corner.z.is_finite() {
                    return None;
                }
                min_z = Some(min_z.map_or(world_corner.z, |current| current.min(world_corner.z)));
            }
        }
    }

    min_z
}

pub fn car_hitbox_touches_floor(player_body: &boxcars::RigidBody, hitbox: CarHitbox) -> bool {
    const FLOOR_CONTACT_MAX_Z: f32 = 5.0;

    car_hitbox_min_world_z(player_body, hitbox).is_some_and(|min_z| min_z <= FLOOR_CONTACT_MAX_Z)
}

pub fn touch_candidate_contact_gap_rank_with_hitbox(
    ball_body: &boxcars::RigidBody,
    player_body: &boxcars::RigidBody,
    hitbox: CarHitbox,
) -> Option<(f32, f32)> {
    touch_candidate_rank_with_hitbox(ball_body, player_body, hitbox).map(
        |(closest_center_distance, current_center_distance)| {
            (
                (closest_center_distance - BALL_COLLISION_RADIUS).max(0.0),
                (current_center_distance - BALL_COLLISION_RADIUS).max(0.0),
            )
        },
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BallTrajectoryDeviation {
    pub position_deviation: f32,
    pub velocity_deviation: f32,
    pub seconds: f32,
}

pub fn ball_trajectory_deviation_with_gravity(
    previous_body: &boxcars::RigidBody,
    previous_time: f32,
    actual_body: &boxcars::RigidBody,
    actual_time: f32,
    gravity_z: f32,
) -> Option<BallTrajectoryDeviation> {
    let seconds = actual_time - previous_time;
    if !seconds.is_finite() || seconds <= 0.0 {
        return None;
    }

    let previous_velocity = vec_to_glam(&previous_body.linear_velocity?);
    let actual_velocity = vec_to_glam(&actual_body.linear_velocity?);
    let gravity = glam::Vec3::new(0.0, 0.0, gravity_z);
    let expected_position = vec_to_glam(&previous_body.location)
        + previous_velocity * seconds
        + 0.5 * gravity * seconds * seconds;
    let expected_velocity = previous_velocity + gravity * seconds;
    let actual_position = vec_to_glam(&actual_body.location);

    let position_deviation = (actual_position - expected_position).length();
    let velocity_deviation = (actual_velocity - expected_velocity).length();
    if !position_deviation.is_finite() || !velocity_deviation.is_finite() {
        return None;
    }

    Some(BallTrajectoryDeviation {
        position_deviation,
        velocity_deviation,
        seconds,
    })
}

/// Ranks how plausible it is that `player_body` was the car that touched the
/// ball near the current frame, using velocity-applied closest approach to the
/// car's moving, oriented hitbox.
///
/// The frame's ball state can already be slightly post-contact, so we do not
/// just compare current distance. Instead we look for the minimum ball/hitbox
/// separation over a short window centered slightly before the frame time.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn touch_candidate_rank(
    ball_body: &boxcars::RigidBody,
    player_body: &boxcars::RigidBody,
) -> Option<(f32, f32)> {
    touch_candidate_rank_with_hitbox(ball_body, player_body, default_car_hitbox())
}

pub fn touch_candidate_rank_with_hitbox(
    ball_body: &boxcars::RigidBody,
    player_body: &boxcars::RigidBody,
    hitbox: CarHitbox,
) -> Option<(f32, f32)> {
    const TOUCH_LOOKBACK_SECONDS: f32 = 0.12;
    const TOUCH_LOOKAHEAD_SECONDS: f32 = 0.03;
    const TOUCH_RANK_SAMPLES: usize = 9;

    let current_distance =
        car_hitbox_distance(vec_to_glam(&ball_body.location), player_body, hitbox)?;

    let mut closest_distance = current_distance;
    for sample_index in 0..=TOUCH_RANK_SAMPLES {
        let sample_fraction = sample_index as f32 / TOUCH_RANK_SAMPLES as f32;
        let sample_time = -TOUCH_LOOKBACK_SECONDS
            + sample_fraction * (TOUCH_LOOKBACK_SECONDS + TOUCH_LOOKAHEAD_SECONDS);
        let sample_ball_body = apply_velocities_to_rigid_body(ball_body, sample_time);
        let sample_player_body = apply_velocities_to_rigid_body(player_body, sample_time);
        let sample_distance = car_hitbox_distance(
            vec_to_glam(&sample_ball_body.location),
            &sample_player_body,
            hitbox,
        )?;
        closest_distance = closest_distance.min(sample_distance);
    }

    Some((closest_distance, current_distance))
}

fn apply_angular_velocity(rigid_body: &boxcars::RigidBody, time_delta: f32) -> boxcars::Quaternion {
    let rbav = rigid_body.angular_velocity.unwrap_or(boxcars::Vector3f {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    });
    let angular_velocity = glam::Vec3::new(rbav.x, rbav.y, rbav.z);
    let magnitude = angular_velocity.length();
    let angular_velocity_unit_vector = angular_velocity.normalize_or_zero();

    let mut rotation = glam::Quat::from_xyzw(
        rigid_body.rotation.x,
        rigid_body.rotation.y,
        rigid_body.rotation.z,
        rigid_body.rotation.w,
    );

    if angular_velocity_unit_vector.length() != 0.0 {
        let delta_rotation =
            glam::Quat::from_axis_angle(angular_velocity_unit_vector, magnitude * time_delta);
        rotation *= delta_rotation;
    }
    rotation = if rotation.length_squared() > 0.0 {
        rotation.normalize()
    } else {
        glam::Quat::IDENTITY
    };

    boxcars::Quaternion {
        x: rotation.x,
        y: rotation.y,
        z: rotation.z,
        w: rotation.w,
    }
}

/// Interpolates between two [`boxcars::RigidBody`] states based on the provided time.
///
/// # Arguments
///
/// * `start_body` - The initial `RigidBody` state.
/// * `start_time` - The timestamp of the initial `RigidBody` state.
/// * `end_body` - The final `RigidBody` state.
/// * `end_time` - The timestamp of the final `RigidBody` state.
/// * `time` - The desired timestamp to interpolate to.
///
/// # Returns
///
/// A new [`boxcars::RigidBody`] that represents the interpolated state at the specified time.
pub fn get_interpolated_rigid_body(
    start_body: &boxcars::RigidBody,
    start_time: f32,
    end_body: &boxcars::RigidBody,
    end_time: f32,
    time: f32,
) -> SubtrActorResult<boxcars::RigidBody> {
    if !(start_time <= time && time <= end_time) {
        return SubtrActorError::new_result(SubtrActorErrorVariant::InterpolationTimeOrderError {
            start_time,
            time,
            end_time,
        });
    }

    if start_body.linear_velocity.is_none() || end_body.linear_velocity.is_none() {
        return Ok(*start_body);
    }

    let duration = end_time - start_time;
    let interpolation_amount = (time - start_time) / duration;
    let start_position = vec_to_glam(&start_body.location);
    let end_position = vec_to_glam(&end_body.location);
    let interpolated_location = start_position.lerp(end_position, interpolation_amount);
    let start_rotation = quat_to_glam(&start_body.rotation);
    let end_rotation = quat_to_glam(&end_body.rotation);
    let interpolated_rotation = start_rotation.slerp(end_rotation, interpolation_amount);

    Ok(boxcars::RigidBody {
        location: glam_to_vec(&interpolated_location),
        rotation: glam_to_quat(&interpolated_rotation),
        sleeping: start_body.sleeping,
        linear_velocity: start_body.linear_velocity,
        angular_velocity: start_body.angular_velocity,
    })
}

#[cfg(test)]
#[path = "geometry_tests.rs"]
mod tests;
