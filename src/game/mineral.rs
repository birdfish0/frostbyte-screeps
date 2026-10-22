use crate::game::*;

impl MineralSource {
    pub fn refresh(&mut self, game_state: &mut GameState) {
        if self.density == MineralDensity::Low
            || self.density == MineralDensity::Ultra
            || game_state
                .random
                .random_ratio(MINERAL_DENSITY_CHANGE_CHANCE_PERCENT, 100)
        {
            self.density = match self.density {
                MineralDensity::Low => utils::random_choice(
                    &mut game_state.random,
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
                    &mut game_state.random,
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
                    &mut game_state.random,
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
                    &mut game_state.random,
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
        self.refresh_timer = game_state.tick + MINERAL_REGEN_TIME as u128;
    }
}
