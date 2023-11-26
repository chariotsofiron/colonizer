
// use eframe::{egui, NativeOptions};
// use egui::{Color32, RichText};
// use resource::N_RESOURCES;
// use state::State;
// mod card_tracker;
// mod hand;
// mod html_parser;
// mod resource;
// mod state;


// fn main() {
//     let username: String = std::env::args()
//         .nth(1)
//         .expect("Please provide your colonist.io username as the first argument");

//     eframe::run_native(
//         "Colonizer",
//         NativeOptions {
//             always_on_top: true,
//             initial_window_size: Some(egui::Vec2::new(560.0, 140.0)),
//             ..Default::default()
//         },
//         Box::new(|_cc| Box::new(MyApp::new(username))),
//     );
// }



// // Formats the rob chance as a probability into a percentage
// fn fmt_rob_chance(rob_chance: f64) -> String {
//     let percentage = (rob_chance * 100.0).round() as u8;
//     if percentage == 100 {
//         // pretty easy to see that it's the only card available
//         "   ".to_owned()
//     } else if percentage < 10 {
//         format!(".0{percentage:<1}")
//     } else {
//         format!(
//             ".{:<2}",
//             if percentage % 10 == 0 {
//                 percentage / 10
//             } else {
//                 percentage
//             }
//         )
//     }
// }

