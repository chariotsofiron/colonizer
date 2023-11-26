//! Parses colonist.io chat messages
//! Main module with colonist.io specific behavior.
//! Takes in html or text messages, parses them, and returns a list of messages
//! Keeps track of players and maps them to IDs.
use crate::{
    card_tracker::MAX_PLAYERS,
    models::{hand::Hand, item::Item, resource::Resource},
};
use regex::Regex;
use scraper::{Html, Selector};
use std::collections::HashMap;

pub type Player = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A player purchases an item.
    Purchase(Player, Item),
    /// Player acquires cards.
    Acquire(Player, Hand),
    /// Player rolls dice
    Roll(Player, u8),
    /// Player moves robber
    MoveRobber(Player, Resource),
    /// Rob where we don't know what was stolen
    Steal(Player, Player),
    /// Rob where we know what was stolen
    StealKnown(Player, Player, Resource),
    /// Player offers a trade
    OfferTrade {
        player: Player,
        offer: Hand,
        request: Hand,
    },
    /// Trade occurs
    AcceptTrade {
        player: Player,
        offer: Hand,
        request: Hand,
        counterparty: Player,
    },
    /// Player discards hand on 7
    Discard(Player, Hand),
    /// Year of plenty card
    YearOfPlenty(Player, Hand),
    /// Player trades with bank / port
    BankTrade {
        player: Player,
        offer: Hand,
        request: Hand,
    },
    /// Monopoly card
    Monopoly(Player, u8, Resource),
}

fn parse_color(text: &str) -> (u8, u8, u8) {
    let pattern = Regex::new(r"(\d+), (\d+), (\d+)").unwrap();
    let captures = pattern.captures(text).unwrap();
    let r = captures[1].parse::<u8>().unwrap();
    let g = captures[2].parse::<u8>().unwrap();
    let b = captures[3].parse::<u8>().unwrap();
    (r, g, b)
}

pub struct Parser {
    pub players: HashMap<String, usize>,
    /// Colors for each player
    colors: [(u8, u8, u8); MAX_PLAYERS],
}

impl Parser {
    pub fn new() -> Self {
        Self {
            players: HashMap::with_capacity(MAX_PLAYERS),
            colors: [(255, 255, 255); MAX_PLAYERS],
        }
    }

    pub fn players(&self) -> Vec<(String, usize, (u8, u8, u8))> {
        self.players
            .iter()
            .map(|(name, &id)| (name.clone(), id, self.colors[id]))
            .collect()
    }

    pub fn with_username(username: String) -> Self {
        // when we encounter `you` in a message, it will be given the same id as `username`
        let players = HashMap::from_iter([
            ("you".to_owned(), 0usize),
            ("You".to_owned(), 0),
            (username, 0),
        ]);
        Self {
            players,
            colors: [(0, 0, 0); MAX_PLAYERS],
        }
    }

    /// Returns the index for a given player
    /// If the player is not in the tracker, it will be added
    fn player_idx(&mut self, name: &str) -> usize {
        // max value of player map
        let i = self.players.iter().map(|(_, &v)| v + 1).max().unwrap_or(0);
        let index = *self.players.entry(name.to_owned()).or_insert(i);
        index
    }

    /// Returns the name of the player with the given id
    pub fn player_name(&self, id: usize) -> String {
        self.players
            .iter()
            .find(|(_, &v)| v == id)
            .map(|(k, _)| k.clone())
            .unwrap_or_else(|| format!("Player {}", id))
    }

    pub fn parse_html(&mut self, document: Html) -> Vec<Event> {
        let msg_selector = Selector::parse(".message_post").unwrap();
        let img_selector = Selector::parse("img").unwrap();

        let mut messages = Vec::new();
        for message in document.select(&msg_selector) {
            let color = parse_color(message.value().attr("style").unwrap());
            let mut text = message.inner_html();
            // replace images with their alt-text
            for img in message.select(&img_selector) {
                let alt_text = format!("{} ", img.value().attr("alt").unwrap());
                text = text.replace(&img.html(), &alt_text);
            }

            if let Ok(msg) = self.parse_line(&text) {
                if let Event::Acquire(player, _) = msg {
                    self.colors[player] = color;
                }
                messages.push(msg);
            }
        }
        messages
    }

    pub fn parse_text(&mut self, text: &str) -> Vec<Event> {
        text.lines()
            .filter_map(|line| self.parse_line(line).ok())
            .collect()
    }

    fn parse_line(&mut self, value: &str) -> Result<Event, ()> {
        let text = value.replace(':', "");
        if let Some(captures) = patterns::ACQUIRE.captures(&text) {
            let player = self.player_idx(&captures[1]);
            let hand = Hand::try_from(&captures[2])?;
            Ok(Event::Acquire(player, hand))
        } else if let Some(captures) = patterns::DISCARD.captures(&text) {
            let player = self.player_idx(&captures[1]);
            let hand = Hand::try_from(&captures[2])?;
            Ok(Event::Discard(player, hand))
        } else if let Some(captures) = patterns::PURCHASE.captures(&text) {
            let player = self.player_idx(&captures[1]);
            let item = Item::try_from(&captures[2])?;
            Ok(Event::Purchase(player, item))
        } else if let Some(captures) = patterns::STEAL.captures(&text) {
            let player = self.player_idx(&captures[1]);
            let victim = self.player_idx(&captures[3]);
            match Resource::try_from(&captures[2]) {
                Ok(resource) => Ok(Event::StealKnown(player, victim, resource)),
                Err(_) => Ok(Event::Steal(player, victim)),
            }
        } else if let Some(captures) = patterns::TRADE_OFFER.captures(&text) {
            Ok(Event::OfferTrade {
                player: self.player_idx(&captures[1]),
                offer: Hand::try_from(&captures[4])?,
                request: Hand::try_from(&captures[5])?,
            })
        } else if let Some(captures) = patterns::TRADE.captures(&text) {
            Ok(Event::AcceptTrade {
                player: self.player_idx(&captures[1]),
                offer: Hand::try_from(&captures[2])?,
                request: Hand::try_from(&captures[3])?,
                counterparty: self.player_idx(&captures[4]),
            })
        } else if let Some(captures) = patterns::YEAROFPLENTY.captures(&text) {
            let player = self.player_idx(&captures[1]);
            let hand = Hand::try_from(&captures[2])?;
            Ok(Event::YearOfPlenty(player, hand))
        } else if let Some(captures) = patterns::TRADE_BANK.captures(&text) {
            Ok(Event::BankTrade {
                player: self.player_idx(&captures[1]),
                offer: Hand::try_from(&captures[2])?,
                request: Hand::try_from(&captures[3])?,
            })
        } else if let Some(captures) = patterns::DICEROLL.captures(&text) {
            let player = self.player_idx(&captures[1]);
            let dice1: u8 = captures[2].parse().map_err(|__| ())?;
            let dice2: u8 = captures[3].parse().map_err(|__| ())?;
            Ok(Event::Roll(player, dice1 + dice2))
        } else if let Some(captures) = patterns::MONOPOLY.captures(&text) {
            let player = self.player_idx(&captures[1]);
            let count: u8 = captures[2].parse().map_err(|__| ())?;
            let resource = Resource::try_from(&captures[3])?;
            Ok(Event::Monopoly(player, count, resource))
        } else {
            Err(())
        }
    }
}

/// Regex partterns for parsing game chat
#[allow(clippy::unwrap_used)]
mod patterns {
    use lazy_static::lazy_static;
    use regex::Regex;

    /// Player name
    const NAME: &str = r"(?:Guest|bot|User)?(\w+(?:#\d+)?)";
    /// Cards
    const CARDS: &str = r"((?:(?:lumber|brick|wool|grain|ore|card) ?)+)";
    /// Item
    const ITEM_PTTN: &str = r"(road|settlement|city|development card)";
    /// Dice
    const DICE: &str = r"(?:dice_([1-6]))";

    macro_rules! pat {
        ($input:expr) => {{
            Regex::new(&format!($input)).unwrap()
        }};
    }

    lazy_static! {
        pub static ref ACQUIRE: Regex = pat!(r"{NAME} (?:got|received starting resources) {CARDS}");
        pub static ref DISCARD: Regex = pat!(r"{NAME} discarded {CARDS}");
        pub static ref PURCHASE: Regex = pat!(r"{NAME} (?:built a|bought) {ITEM_PTTN}");
        pub static ref STEAL: Regex = pat!(r"{NAME} stole {CARDS} from? {NAME}");
        pub static ref TRADE_OFFER: Regex =
            pat!(r"{NAME} wants to give( {NAME})? {CARDS} for {CARDS}");
        pub static ref TRADE: Regex = pat!(r"{NAME} traded {CARDS} for {CARDS} with {NAME}");
        pub static ref YEAROFPLENTY: Regex = pat!(r"{NAME} took from bank {CARDS}");
        pub static ref TRADE_BANK: Regex = pat!(r"{NAME} gave bank {CARDS} and took {CARDS}");
        pub static ref MONOPOLY: Regex = pat!(r"{NAME} stole (\d+) {CARDS}");
        pub static ref DICEROLL: Regex = pat!(r"{NAME} rolled {DICE} {DICE}");
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{hand::Hand, item::Item, resource::Resource};

    use super::{Event, Parser};

    #[test]
    fn test_parse() {
        let test_cases = [
            (
                "GuestLookaside received starting resources: grainlumberwool",
                Event::Acquire(0, Hand::from([1, 0, 1, 1, 0])),
            ),
            (
                "GuestPoten77a got: brick",
                Event::Acquire(1, Hand::from([0, 1, 0, 0, 0])),
            ),
            (
                "GuestAleBalu discarded: lumberwoolwoolgrain",
                Event::Discard(2, Hand::from([1, 0, 2, 1, 0])),
            ),
            (
                "GuestPetrovski built a city: +1 VP",
                Event::Purchase(3, Item::City),
            ),
            (
                "GuestYou stole: ore from: Petrovski",
                Event::StealKnown(0, 3, Resource::Ore),
            ),
            (
                "GuestPetrovski stole: brick from you",
                Event::StealKnown(3, 0, Resource::Brick),
            ),
            ("UserPetrovski stole card from: AleBalu", Event::Steal(3, 2)),
            (
                "GuestLookaside wants to give: woolwool for: grain",
                Event::OfferTrade {
                    player: 0,
                    offer: Hand::from([0, 0, 2, 0, 0]),
                    request: Hand::from([0, 0, 0, 1, 0]),
                },
            ),
            (
                "GuestPetrovski wants to give: Lookaside: brick for: wool",
                Event::OfferTrade {
                    player: 3,
                    offer: Hand::from([0, 1, 0, 0, 0]),
                    request: Hand::from([0, 0, 1, 0, 0]),
                },
            ),
            (
                "UserLookaside traded: lumber for: ore with: Petrovski",
                Event::AcceptTrade {
                    player: 0,
                    counterparty: 3,
                    offer: Hand::from([1, 0, 0, 0, 0]),
                    request: Hand::from([0, 0, 0, 0, 1]),
                },
            ),
            (
                "GuestLookaside took from bank: ore ore",
                Event::YearOfPlenty(0, Hand::from([0, 0, 0, 0, 2])),
            ),
            (
                "GuestPetrovski gave bank: lumberlumberlumberlumber and took ore",
                Event::BankTrade {
                    player: 3,
                    offer: Hand::from([4, 0, 0, 0, 0]),
                    request: Hand::from([0, 0, 0, 0, 1]),
                },
            ),
            ("GuestLookaside rolled: dice_2 dice_6", Event::Roll(0, 8)),
            (
                "UserPetrovski stole 6: ore",
                Event::Monopoly(3, 6, Resource::Ore),
            ),
        ];

        let mut parser = Parser::with_username("Lookaside".to_owned());
        for (input, expected) in test_cases.iter() {
            let parsed = parser.parse_line(input).unwrap();
            assert_eq!(parsed, *expected);
        }
    }
}
