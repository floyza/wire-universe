use std::sync::Arc;
use std::{sync::Mutex, time::Duration};

use actix::prelude::*;
use actix::{Actor, Addr, Context, Handler, Message, StreamHandler};
use actix_files::Files;
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize)]
enum CellState {
    Alive,
    Dead,
    Empty,
    Wire,
}

const WORLD_WIDTH: usize = 500;
const WORLD_HEIGHT: usize = 500;

#[derive(Clone)]
struct World {
    tiles: [CellState; WORLD_WIDTH * WORLD_HEIGHT],
}

impl World {
    fn new() -> World {
        World {
            tiles: [CellState::Empty; WORLD_WIDTH * WORLD_HEIGHT],
        }
    }

    fn idx(x: usize, y: usize) -> usize {
        y * WORLD_WIDTH + x
    }

    fn step(&mut self) {
        let copy = self.clone();
        for i in 0..copy.tiles.len() {
            self.tiles[i] = copy.next_state(i);
        }
    }

    fn next_state(&self, idx: usize) -> CellState {
        match self.tiles[idx] {
            CellState::Alive => CellState::Dead,
            CellState::Dead => CellState::Wire,
            CellState::Empty => CellState::Empty,
            CellState::Wire => {
                let mut living_neighbors = 0;
                for i in [
                    idx - 1,
                    idx + 1,
                    idx - WORLD_WIDTH,
                    idx + WORLD_WIDTH,
                    idx - WORLD_WIDTH - 1,
                    idx - WORLD_WIDTH + 1,
                    idx + WORLD_WIDTH - 1,
                    idx + WORLD_WIDTH + 1,
                ] {
                    if let Some(CellState::Alive) = self.tiles.get(i) {
                        living_neighbors += 1;
                    }
                }
                if living_neighbors == 1 || living_neighbors == 2 {
                    CellState::Alive
                } else {
                    CellState::Wire
                }
            }
        }
    }
}

// #[derive(Deserialize)]
// struct WorldQuery {
//     x: usize,
//     y: usize,
//     w: usize,
//     h: usize,
// }

// #[get("/tiles")]
// async fn get_world(data: web::Data<World>, query: web::Query<WorldQuery>) -> impl Responder {
//     let mut ret = Vec::new();
//     for y in query.y..query.y + query.h {
//         let mut row = Vec::new();
//         for x in query.x..query.x + query.w {
//             let idx = World::idx(x, y);
//             row.push(data.tiles[idx]);
//         }
//         ret.push(row);
//     }
//     HttpResponse::Ok().json(ret)
// }

struct AppState {
    ws_clients: Arc<Mutex<Vec<Addr<MyWs>>>>,
}

#[get("/ws")]
async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let (addr, resp) = ws::WsResponseBuilder::new(MyWs, &req, stream).start_with_addr()?;
    state.ws_clients.lock().unwrap().push(addr);
    println!("{:?}", resp);
    Ok(resp)
}

struct MyWs;

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct ServerEvent {
    event: String,
}

impl Handler<ServerEvent> for MyWs {
    type Result = ();

    fn handle(&mut self, msg: ServerEvent, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.event);
    }
}

async fn world_updator(mut world: Box<World>, clients: Arc<Mutex<Vec<Addr<MyWs>>>>) {
    let mut interval = actix_rt::time::interval(Duration::from_secs(5));
    loop {
        // world.step();
        for client in clients.lock().unwrap().iter() {
            client
                .send(ServerEvent {
                    event: "thing".to_string(),
                })
                .await
                .unwrap();
        }
        interval.tick().await;
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut world = Box::new(World::new());
    world.tiles[0] = CellState::Alive;
    world.tiles[1] = CellState::Wire;
    world.tiles[2] = CellState::Wire;
    let clients = web::Data::new(AppState {
        ws_clients: Arc::new(Mutex::new(Vec::new())),
    });
    actix_rt::spawn(world_updator(world, clients.ws_clients.clone()));
    HttpServer::new(move || {
        App::new()
            .app_data(clients.clone())
            .service(websocket_handler)
            .service(
                Files::new("/", "/home/gavin/src/wire-universe/src/static")
                    .index_file("index.html"),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
