use parser::*;
use parser_macros::{command, register, command_n};
use axum::extract::Path;
use axum::{extract::ws::{WebSocket, WebSocketUpgrade, Message}, Router, routing::get, response::{Response, IntoResponse}, http::StatusCode};
use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use jsonwebtoken::{decode, DecodingKey, Algorithm, Validation, decode_header};
use reqwest;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/terminal/:token", get(ws_handler));
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    exp: usize,                 
    iat: usize,     
    aud: String, 
    iss: String,      
    sub: String,
    auth_time: usize,
}

async fn get_decoding_key(kid: &String) -> Option<DecodingKey> {
    let response = reqwest::get("https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com").await.unwrap();
    let keys: HashMap<String, String> = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    let public_key =  keys.get::<String>(kid);
    return match public_key {
        Some(key) => Some(DecodingKey::from_rsa_pem(key.as_bytes()).unwrap()),
        None => None,
    }
}

async fn authorise(token: &str, project_id: &str) -> Result<Claims, Response> {
    fn unauthorised() -> Response {
        StatusCode::UNAUTHORIZED.into_response()
    }
    let algorithm = Algorithm::RS256;
    let validation = Validation::new(algorithm);
    let header = decode_header(token).map_err(|_| unauthorised())?;
    if header.alg != algorithm {
        return Err(unauthorised());
    }
    let decoding_key = get_decoding_key(&header.kid.ok_or(unauthorised())?).await.ok_or(unauthorised())?;
    let decoded_token = decode::<Claims>(token, &decoding_key, &validation).map_err(|_| unauthorised())?;
    let issuer = format!("https://securetoken.google.com/{}", project_id);
    let claims = decoded_token.claims;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let secs = now.as_secs() as usize;
    //expiration date must be in the future, issued-at-time must be in the past,
    //authentication-time must be in the past, aud must be equal to project_id, issuer must be 
    //"https://securetoken.google.com/<projectId>", subject must be a non empty string(user id)
    if claims.exp <= secs || claims.iat >= secs || claims.auth_time >= secs || project_id != claims.aud || claims.iss != issuer || claims.sub.is_empty() {
        return Err(unauthorised());
    }
    Ok(claims)
}

async fn ws_handler(socket_up: WebSocketUpgrade, Path(token): Path<String>) -> Result<Response, Response> {
    let claims = authorise(&token, "personal-743af").await?;
    println!("{:?}", claims);
    Ok(socket_up.on_upgrade(ws))
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
        let outputs = match parse(&message_text) {
            Ok(value) => value,
            Err(err) => vec![Output::Error(err.to_string())],

        };
        for output in outputs {
            let reply = Message::Text(output.to_json());
        if socket.send(reply).await.is_err() {
            // client disconnected
            return;
            }
        }
    }
}

register! {
    "logout": logout
    "lofi": lofi
    "some"
        "another"
            "lol": lol
    "yeah": yeah
    "table": table
    "tree": tree
    "pomodoro": pomodoro
}

#[command]
fn logout() -> Output {
    return Output::Logout;
}

#[command(duration = 1620, pause = 180)]
fn pomodoro(topic_id: i32, duration: i32, pause: i32) -> Output {
    return Output::PomodoroTimer(
        PomodoroTimer { duration: duration, pause: pause, topic_id: topic_id, topic_name: "some name".to_owned() }
    )
}

#[command]
fn tree() -> Output {
    return Output::Tree(
        TreeNode { name: "first".to_owned(), children: Some(vec![
            TreeNode {
                name: "first child".to_owned(), children: Some(vec![
                    TreeNode {
                        name: "another child".to_owned(), children: None,
                    }
                ])
            },
            TreeNode{
                name: "second child".to_owned(), children: None,
            }
        ]) }
    )
}

#[command]
fn table() -> Output {
    return Output::Table(Table{
        title: "Some Title".to_owned(),
        data: vec![vec!["first".to_owned(), "second".to_owned(), "third".to_owned()], vec!["one".to_owned(), "two".to_owned(), "three".to_owned()], vec!["four".to_owned(), "five".to_owned(), "six".to_owned()]],
    })
}


#[command(two = 2, three = "sdfkl")]
fn lofi(one: String, two: i32, three: String) -> Output {
    println!("lofi");
    return Output::Empty;
}

#[command_n]
fn yeah() -> Vec<Output> {
    return vec![Output::Text("one".to_owned()), Output::Text("two".to_owned())];
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
