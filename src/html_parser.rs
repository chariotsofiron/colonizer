use regex::Regex;
use scraper::{Html, Selector};

fn parse_color(text: &str) -> (u8, u8, u8) {
    let pattern = Regex::new(r"(\d+), (\d+), (\d+)").unwrap();
    let captures = pattern.captures(text).unwrap();
    let r = captures.get(1).unwrap().as_str().parse::<u8>().unwrap();
    let g = captures.get(2).unwrap().as_str().parse::<u8>().unwrap();
    let b = captures.get(3).unwrap().as_str().parse::<u8>().unwrap();
    (r, g, b)
}

/// Parses HTML into text messages
pub fn parse(html: &str) -> Vec<((u8, u8, u8), String)> {
    let document = Html::parse_document(html);
    let msg_selector = Selector::parse(".message_post").unwrap();
    let img_selector = Selector::parse("img").unwrap();

    let mut lines = Vec::new();
    for message in document.select(&msg_selector) {
        let color = parse_color(message.value().attr("style").unwrap());
        let mut text = message.inner_html();
        // replace images with their alt-text
        for img in message.select(&img_selector) {
            let alt_text = format!("{} ", img.value().attr("alt").unwrap());
            text = text.replace(&img.html(), &alt_text);
        }

        if text.contains("<hr>") {
            continue;
        }
        lines.push((color, text));
    }
    lines
}
