//! Colonist controller
//! - accepts a game event
//! - updates the various trackers

use crate::{card_tracker::{CardTracker, MAX_PLAYERS}, chat::Event, models::{hand::Hand, resource::N_RESOURCES}};

pub struct Controller {
    cards: CardTracker,
    // dice_tracker: DiceTracker,
    // devcard_tracker: DevCardTracker,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            cards: CardTracker::default(),
        }
    }

    pub fn table(&self) -> [[(u8, f64, f64); N_RESOURCES]; MAX_PLAYERS] {
        self.cards.table()
    }

    pub fn process_event(&mut self, message: &Event) {
        match *message {
            Event::Acquire(player, hand) => self.cards.add(player, hand),
            Event::Discard(player, hand) => self.cards.remove(player, hand),
            Event::Purchase(player, item) => self.cards.remove(player, item.cost()),
            Event::Steal(robber, victim) => self.cards.rob(robber, victim),
            Event::StealKnown(robber, victim, card) => {
                let hand = Hand::from(card);
                self.cards.remove(victim, hand);
                self.cards.add(robber, hand);
            }
            Event::OfferTrade {
                player,
                offer,
                request: _,
            } => {
                self.cards.know_has(player, offer);
            }
            Event::AcceptTrade {
                player,
                counterparty,
                offer,
                request,
            } => {
                self.cards.add(player, request);
                self.cards.remove(counterparty, request);
                self.cards.add(counterparty, offer);
                self.cards.remove(player, offer);
            }
            Event::YearOfPlenty(player, hand) => self.cards.add(player, hand),
            Event::BankTrade {
                player,
                offer,
                request,
            } => {
                self.cards.remove(player, offer);
                self.cards.add(player, request);
            }
            Event::Monopoly(player, count, resource) => {
                self.cards.monopoly(player, resource, count);
            }
            Event::Roll(_, _) | Event::MoveRobber(_, _) => {}
        }
    }
}
