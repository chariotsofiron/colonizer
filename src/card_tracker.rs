use crate::{
    hand::Hand,
    resource::{Resource, N_RESOURCES},
};
use std::collections::HashMap;

pub const fn gcd(a: u32, b: u32) -> u32 {
    // Use Stein's algorithm
    let mut m = a;
    let mut n = b;
    if m == 0 || n == 0 {
        return m | n;
    }

    // find common factors of 2
    let shift = (m | n).trailing_zeros();

    // divide n and m by 2 until odd
    m >>= m.trailing_zeros();
    n >>= n.trailing_zeros();

    while m != n {
        if m > n {
            m -= n;
            m >>= m.trailing_zeros();
        } else {
            n -= m;
            n >>= n.trailing_zeros();
        }
    }
    m << shift
}

pub const MAX_PLAYERS: usize = 6;
pub type State = [Hand; MAX_PLAYERS];

pub struct CardTracker {
    /// A list of all states and their frequency
    states: Vec<(State, u32)>,
}

impl Default for CardTracker {
    fn default() -> Self {
        Self {
            states: vec![(State::default(), 1)],
        }
    }
}

impl CardTracker {
    /// Returns the number of states. This is the number of possible
    /// combinations of cards that are consistent with the game log.
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// Removes states where player does not have that many cards.
    pub fn know_has(&mut self, player: usize, cards: Hand) {
        self.states.retain(|(state, _)| {
            state[player]
                .values()
                .zip(cards.values())
                .all(|(a, b)| a >= b)
        });
        assert!(!self.states.is_empty(), "Arrived at inconsistent state!");
    }

    /// Adds a `Hand` of cards to every state for a player
    pub fn add(&mut self, player: usize, cards: Hand) {
        for (state, _) in &mut self.states {
            for (card, count) in cards {
                state[player][card] += count;
            }
        }
    }

    /// Removes a `Hand` of cards from every state for a player
    pub fn remove(&mut self, player: usize, cards: Hand) {
        self.know_has(player, cards);
        for (state, _) in &mut self.states {
            for (card, count) in cards {
                state[player][card] -= count;
            }
        }
    }

    /// Handles a rob involving two players where we don't know what card was taken
    pub fn rob(&mut self, robber: usize, victim: usize) {
        let mut results = HashMap::new();
        for (state, count) in &self.states {
            for (card, num) in state[victim].into_iter().filter(|(_, c)| *c > 0) {
                let mut s_new = *state;
                s_new[robber][card] += 1;
                s_new[victim][card] -= 1;
                *results.entry(s_new).or_insert(0) += u32::from(num) * count;
            }
        }
        // normalize the counts
        let gcd = results.values().fold(0, |a, b| gcd(a, *b));
        self.states = results.into_iter().map(|(a, b)| (a, b / gcd)).collect();
    }

    /// Handles a monopoly event where a `player` steals `count` `card`s from other players.
    pub fn monopoly(&mut self, player: usize, card: Resource, count: u8) {
        // remove all states where the count doesn't match the total
        self.states.retain(|(state, _)| {
            state
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != player) // don't count the monopoler
                .map(|(_, hand)| hand[card])
                .sum::<u8>()
                == count
        });

        // update the states with the new count
        for (state, _) in &mut self.states {
            for (i, hand) in state.iter_mut().enumerate() {
                if i == player {
                    hand[card] += count;
                } else {
                    hand[card] = 0;
                }
            }
        }
    }

    /// Computes the expected value for the number of cards each player has
    fn expected(&self) -> [[f64; N_RESOURCES]; MAX_PLAYERS] {
        let n_states = f64::from(self.states.iter().map(|(_, count)| *count).sum::<u32>());
        let mut expected = <[[f64; N_RESOURCES]; MAX_PLAYERS]>::default();
        for &(state, count) in &self.states {
            for (player, cards) in state.iter().enumerate() {
                for (card, num) in cards.into_iter() {
                    expected[player][usize::from(card)] +=
                        f64::from(num) * f64::from(count) / n_states;
                }
            }
        }
        expected
    }

    /// Computes the minimum number of cards each player could have
    fn sure(&self) -> State {
        let mut sure = self.states[0].0; // there should be at least one
        for (state, _) in &self.states {
            for (player, cards) in state.iter().enumerate() {
                for (card, num) in cards.into_iter() {
                    sure[player][card] = std::cmp::min(sure[player][card], num);
                }
            }
        }
        sure
    }

    /// Computes the expected and minimum value for each card of the player
    /// (sure, expected, rob chance)
    pub fn table(&self) -> [[(u8, f64, f64); N_RESOURCES]; MAX_PLAYERS] {
        let mut table: [[(u8, f64, f64); N_RESOURCES]; MAX_PLAYERS] = Default::default();
        for (i, (sure, expected)) in self
            .sure()
            .into_iter()
            .zip(self.expected().into_iter())
            .enumerate()
        {
            let total = expected.iter().sum::<f64>();
            for (j, (sure, expected)) in sure.values().zip(expected.into_iter()).enumerate() {
                let rob_chance = if total == 0.0 { 0.0 } else { expected / total };
                table[i][j] = (sure, expected, rob_chance);
            }
        }
        table
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_add() {
        let mut tracker = CardTracker::default();
        tracker.add(0, Hand::from(Resource::Brick));
        assert_eq!(tracker.states[0].0[0][Resource::Brick], 1);
    }

    #[test]
    fn test_rob_unknown() {
        let mut tracker = CardTracker::default();

        tracker.add(0, Hand::from([5, 7, 9, 13, 15]));
        tracker.add(1, Hand::from([12, 11, 6, 5, 3]));

        tracker.rob(1, 0);
        tracker.rob(1, 0);
        tracker.rob(1, 0);
        tracker.rob(0, 1);
        tracker.rob(0, 1);
        tracker.rob(0, 1);

        assert_eq!(tracker.states.len(), 471);
    }
}
