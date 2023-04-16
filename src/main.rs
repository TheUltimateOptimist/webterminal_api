use parser::{self, Output, TreeNode, ToJson};
use parser_macros::{command, register, command_result};
use axum::extract::{Path, State};
use axum::{extract::ws::{WebSocket, WebSocketUpgrade, Message}, Router, routing::{get, post}, response::{Response, IntoResponse}, http::StatusCode};
use std::net::SocketAddr;
use sqlx::{PgPool, Pool, Postgres, Transaction};
use std::future::Future;
use std::pin::Pin;
mod authorise;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let pool = PgPool::connect("postgres://postgres:!566071!Tusik1978@localhost/worktracker").await.expect("creating connection pool failed");
    let app = Router::new().
        route("/terminal/:token", get(ws_handler)).
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
    duration: i32,
    topic_id: i32,
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
        let outputs = match parse(&message_text, pool.clone()).await {
            Ok(value) => value,
            Err(err) => vec![Output::Error(err.to_string())],

        };
        let reply = Message::Text(outputs.to_json().to_string());
        if socket.send(reply).await.is_err() {
            // client disconnected
            return;
            }
           
    }
}

register! {
----"logout": logout
----"track": track
----"lofi": lofi
----"topics"
--------"add": add_topic
----"sessions"
--------"add": add_session
}


#[derive(Debug)]
struct UnexpectedNone;

impl std::fmt::Display for UnexpectedNone {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", "Internal Error: Got an unexpexted None value.")
    }
}

impl std::error::Error for UnexpectedNone {}

trait OptionIntoResult<T> {
    fn ok(self) -> Result<T, Box<dyn std::error::Error>>;
}

impl<T> OptionIntoResult<T> for Option<T> {
    fn ok(self) -> Result<T, Box<dyn std::error::Error>> {
        match self {
            Some(value) => Ok(value),
            None => Err(Box::new(UnexpectedNone)),
        }
    }
}

trait ResultMessage<T> {
    fn m(self) -> Result<T, String>;
}

impl<T, E: std::error::Error> ResultMessage<T> for Result<T, E> {
    fn m(self) -> Result<T, String> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(err.to_string()),
        }
    }
}

#[command_result]
fn add_session(topic_id: u32, start: f64, duration: u32, pool: Pool<Postgres>) {
    println!("received it");
    sqlx::query!("INSERT INTO sessions(topic_id, start, duration) VALUES ($1, $2, $3)", topic_id as i64, start, duration as i64).execute(&pool).await?;
    Ok(Output::Empty)
}

#[command_result]
fn add_topic(name: String, parent: i32, pool: Pool<Postgres>) {
    let mut transaction = pool.begin().await?;
    let found_parent = sqlx::query_scalar!("SELECT id from topics where id = $1", parent).fetch_one(&mut transaction).await;
    if found_parent.is_err() {
        return Ok(Output::Error("The given parent id does not exist.".to_owned()));
    }
    sqlx::query!("INSERT INTO topics(name) VALUES($1)", name).execute(&mut transaction).await?;
    let id = sqlx::query_scalar!("SELECT MAX(id) from topics").fetch_one(&mut transaction).await?.ok()?;
    sqlx::query!("INSERT INTO hierarchy (parent, child, depth) SELECT parent, $1, depth + 1 from hierarchy where child = $2", id, parent).execute(&mut transaction).await?;
    sqlx::query!("INSERT INTO hierarchy (parent, child, depth) VALUES($1, $2, 1)", parent, id).execute(&mut transaction).await?;
    transaction.commit().await?;
    Ok(Output::Empty)
}

#[command]
fn logout(_: Pool<Postgres>) {
    return Output::Logout;
}

async fn get_past_topic_id<E: for<'a> sqlx::Executor<'a, Database = Postgres>>(steps_back: i64, executor: E) -> Result<i32, Box<dyn std::error::Error>> {
    assert!(steps_back >= 1);
    let ids = sqlx::query!("SELECT topic_id from sessions order by id DESC limit $1", steps_back).fetch_all(executor).await?;
    let id = ids.get((steps_back - 1) as usize).ok()?;
    Ok(id.topic_id.ok()?)
}

#[command_result(id = 0, duration = 1620, pause = 180, last = false, secondlast = false, thirdlast = false)]
fn track(id: u32, duration: u32, pause: u32, last: bool, secondlast: bool, thirdlast: bool, pool: Pool<Postgres>) {
    if (last && secondlast) || (last && thirdlast) || (secondlast && thirdlast) {
        let message = "Only one of the flags --last, --secondlast, --thirdlast can be applied at a time.".to_owned();
        return Ok(Output::Error(message))
    }
    let mut id = id;
    if last || secondlast || thirdlast {
        let mut steps_back = 1;
        if secondlast {
            steps_back = 2;
        }
        if thirdlast {
            steps_back = 3;
        }
        id = get_past_topic_id(steps_back, &pool).await? as u32;
    }
    let topic_name = sqlx::query!("SELECT name from topics where id = $1", id as i64).fetch_one(&pool).await?.name;
    return Ok(Output::PomodoroTimer(
        parser::PomodoroTimer { duration: duration as i32, pause: pause as i32, topic_id: id as i32, topic_name: topic_name }
    ))
}

#[command]
fn lofi(_: Pool<Postgres>) {
    return Output::Url("https://www.youtube.com/watch?v=jfKfPfyJRdk".to_owned());
}

// register! {
//     "lofi": lofi
//     "start": start
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
