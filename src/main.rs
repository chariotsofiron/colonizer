mod card_tracker;
mod chat;
mod controller;
mod gui;
mod models;

use crate::{chat::Parser, controller::Controller, gui::MyApp};

use eframe::NativeOptions;

fn main() {
    // println!("Hello, world!");

    // let text = std::fs::read_to_string("dataset2/000.txt").unwrap();

    // let mut parser = Parser::new();
    // let msgs = parser.parse_text(&text);
    // println!("msgs: {:?}", parser.players);

    // let mut controller = Controller::new();
    // for msg in msgs {
    //     controller.process_event(msg);
    // }

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
