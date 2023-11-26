use crate::{chat::Parser, controller::Controller, models::resource::N_RESOURCES};
use chromia::client::Client;
use egui::{Color32, RichText};
use scraper::Html;

pub struct MyApp {
    state: Controller,
    chat: Parser,
    last_update: std::time::Instant,
    cdp: Client,
    last_msg: usize,
}

const UPDATE: std::time::Duration = std::time::Duration::from_millis(250);
pub type Record = (String, Color32, [(u8, f64, f64); N_RESOURCES]);

impl MyApp {
    pub fn new(username: String) -> Self {
        let cdp = Client::from_page(9222, "colonist.io");
        Self {
            state: Controller::new(),
            chat: Parser::with_username(username),
            last_update: std::time::Instant::now() - UPDATE,
            cdp,
            last_msg: 0,
        }
    }

    fn update_state(&mut self) {
        // get new chat messages
        let response = self
            .cdp
            .execute("document.getElementById('game-log-text').innerHTML");
        // parse it as html
        let document = Html::parse_document(response["value"].as_str().unwrap());
        // parse the html into chat messages
        let messages = self.chat.parse_html(document);
        for msg in messages.iter().skip(self.last_msg) {
            self.state.process_event(msg);
        }
        self.last_msg = messages.len();
    }

    pub fn build_table(&self) -> Vec<Record> {
        // associate the player names with the table
        let table = self.state.table();
        let mut result: Vec<Record> = Vec::new();
        for (name, id, (r, g, b)) in &self.chat.players() {
            result.push((name.to_string(), Color32::from_rgb(*r, *g, *b), table[*id]));
        }
        result
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.last_update.elapsed() > UPDATE {
            self.update_state();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
            egui::Grid::new("id1").striped(true).show(ui, |ui| {
                ui.label(RichText::new("Player").color(Color32::LIGHT_BLUE));
                ui.label(RichText::new("Lumber").color(Color32::from_rgb(95, 185, 60)));
                ui.label(RichText::new("Brick").color(Color32::from_rgb(185, 100, 90)));
                ui.label(RichText::new("Wool").color(Color32::from_rgb(140, 200, 60)));
                ui.label(RichText::new("Grain").color(Color32::from_rgb(210, 150, 70)));
                ui.label(RichText::new("Ore").color(Color32::from_rgb(140, 175, 160)));
                ui.label("Total");
                ui.end_row();

                let data = self.build_table();

                // compute the best odds of getting each resource
                let mut best = [0.0f64; N_RESOURCES];
                for (_, _, cards) in &data {
                    for (i, &(_, _, rob_chance)) in cards.iter().enumerate() {
                        best[i] = best[i].max(rob_chance);
                    }
                }
                // running totals of each resource type
                let mut resource_totals = [0.0; N_RESOURCES];
                for (name, color, cards) in data {
                    ui.label(RichText::new(name).color(color));
                    let mut player_total: f64 = 0.0;
                    for (i, &(sure, expected, rob_chance)) in cards.iter().enumerate() {
                        player_total += expected; // row wise
                        resource_totals[i] += expected; // column wise
                        let color = if rob_chance == best[i] {
                            Color32::from_rgb(95, 185, 60)
                        } else {
                            Color32::WHITE
                        };

                        ui.label(
                            RichText::new(fmt_resource(sure, expected, rob_chance)).color(color),
                        );
                    }
                    ui.label(format!("{player_total:>5.2}"));
                    ui.end_row();
                }
                ui.label("Totals:");
                for total in &resource_totals {
                    ui.label(format!("{total:>5.2}"));
                }
                // ui.label(format!("{:>5}", self.state.len()));
            });
        });
    }
}

/// custom formatting based on the info that needs to be displayed
/// each field should be 11 chars long
fn fmt_resource(sure: u8, expected: f64, rob_chance: f64) -> String {
    let chance = fmt_rob_chance(rob_chance);
    let unsure = expected - f64::from(sure);

    if rob_chance == 0.0 {
        "           ".to_owned()
    } else if unsure == 0.0 {
        format!("{chance} {sure:>2}     ")
    } else if unsure < 1.0 {
        format!("{chance} {:>5.2}  ", f64::from(sure) + unsure)
    } else {
        format!("{chance} {sure:>2}+{unsure:>4.2}")
    }
}

// Formats the rob chance as a probability into a percentage
fn fmt_rob_chance(rob_chance: f64) -> String {
    let percentage = (rob_chance * 100.0).round() as u8;
    if percentage == 100 {
        // pretty easy to see that it's the only card available
        "   ".to_owned()
    } else if percentage < 10 {
        format!(".0{percentage:<1}")
    } else {
        format!(
            ".{:<2}",
            if percentage % 10 == 0 {
                percentage / 10
            } else {
                percentage
            }
        )
    }
}
