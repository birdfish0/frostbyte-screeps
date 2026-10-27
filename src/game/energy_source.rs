use log::error;

use crate::utils::event_queue::{EventCallback, QueuedEvent};

use super::*;

#[derive(PartialEq, Eq, Copy, Clone, Debug, Default)]
pub struct EnergySource {
    pub id: u64,
    pub pos: RoomPos,
    pub amount: u32,
    pub max_amount: u32,
    pub refresh_timer: u128,
}

pub const ENERGY_SOURCE_REFRESH_TICKS: u128 = 3000;

pub struct EnergySourceRefreshEvent {
    pub id: u64,
}

impl EventCallback for EnergySourceRefreshEvent {
    fn callback(&self, game_state: &mut GameState, event_queue: &mut EventQueue) {
        if let Some(me) = game_state.energy_sources.get_mut(&self.id) {
            me.amount = me.max_amount;
            event_queue.enqueue(QueuedEvent {
                tick: game_state.tick + ENERGY_SOURCE_REFRESH_TICKS,
                callback: Box::new(EnergySourceRefreshEvent { id: self.id }),
            });
        } else {
            error!(
                "Couldn't find energy source of id {} queued for refresh.",
                self.id
            );
        }
    }
}
