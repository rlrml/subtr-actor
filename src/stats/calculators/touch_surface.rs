use super::*;

impl TouchCalculator {
    pub(crate) fn height_band_for_touch(
        sample: Option<&PlayerVerticalSample>,
    ) -> PlayerVerticalBand {
        let Some(sample) = sample else {
            return PlayerVerticalBand::Ground;
        };

        if sample.height < AERIAL_TOUCH_MIN_PLAYER_Z {
            PlayerVerticalBand::Ground
        } else {
            sample.band
        }
    }

    pub(crate) fn surface_for_touch(
        player_position: Option<glam::Vec3>,
        height_band: PlayerVerticalBand,
    ) -> TouchSurface {
        if player_position.is_some_and(player_is_on_wall) {
            TouchSurface::Wall
        } else if height_band.is_grounded() {
            TouchSurface::Ground
        } else {
            TouchSurface::Air
        }
    }
}
