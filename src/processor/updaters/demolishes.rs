use super::*;

const DEMOLISH_VELOCITY_NORMALIZATION_FACTOR: f32 = 100.0;

impl<'a> ReplayProcessor<'a> {
    /// Detects and records demolishes observed in actor state and frame updates.
    pub(crate) fn update_demolishes(
        &mut self,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        if self.demolish_format.is_none() {
            self.demolish_format = self.detect_demolish_format();
        }

        let new_demolishes: Vec<_> = self.get_active_demos()?.collect();

        for demolish in new_demolishes {
            self.try_push_demolish(&demolish, frame, frame_index);
        }

        for update in &frame.updated_actors {
            let demolish = match &update.attribute {
                boxcars::Attribute::DemolishExtended(d) => {
                    self.demolish_format = Some(DemolishFormat::Extended);
                    Some(DemolishAttribute::Extended(**d))
                }
                boxcars::Attribute::DemolishFx(d) => {
                    self.demolish_format = Some(DemolishFormat::Fx);
                    Some(DemolishAttribute::Fx(**d))
                }
                _ => None,
            };
            if let Some(demolish) = demolish {
                self.try_push_demolish(&demolish, frame, frame_index);
            }
        }

        Ok(())
    }

    fn try_push_demolish(
        &mut self,
        demolish: &DemolishAttribute,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) {
        if self.demolish_is_known(demolish, frame_index) {
            return;
        }
        self.known_demolishes.push((demolish.clone(), frame_index));
        if let Ok(info) = self.build_demolish_info(demolish, frame, frame_index) {
            self.demolishes.push(info);
        } else {
            log::warn!(
                "Error building demolish info: attacker_car={:?}, victim_car={:?}",
                demolish.attacker_actor_id(),
                demolish.victim_actor_id(),
            );
        }
    }

    fn build_demolish_info(
        &self,
        demo: &DemolishAttribute,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<DemolishInfo> {
        let attacker = self.get_player_id_from_car_id(&demo.attacker_actor_id())?;
        let victim = self.get_player_id_from_car_id(&demo.victim_actor_id())?;
        let (current_rigid_body, _) =
            self.get_player_rigid_body_and_updated_or_recently_deleted(&victim)?;
        Ok(DemolishInfo {
            time: frame.time,
            seconds_remaining: self.get_seconds_remaining()?,
            frame: frame_index,
            attacker,
            victim,
            attacker_velocity: self.normalize_vector_by_factor(
                demo.attacker_velocity(),
                DEMOLISH_VELOCITY_NORMALIZATION_FACTOR,
            ),
            victim_velocity: self.normalize_vector_by_factor(
                demo.victim_velocity(),
                DEMOLISH_VELOCITY_NORMALIZATION_FACTOR,
            ),
            victim_location: self.normalize_vector(current_rigid_body.location),
        })
    }
}
