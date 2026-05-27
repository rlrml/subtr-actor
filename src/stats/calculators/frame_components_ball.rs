use super::super::BallSample;

#[derive(Debug, Clone, Default)]
pub enum BallFrameState {
    #[default]
    Missing,
    Present(BallSample),
}

impl BallFrameState {
    pub fn sample(&self) -> Option<&BallSample> {
        match self {
            Self::Missing => None,
            Self::Present(ball) => Some(ball),
        }
    }

    pub fn into_sample(self) -> Option<BallSample> {
        match self {
            Self::Missing => None,
            Self::Present(ball) => Some(ball),
        }
    }

    pub fn position(&self) -> Option<glam::Vec3> {
        self.sample().map(BallSample::position)
    }

    pub fn velocity(&self) -> Option<glam::Vec3> {
        self.sample().map(BallSample::velocity)
    }
}

impl From<BallSample> for BallFrameState {
    fn from(ball: BallSample) -> Self {
        Self::Present(ball)
    }
}

impl From<Option<BallSample>> for BallFrameState {
    fn from(ball: Option<BallSample>) -> Self {
        match ball {
            Some(ball) => Self::Present(ball),
            None => Self::Missing,
        }
    }
}
