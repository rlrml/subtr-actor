#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaMechanicKind {
    SpeedFlip = 1,
    HalfFlip = 2,
    Wavedash = 3,
    BallCarry = 4,
    AirDribble = 5,
    CeilingShot = 6,
    WallAerial = 7,
    WallAerialShot = 8,
    Center = 9,
    FlipReset = 10,
    DoubleTap = 11,
    Flick = 12,
    MustyFlick = 13,
    OneTimer = 14,
    Pass = 15,
    HalfVolley = 16,
    Whiff = 17,
    Bump = 18,
    Backboard = 19,
    BoostPickup = 20,
    Demo = 21,
    FiftyFifty = 22,
    AerialGoal = 23,
    HighAerialGoal = 24,
    LongDistanceGoal = 25,
    OwnHalfGoal = 26,
    EmptyNetGoal = 27,
    CounterAttackGoal = 28,
    FlickGoal = 29,
    DoubleTapGoal = 30,
    OneTimerGoal = 31,
    AirDribbleGoal = 32,
    FlipResetGoal = 33,
    HalfVolleyGoal = 34,
    Goal = 35,
    Shot = 36,
    Save = 37,
    Assist = 38,
    Death = 39,
    PassingGoal = 40,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SaMechanicEvent {
    pub kind: SaMechanicKind,
    pub player_index: u32,
    pub is_team_0: u8,
    pub frame_number: u64,
    pub time: f32,
    pub confidence: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaTeamEventKind {
    Rush = 1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SaTeamEvent {
    pub kind: SaTeamEventKind,
    pub is_team_0: u8,
    pub start_frame: u64,
    pub end_frame: u64,
    pub start_time: f32,
    pub end_time: f32,
    pub attackers: u32,
    pub defenders: u32,
    pub confidence: f32,
}
