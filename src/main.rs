use eframe::{egui, NativeOptions};
use egui::{Color32, RichText};
use resource::N_RESOURCES;
use state::State;
mod card_tracker;
mod hand;
mod html_parser;
mod resource;
mod state;

fn main() {
    let username: String = std::env::args()
        .nth(1)
        .expect("Please provide your colonist.io username as the first argument");

    eframe::run_native(
        "Colonizer",
        NativeOptions {
            always_on_top: true,
            initial_window_size: Some(egui::Vec2::new(560.0, 140.0)),
            ..Default::default()
        },
        Box::new(|_cc| Box::new(MyApp::new(username))),
    );
}

struct MyApp {
    state: State,
    last_update: std::time::Instant,
    cdp: cdp_client::Browser,
}

impl MyApp {
    fn new(username: String) -> Self {
        let browser = cdp_client::Browser::new("http://localhost:9222/json")
            .expect("Unable to connect to Chrome");
        Self {
            state: State::new(username),
            last_update: std::time::Instant::now() - std::time::Duration::from_secs(1),
            cdp: browser,
        }
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

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.last_update.elapsed() > std::time::Duration::from_secs(1) {
            let html = self
                .cdp
                .evaluate(r#"document.getElementById("game-log-text").innerHTML"#)
                .expect("Unable to read game log");
            // let html = std::fs::read_to_string("games/game3.html").unwrap();
            self.state.update(&html);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
            egui::Grid::new("id1").striped(true).show(ui, |ui| {
                ui.label(egui::RichText::new("Player").color(Color32::LIGHT_BLUE));
                ui.label(egui::RichText::new("Lumber").color(Color32::from_rgb(95, 185, 60)));
                ui.label(egui::RichText::new("Brick").color(Color32::from_rgb(185, 100, 90)));
                ui.label(egui::RichText::new("Wool").color(Color32::from_rgb(140, 200, 60)));
                ui.label(egui::RichText::new("Grain").color(Color32::from_rgb(210, 150, 70)));
                ui.label(egui::RichText::new("Ore").color(Color32::from_rgb(140, 175, 160)));
                ui.label("Total");
                ui.end_row();

                let data = self.state.build_table();

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
                    ui.label(egui::RichText::new(name).color(color));
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
                ui.label(format!("{:>5}", self.state.len()));
            });
        });
    }
}
