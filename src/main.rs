use actix_files::Files;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
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

#[derive(Deserialize)]
struct WorldQuery {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

#[get("/tiles")]
async fn get_world(data: web::Data<World>, query: web::Query<WorldQuery>) -> impl Responder {
    let mut ret = Vec::new();
    for y in query.y..query.y + query.h {
        let mut row = Vec::new();
        for x in query.x..query.x + query.w {
            let idx = World::idx(x, y);
            row.push(data.tiles[idx]);
        }
        ret.push(row);
    }
    HttpResponse::Ok().json(ret)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut world = World::new();
    world.tiles[0] = CellState::Alive;
    world.tiles[1] = CellState::Wire;
    world.tiles[2] = CellState::Wire;
    let world = web::Data::new(world);
    HttpServer::new(move || {
        App::new()
            .app_data(world.clone())
            .service(get_world)
            .service(
                Files::new("/", "/home/gavin/src/wire-universe/src/static")
                    .index_file("index.html"),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
