use parser::{self, Output, TreeNode};
use parser_macros::{command, register, command_n};
use axum::extract::{Path, State};
use axum::{extract::ws::{WebSocket, WebSocketUpgrade, Message}, Router, routing::{get, post}, response::{Response, IntoResponse}, http::StatusCode};
use std::net::SocketAddr;
use sqlx::{PgPool, Pool, Postgres};
use std::future::Future;
use std::pin::Pin;
mod authorise;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let pool = PgPool::connect("postgres://postgres:!566071!Tusik1978@localhost/worktracker").await.expect("creating connection pool failed");
    let app = Router::new().
        route("/terminal/:token", get(ws_handler)).
        route("/worktracker/sessions/add/:topic_id/:start/:end", post(add_session)).
        with_state(pool);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await.unwrap();
    Ok(())
    
}


#[derive(sqlx::FromRow)]
struct Session {
    id: i32,
    start: f64,
    end: f64,
    topic_id: i32,
}

async fn add_session(Path((topic_id, start, end)): Path<(i32, f64, f64)> , State(pool): State<Pool<Postgres>>) {
    println!("received it");
    sqlx::query!("INSERT INTO sessions(topic_id, start, \"end\") VALUES ($1, $2, $3)", topic_id, start, end).execute(&pool).await.unwrap();
}

struct InsertableTopic {
    name: String,
    parent: i32,
}


async fn ws_handler(socket_up: WebSocketUpgrade, Path(token): Path<String>, State(pool): State<Pool<Postgres>>) -> Result<Response, Response> {
    let claims = authorise::authorise(&token, "personal-743af").await?;
    println!("{:?}", claims);
    //ws.on_upgrade(move |socket| handle_socket(socket, addr));
    Ok(socket_up.on_upgrade(|socket| ws(socket, pool)))
}

async fn ws(mut socket: WebSocket, pool: Pool<Postgres>) {
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            // client disconnected
            return;
        };
        let message_text = msg.into_text().unwrap();
        let outputs = match parse(&message_text, parser::State(pool.clone())).await {
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
----"logout": logout
----"lofi": lofi
----"some"
--------"another"
------------"lol": lol
----"yeah": yeah
----"table": table
----"tree": tree
----"pomodoro": pomodoro
}

#[command]
fn logout(_: parser::State<Pool<Postgres>>) -> Output {
    return Output::Logout;
}

#[command(duration = 1620, pause = 180)]
fn pomodoro(topic_id: i32, duration: i32, pause: i32, _: parser::State<Pool<Postgres>>) -> Output {
    return Output::PomodoroTimer(
        parser::PomodoroTimer { duration: duration, pause: pause, topic_id: topic_id, topic_name: "some name".to_owned() }
    )
}

#[command]
fn tree(_: parser::State<Pool<Postgres>>) -> Output {
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
fn table(parser::State(pool): parser::State<Pool<Postgres>>) -> Output {
    return Output::Table(parser::Table{
        title: "Some Title".to_owned(),
        data: vec![vec!["first".to_owned(), "second".to_owned(), "third".to_owned()], vec!["one".to_owned(), "two".to_owned(), "three".to_owned()], vec!["four".to_owned(), "five".to_owned(), "six".to_owned()]],
    })
}


#[command(two = 2, three = "sdfkl")]
fn lofi(one: String, two: i32, three: String, _: parser::State<Pool<Postgres>>) -> Output {
    println!("lofi");
    return Output::Empty;
}

#[command_n]
fn yeah(_: parser::State<Pool<Postgres>>) -> Vec<Output> {
    return vec![Output::Text("one".to_owned()), Output::Text("two".to_owned())];
}

#[command]
fn lol(_: parser::State<Pool<Postgres>>) -> Output {
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
