use parser::*;
use parser_macros::{command, register};
use axum::{extract::ws::{WebSocket, WebSocketUpgrade, Message}, Router, routing::get, response::Response};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/terminal", get(ws_handler));
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn ws_handler(socket_up: WebSocketUpgrade) -> Response {
    socket_up.on_upgrade(ws)
}

async fn ws(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            // client disconnected
            return;
        };
        let message_text = msg.into_text().unwrap();
        let output: Output = match parse(&message_text) {
            Ok(value) => value,
            Err(err) => Output::Error(err.to_string()),

        };
        let reply = Message::Text(output.to_json());
        if socket.send(reply).await.is_err() {
            // client disconnected
            return;
        }
    }
}

register! {
    "lofi": lofi
    "some"
        "another"
            "lol": lol
}

#[command(two = 2, three = "sdfkl")]
fn lofi(one: String, two: i32, three: String) -> Output {
    println!("lofi");
    return Output::Empty;
}

#[command]
fn lol() -> Output {
    println!("lol");
    return Output::Empty;
}

// register! {
//     "lofi": lofi
//     "start": start
//     "popularize": popularize
//     "send": send
//     "sessions"
//         "show"
//             "root": root
//             "today": today
//             "yesterday": yesterday
//         "count"
//             "root": root
//             "today": today
//             "yesterday": yesterday
//     "topics"
//         "add": add
//         "show": show
//     "track": track
// }
