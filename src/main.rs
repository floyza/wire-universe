use std::time::Duration;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, get_service},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{
    select,
    sync::mpsc,
    time::{interval, Interval},
};
use tokio::{sync::broadcast, task};
use tower_http::services::ServeDir;
use wireworld::{CellState, World};

mod wireworld;

#[derive(Clone)]
struct AppState {
    world_sender: broadcast::Sender<World>,
    update_sender: mpsc::UnboundedSender<CellModification>,
}

async fn handler(ws: WebSocketUpgrade, state: State<AppState>) -> Response {
    ws.on_upgrade(move |socket| {
        handle_socket(
            socket,
            state.world_sender.subscribe(),
            state.update_sender.clone(),
        )
    })
}

#[derive(Serialize, PartialEq, Debug)]
enum FromServer {
    Refresh {
        x: usize,
        y: usize,
        tiles: Vec<Vec<CellState>>,
    },
}

#[derive(Deserialize, PartialEq, Debug)]
enum FromClient {
    ModifyCell {
        x: usize,
        y: usize,
        cell: CellState,
    },
    SetView {
        x: usize,
        y: usize,
        w: usize,
        h: usize,
    },
}

#[cfg(test)]
mod tests {
    use crate::wireworld::CellState;

    #[test]
    fn from_client_deserialization() {
        use super::FromClient;
        let js = r#"{"ModifyCell": { "x": 1, "y": 5, "cell": "Alive" }}"#;
        let msg = FromClient::ModifyCell {
            x: 1,
            y: 5,
            cell: CellState::Alive,
        };
        assert_eq!(serde_json::from_str::<FromClient>(js).unwrap(), msg);
    }
}

struct CellModification {
    x: usize,
    y: usize,
    cell: wireworld::CellState,
}

async fn handle_socket(
    mut socket: WebSocket,
    mut world_receiver: broadcast::Receiver<World>,
    update_sender: mpsc::UnboundedSender<CellModification>,
) {
    let mut view_x = 0;
    let mut view_y = 0;
    let mut view_w = 30;
    let mut view_h = 30;
    loop {
        select! {
            Ok(world) = world_receiver.recv() => {
                let tiles = world.copy_slice(view_x, view_y, view_w, view_h);
                let msg = FromServer::Refresh { x: view_x, y: view_y, tiles };
                if socket.send(Message::Text(serde_json::to_string(&msg).unwrap())).await.is_err() {
                    return;
                }
            }
            Some(msg) = socket.recv() => {
                match msg {
                    Ok(Message::Text(data)) => {
                        if let Ok(val) = serde_json::from_str::<FromClient>(&data) {
                            match val {
                                FromClient::ModifyCell { x, y, cell } => {
                                    _ = update_sender.send(CellModification { x, y, cell });
                                }
                                FromClient::SetView { x, y, w, h } => {
                                    view_x = x;
                                    view_y = y;
                                    view_w = w;
                                    view_h = h;
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
            else => {
                return;
            }
        }
    }
}

async fn error_404(uri: Uri) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("Not found: {}", uri.path()))
}

async fn world_updator(
    mut world: World,
    world_sender: broadcast::Sender<World>,
    mut update_receiver: mpsc::UnboundedReceiver<CellModification>,
) {
    let mut interval = interval(Duration::from_millis(500));
    loop {
        _ = world_sender.send(world.clone()); // it's fine if there isn't anyone listening
        loop {
            select! {
                Some(CellModification {x, y, cell}) = update_receiver.recv() => {
                    world.tiles[World::idx(x, y)] = cell;
                }
                _ = interval.tick() => {
                    break;
                }
            }
        }
        world.step();
    }
}

#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel::<World>(16);
    let (tx2, rx) = mpsc::unbounded_channel::<CellModification>();
    let world_task = task::spawn(world_updator(wireworld::sample_world(), tx.clone(), rx));
    let serve_dir = get_service(ServeDir::new("assets")).handle_error(handle_error);
    let state = AppState {
        world_sender: tx,
        update_sender: tx2,
    };
    let app = Router::new()
        .route("/ws", get(handler))
        .nest_service("/", serve_dir)
        .fallback(error_404)
        .with_state(state);

    let server =
        axum::Server::bind(&"0.0.0.0:3000".parse().unwrap()).serve(app.into_make_service());

    select! {_ = world_task => {} _ = server => {}}
}

async fn handle_error(err: std::io::Error) -> impl IntoResponse {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Internal server error: {}", err),
    )
}
