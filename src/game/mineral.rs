use rand::Rng;

use crate::{
    game::*,
    utils::event_queue::{self, EventCallback, QueuedEvent},
};

#[derive(Clone, Debug)]
pub struct MineralSource {
    pub id: u64,
    pub mineral_type: ResourceType,
    pub pos: RoomPos,
    pub amount: u32,
    pub refresh_timer: u128,
    pub density: MineralDensity,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct MineralRefreshEvent {
    pub id: u64,
}

impl EventCallback for MineralRefreshEvent {
    fn callback(&self, game_state: &mut GameState, event_queue: &mut EventQueue) {
        let src = game_state.mineral_sources.get_mut(&self.id).unwrap();

        src.refresh(&mut game_state.random, game_state.tick);

        event_queue.enqueue(QueuedEvent {
            tick: src.refresh_timer,
            callback: Box::new(MineralRefreshEvent { id: self.id }),
        });
    }
}

impl MineralSource {
    pub fn refresh<T>(&mut self, rng: &mut T, tick: u128)
    where
        T: Rng,
    {
        if self.density == MineralDensity::Low
            || self.density == MineralDensity::Ultra
            || rng.random_ratio(MINERAL_DENSITY_CHANGE_CHANCE_PERCENT, 100)
        {
            self.density = match self.density {
                MineralDensity::Low => utils::random_choice(
                    rng,
                    &[
                        MineralDensity::Medium,
                        MineralDensity::High,
                        MineralDensity::Ultra,
                    ],
                    &[
                        MINERAL_DENSITY_MEDIUM_CHANCE_PER1000,
                        MINERAL_DENSITY_HIGH_CHANCE_PER1000,
                        MINERAL_DENSITY_ULTRA_CHANCE_PER1000,
                    ],
                ),
                MineralDensity::Medium => utils::random_choice(
                    rng,
                    &[
                        MineralDensity::Low,
                        MineralDensity::High,
                        MineralDensity::Ultra,
                    ],
                    &[
                        MINERAL_DENSITY_LOW_CHANCE_PER1000,
                        MINERAL_DENSITY_HIGH_CHANCE_PER1000,
                        MINERAL_DENSITY_ULTRA_CHANCE_PER1000,
                    ],
                ),
                MineralDensity::High => utils::random_choice(
                    rng,
                    &[
                        MineralDensity::Low,
                        MineralDensity::Medium,
                        MineralDensity::Ultra,
                    ],
                    &[
                        MINERAL_DENSITY_LOW_CHANCE_PER1000,
                        MINERAL_DENSITY_MEDIUM_CHANCE_PER1000,
                        MINERAL_DENSITY_ULTRA_CHANCE_PER1000,
                    ],
                ),
                MineralDensity::Ultra => utils::random_choice(
                    rng,
                    &[
                        MineralDensity::Low,
                        MineralDensity::Medium,
                        MineralDensity::High,
                    ],
                    &[
                        MINERAL_DENSITY_LOW_CHANCE_PER1000,
                        MINERAL_DENSITY_MEDIUM_CHANCE_PER1000,
                        MINERAL_DENSITY_HIGH_CHANCE_PER1000,
                    ],
                ),
            }
        }
        self.amount = match self.density {
            MineralDensity::Low => MINERAL_DENSITY_LOW_VALUE,
            MineralDensity::Medium => MINERAL_DENSITY_MEDIUM_VALUE,
            MineralDensity::High => MINERAL_DENSITY_HIGH_VALUE,
            MineralDensity::Ultra => MINERAL_DENSITY_ULTRA_VALUE,
        };
        self.refresh_timer = tick + MINERAL_REGEN_TIME as u128;
    }
}
