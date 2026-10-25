//! Provides the [MineralSource] struct and its implementation, as well as related definitions like [MineralDensity].
//!
//! Structs and functions related to scheduling mineral source refresh are also defined here.

use log::error;
use rand::Rng;
use utils::random::random_choice;

use crate::{
    event_queue::{EventCallback, QueuedEvent},
    game::*,
};

/// Density of a [MineralSource]. This determines the amount of minerals it will contain on next refresh.
///
/// See [crate::consts::MINERAL_DENSITY_LOW_VALUE], [crate::consts::MINERAL_DENSITY_MEDIUM_VALUE],
/// [crate::consts::MINERAL_DENSITY_HIGH_VALUE], and [crate::consts::MINERAL_DENSITY_ULTRA_VALUE]
/// for the exact values of the amount of minerals for each density level.
#[derive(PartialEq, Eq, Copy, Clone, Debug, Default)]
pub enum MineralDensity {
    #[default]
    Low,
    Medium,
    High,
    Ultra,
}

/// Object representing a mineral source within a room. Contains a limited amount of minerals that can be harvested through an
/// [Extractor][StructureType::Extractor] [Structure] and a [Creep] with a [Work][CreepPart::Work] part (or upgraded Work part).
#[derive(Clone, Debug)]
pub struct MineralSource {
    /// Unique identifier of the MineralSource.
    pub id: u64,
    /// Type of mineral the source provides. It should be one of
    /// [ResourceType::Hydrogen],
    /// [ResourceType::Oxygen],
    /// [ResourceType::Utrium],
    /// [ResourceType::Keanium],
    /// [ResourceType::Lemergium],
    /// [ResourceType::Zynthium],
    /// [ResourceType::Catalyst].
    pub mineral_type: ResourceType,
    /// The position and [Room] of the mineral.
    pub pos: RoomPos,
    /// The amount of minerals currently left in this source.
    pub amount: u32,
    /// The [tick][field@GameState::tick] when this mineral will refresh.
    pub refresh_timer: u128,
    /// Density of the mineral source that will be used upon next refresh.
    pub density: MineralDensity,
}

/// Scheduled event that refreshes a [MineralSource] and changes its [density][MineralSource::density] if need be.
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct MineralRefreshEvent {
    /// ID of the [MineralSource] to refresh.
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
    /// Refresh the mineral source, using the provided random number generator to determine whether to change the density and what to change it to,
    /// then updating [Self::refresh_timer] according to `tick` and [crate::consts::MINERAL_REGEN_TIME].
    pub fn refresh<T>(&mut self, rng: &mut T, tick: u128)
    where
        T: Rng,
    {
        self.amount = match self.density {
            MineralDensity::Low => MINERAL_DENSITY_LOW_VALUE,
            MineralDensity::Medium => MINERAL_DENSITY_MEDIUM_VALUE,
            MineralDensity::High => MINERAL_DENSITY_HIGH_VALUE,
            MineralDensity::Ultra => MINERAL_DENSITY_ULTRA_VALUE,
        };
        if self.density == MineralDensity::Low
            || self.density == MineralDensity::Ultra
            || rng.random_ratio(MINERAL_DENSITY_CHANGE_CHANCE_PERCENT, 100)
        {
            self.density = match self.density {
                MineralDensity::Low => random_choice(
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
                )
                .unwrap_or_else(|err| {
                    error!("Failed to generate new mineral density: {}", err);
                    Default::default()
                }),
                MineralDensity::Medium => random_choice(
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
                )
                .unwrap_or_else(|err| {
                    error!("Failed to generate new mineral density: {}", err);
                    Default::default()
                }),
                MineralDensity::High => random_choice(
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
                )
                .unwrap_or_else(|err| {
                    error!("Failed to generate new mineral density: {}", err);
                    Default::default()
                }),
                MineralDensity::Ultra => random_choice(
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
                )
                .unwrap_or_else(|err| {
                    error!("Failed to generate new mineral density: {}", err);
                    Default::default()
                }),
            }
        }
        self.refresh_timer = tick + MINERAL_REGEN_TIME as u128;
    }
}
