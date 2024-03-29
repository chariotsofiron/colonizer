//! Parses game lock
//! Updates different trackers
use std::collections::HashMap;

use crate::card_tracker::MAX_PLAYERS;
use crate::html_parser;
use crate::resource::{Resource, N_RESOURCES};
use crate::{card_tracker::CardTracker, hand::Hand};
use egui::Color32;
use lazy_static::lazy_static;
use regex::Regex;

const NAME: &str = r"(\w+(?:#\d+)?)";
const CARDS: &str = r"((?:(?:lumber|brick|wool|grain|ore|card) ?)+)";
const ITEM_PTTN: &str = r"(road|settlement|city|development card)";

type Callback = fn(&mut State, &[&str]) -> ();
pub type Record = (String, Color32, [(u8, f64, f64); N_RESOURCES]);

lazy_static! {
    static ref PATTERNS: [(Regex, Callback); 9] = [
        (
            Regex::new(&format!(
                r"{NAME} (?:got|received starting resources) {CARDS}"
            ))
            .unwrap(),
            State::handle_receive,
        ),
        (
            Regex::new(&format!(r"{NAME} discarded {CARDS}")).unwrap(),
            State::handle_discard,
        ),
        (
            Regex::new(&format!(r"{NAME} (?:built a|bought) {ITEM_PTTN}")).unwrap(),
            State::handle_purchase,
        ),
        (
            Regex::new(&format!(r"{NAME} stole {CARDS} from {NAME}")).unwrap(),
            State::handle_rob,
        ),
        (
            Regex::new(&format!(r"{NAME} wants to give {CARDS} for {CARDS}")).unwrap(),
            State::handle_trade_offer,
        ),
        (
            Regex::new(&format!(r"{NAME} traded {CARDS} for {CARDS} with {NAME}")).unwrap(),
            State::handle_trade,
        ),
        (
            Regex::new(&format!(r"{NAME} took from bank {CARDS}")).unwrap(),
            State::handle_year_of_plenty,
        ),
        (
            Regex::new(&format!(r"{NAME} gave bank {CARDS} and took {CARDS}")).unwrap(),
            State::handle_bank_trade,
        ),
        (
            Regex::new(&format!(r"{NAME} stole (\d+) {CARDS}")).unwrap(),
            State::handle_monopoly,
        ),
    ];
}

pub struct State {
    /// The player's username
    username: String,
    /// Maps player names to their index
    players: HashMap<String, usize>,
    colors: HashMap<String, Color32>,
    last_line: usize,
    card_tracker: CardTracker,
    // dice_tracker: DiceTracker,
    // devcard_tracker: DevCardTracker,
}

impl State {
    pub fn new(username: String) -> Self {
        Self {
            username,
            players: HashMap::new(),
            colors: Default::default(),
            last_line: 0,
            card_tracker: CardTracker::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.card_tracker.len()
    }

    fn normalize(&self, s: &str) -> String {
        // remove consecutive spaces and newlines
        let s = s
            .trim()
            .replace(':', "")
            .replace("you", &self.username)
            .replace("You", &self.username)
            .split([' ', '\n'])
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        // fix for usernames prefixed with resources
        let re = Regex::new("(from|with|User|Guest) (brick|lumber|wool|grain|ore) ").unwrap();
        let s = re.replace_all(&s, "$1 $2");
        s.into_owned()
    }

    pub fn update(&mut self, html: &str) {
        let lines = html_parser::parse(html);
        let tmp = lines.len();
        for ((r, g, b), line) in lines.into_iter().skip(self.last_line) {
            let line = self.normalize(&line);
            if line.contains("starting") {
                let name = line.split(' ').nth(1).unwrap();
                self.colors
                    .insert(name.to_owned(), Color32::from_rgb(r, g, b));
            }
            self.handle_line(&line);
        }
        self.last_line = tmp;
    }

    /// Returns the index for a given player
    /// If the player is not in the tracker, it will be added
    ///
    /// # Panics
    ///
    /// Panics if the number of players is greater than `MAX_PLAYERS`
    fn get_player_index(&mut self, name: &str) -> usize {
        let i = self.players.len();
        let tmp = *self.players.entry(name.to_owned()).or_insert(i);

        assert!(
            self.players.len() <= MAX_PLAYERS,
            "Too many players! {:?}",
            self.players.keys().collect::<Vec<_>>()
        );
        tmp
    }

    pub fn handle_line(&mut self, line: &str) {
        for (regex, event) in PATTERNS.iter() {
            if let Some(caps) = regex.captures(line) {
                let line = caps
                    .iter()
                    // skip over capture group 0
                    .skip(1)
                    .map(|m| m.unwrap().as_str())
                    .collect::<Vec<_>>();
                event(self, &line);
                return;
            }
        }
    }

    fn handle_receive(&mut self, line: &[&str]) {
        println!("{} got {}", line[0], line[1]);
        let player = self.get_player_index(line[0]);
        self.card_tracker.add(player, Hand::from(line[1]));
    }

    fn handle_discard(&mut self, line: &[&str]) {
        println!("{} discarded {}", line[0], line[1]);
        let player = self.get_player_index(line[0]);
        self.card_tracker.remove(player, Hand::from(line[1]));
    }

    fn handle_purchase(&mut self, line: &[&str]) {
        println!("{} purchased {}", line[0], line[1]);
        let player = self.get_player_index(line[0]);
        let cost = match line[1] {
            "road" => Hand::from([1, 1, 0, 0, 0]),
            "settlement" => Hand::from([1, 1, 1, 1, 0]),
            "city" => Hand::from([0, 0, 0, 2, 3]),
            "development card" => Hand::from([0, 0, 1, 1, 1]),
            _ => panic!("Unknown item: {}", line[1]),
        };
        self.card_tracker.remove(player, cost);
    }

    fn handle_rob(&mut self, line: &[&str]) {
        println!("{} stole {} from {}", line[0], line[1], line[2]);
        let robber = self.get_player_index(line[0]);
        let victim = self.get_player_index(line[2]);

        if line[1] == "card" {
            // we don't know which card was stolen
            self.card_tracker.rob(robber, victim);
        } else {
            // rob involving ourselves, so we know which card was stolen
            self.card_tracker.add(robber, Hand::from(line[1]));
            self.card_tracker.remove(victim, Hand::from(line[1]));
        }
    }

    fn handle_trade_offer(&mut self, line: &[&str]) {
        println!("{} offered {} for {}", line[0], line[1], line[2]);
        let player = self.get_player_index(line[0]);
        self.card_tracker.know_has(player, Hand::from(line[1]));
    }

    fn handle_trade(&mut self, line: &[&str]) {
        println!(
            "{} traded {} for {} with {}",
            line[0], line[1], line[2], line[3]
        );
        let player = self.get_player_index(line[0]);
        let counterparty = self.get_player_index(line[3]);
        let offer = Hand::from(line[1]);
        let request = Hand::from(line[2]);
        self.card_tracker.add(player, request);
        self.card_tracker.remove(counterparty, request);
        self.card_tracker.add(counterparty, offer);
        self.card_tracker.remove(player, offer);
    }

    fn handle_year_of_plenty(&mut self, line: &[&str]) {
        println!("{} took from bank {}", line[0], line[1]);
        let player = self.get_player_index(line[0]);
        self.card_tracker.add(player, Hand::from(line[1]));
    }

    fn handle_bank_trade(&mut self, line: &[&str]) {
        println!("{} gave bank {} for {}", line[0], line[1], line[2]);
        let player = self.get_player_index(line[0]);
        self.card_tracker.remove(player, Hand::from(line[1]));
        self.card_tracker.add(player, Hand::from(line[2]));
    }

    fn handle_monopoly(&mut self, line: &[&str]) {
        println!("{} monopolied {} {}", line[0], line[1], line[2]);
        let player = self.get_player_index(line[0]);
        self.card_tracker.monopoly(
            player,
            Resource::try_from(line[2]).unwrap(),
            line[1].parse().unwrap(),
        );
    }

    pub fn build_table(&self) -> Vec<Record> {
        // associate the player names with the table
        let table = self.card_tracker.table();
        let mut result: Vec<Record> = Vec::new();
        for (name, id) in &self.players {
            result.push((
                name.to_string(),
                *self.colors.get(name).unwrap_or(&Color32::WHITE),
                table[*id],
            ));
        }
        result
    }
}
