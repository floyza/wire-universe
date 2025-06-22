use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

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
use tokio::{select, sync::mpsc, time::interval};
use tokio::{sync::broadcast, task};
use tower_http::services::ServeDir;

use wire_universe::{
    proto::{FromClient, FromServer},
    CellState, Point,
};
use world::World;

pub mod world;

#[derive(Clone)]
struct AppState {
    world_sender: broadcast::Sender<World>,
    update_sender: mpsc::UnboundedSender<CellModification>,
    last_world: Arc<Mutex<Arc<World>>>,
}

async fn handler(ws: WebSocketUpgrade, state: State<AppState>) -> Response {
    ws.on_upgrade(move |socket| {
        handle_socket(
            socket,
            state.world_sender.subscribe(),
            state.update_sender.clone(),
            state.last_world.clone(),
        )
    })
}

struct CellModification {
    x: i32,
    y: i32,
    cell: CellState,
}

async fn handle_socket(
    mut socket: WebSocket,
    mut world_receiver: broadcast::Receiver<World>,
    update_sender: mpsc::UnboundedSender<CellModification>,
    last_world: Arc<Mutex<Arc<World>>>,
) {
    let mut view_x = 0;
    let mut view_y = 0;
    let mut view_w = 30;
    let mut view_h = 30;
    let mut sending = false;
    let mut synced = false;
    loop {
        select! {
            world = world_receiver.recv() => {
                match world {
                    Ok(world) => {
                        if sending {
                            let msg;
                            if synced {
                                let tiles = world.copy_perimeter(view_x, view_y, view_w, view_h);
                                msg = FromServer::PartialRefresh { tiles };
                            } else {
                                let tiles = world.copy_slice(view_x, view_y, view_w, view_h);
                                msg = FromServer::FullRefresh { x: view_x, y: view_y, tiles };
                                synced = true;
                            }
                            if socket.send(Message::Binary(rmp_serde::to_vec(&msg).unwrap())).await.is_err() {
                                return;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {}
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        synced = false;
                    }
                }
            }
            Some(msg) = socket.recv() => {
                match msg {
                    Ok(Message::Binary(data)) => {
                        if let Ok(val) = rmp_serde::from_slice::<FromClient>(&data) {
                            match val {
                                FromClient::ModifyCell { x, y, cell } => {
                                    _ = update_sender.send(CellModification { x, y, cell });
                                }
                                FromClient::SetView { x, y, w, h } => {
                                    view_x = x;
                                    view_y = y;
                                    view_w = w;
                                    view_h = h;
                                    synced = false;
                                }
                                FromClient::StartStream => {
                                    let world = last_world.lock().unwrap().clone();
                                    let tiles = world.copy_slice(view_x, view_y, view_w, view_h);
                                    let msg = FromServer::FullRefresh { x: view_x, y: view_y, tiles };
                                    if socket.send(Message::Binary(rmp_serde::to_vec(&msg).unwrap())).await.is_err() {
                                        return;
                                    }
                                    sending = true;
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
    last_world: Arc<Mutex<Arc<World>>>,
) {
    let mut interval = interval(Duration::from_millis(100));
    loop {
        // TODO share both of these clones so we have to do half as much cloning
        _ = world_sender.send(world.clone()); // it's fine if there isn't anyone listening
        *last_world.lock().unwrap() = Arc::new(world.clone());
        loop {
            select! {
                Some(CellModification {x, y, cell}) = update_receiver.recv() => {
                    world.set_tile(Point {x, y}, cell);
                }
                _ = interval.tick() => {
                    break;
                }
            }
        }
        world.step();
    }
}

pub async fn serve() {
    let (tx, _) = broadcast::channel::<World>(16);
    let (tx2, rx) = mpsc::unbounded_channel::<CellModification>();
    let starting_world: World = World::from_wi(Path::new("./primes.wi")).unwrap();
    let last_world = Arc::new(Mutex::new(Arc::new(starting_world.clone())));
    let world_task = task::spawn(world_updator(
        starting_world,
        tx.clone(),
        rx,
        last_world.clone(),
    ));
    let serve_dir = get_service(ServeDir::new("assets")).handle_error(handle_error);
    let state = AppState {
        world_sender: tx,
        update_sender: tx2,
        last_world,
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
