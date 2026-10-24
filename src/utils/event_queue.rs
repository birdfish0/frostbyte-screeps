//! Contains the [EventQueue] struct and its implementation.

use std::{any::Any, collections::VecDeque, fmt::Debug};

use crate::{game::game_state::GameState, utils::event_queue};

/// A trait that contains a function which is contained in [QueuedEvent::callback] and called in [EventQueue::tick]. It should not be called elsewhere.
pub trait EventCallback {
    /// The function to be called when the event's tick value is reached.
    ///
    /// It will be called on `&self` with a mutable reference to the current [GameState] as the only argument.
    fn callback(&self, game_state: &mut GameState, event_queue: &mut EventQueue);
}

pub struct QueuedEvent {
    /// The tick value at which the event's callback should be called.
    pub tick: u128,
    /// The callback that should be called when the event's tick value is reached.
    pub callback: Box<dyn EventCallback>,
}

impl Debug for QueuedEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "QueuedEvent {{ tick: {}, callback: {:?} }}",
            self.tick,
            self.callback.type_id()
        )
    }
}

/// A queue of events that need to be called at a certain tick value.
///
/// Events are represented as [QueuedEvent]s in a sorted [VecDeque], from lowest to highest [tick][QueuedEvent::tick] value.
#[derive(Debug)]
pub struct EventQueue {
    /// The deque that stores the queued events, sorted from lowest to highest [tick][QueuedEvent::tick] value.
    pub events: VecDeque<QueuedEvent>,
}

impl EventQueue {
    /// Constructor. Sets the [events][EventQueue::events] field directly.
    pub fn from_deque(events: VecDeque<QueuedEvent>) -> Self {
        Self { events }
    }

    /// Constructor. Creates the struct with an empty deque.
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
        }
    }

    /// Add an event to the queue.
    ///
    /// The event will be inserted into the queue using a binary search on [`tick`][QueuedEvent::tick] to ensure the queue remains sorted.
    pub fn enqueue(&mut self, event: QueuedEvent) {
        let idx = match self.events.binary_search_by_key(&event.tick, |ev| ev.tick) {
            Ok(idx) => idx,
            Err(idx) => idx,
        };
        self.events.insert(idx, event);
    }

    /// Update the event queue with the given tick value.
    ///
    /// All events within the queue that have a [`tick`][QueuedEvent::tick] value less than the one provided in the arguments
    /// will have their [`callback`][QueuedEvent::callback] called, and then be removed from the queue.
    pub fn tick(&mut self, tick: u128, game_state: &mut GameState) {
        while let Some(evt) = self.events.pop_front_if(|ev| ev.tick <= tick) {
            evt.callback.callback(game_state, self);
        }
    }
}
