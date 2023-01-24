use serde_json::{json, Value};
use std::net::TcpStream;
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};
use ureq;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GCDResponse {
    pub id: u32,
    pub result: Value,
}

pub type Root1 = Vec<Root2>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root2 {
    pub web_socket_debugger_url: String,
}

pub struct Browser {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    request_id: u32,
}

impl Browser {
    pub fn new(url: &str) -> Result<Browser, Box<dyn std::error::Error>> {
        let urls = ureq::get(url)
            .call()
            .expect("Failed to connect to debugger")
            .into_json::<Root1>()?;
        let url = &urls[0].web_socket_debugger_url;
        let (socket, _response) = connect(url)?;
        Ok(Browser {
            socket,
            request_id: 0,
        })
    }

    /// Sends a request to the server
    pub fn send(
        &mut self,
        method: &str,
        params: Value,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let data = json!({
            "id": self.request_id,
            "method": method,
            "params": params,
        });
        let msg = Message::Text(data.to_string());
        self.socket.write_message(msg)?;

        let msg = self.socket.read_message().expect("Error reading message");
        let json: GCDResponse = serde_json::from_str(msg.to_text()?)?;
        assert!(json.id == self.request_id);

        self.request_id += 1;
        Ok(json.result)
    }

    pub fn evaluate(&mut self, command: &str) -> Result<String, Box<dyn std::error::Error>> {
        let response = self.send("Runtime.evaluate", json!({ "expression": command }))?;

        if let Value::String(html) = &response["result"]["value"] {
            Ok(html.to_owned())
        } else {
            panic!("Failed to find game log.");
        }
    }
}
