use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rand::seq::SliceRandom;
use serde_json::json;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::Duration;

mod battlesnake_api;
mod game_state;
mod heuristic;
mod search;
mod visualizer;

use crate::battlesnake_api::{BattlesnakeRequest, MoveResponse};
use crate::game_state::Direction;
use crate::search::MCTS;
use crate::visualizer::visualize_game_state;

async fn index() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "apiversion": "1",
        "author": "Coolism",
        "color": "#888888",
        "head": "default",
        "tail": "default",
        "version": "0.0.1"
    }))
}

async fn start(info: web::Json<BattlesnakeRequest>) -> impl Responder {
    println!("Game started: {}", info.game.id);
    HttpResponse::Ok()
}

async fn r#move(info: web::Json<BattlesnakeRequest>) -> impl Responder {
    let game_state = info.to_game_state();

    println!("Turn: {}", info.turn);
    println!("Game state:\n{}", visualize_game_state(&game_state));

    let mut mcts = MCTS::new(game_state.clone());

    let duration = Duration::from_millis(400);
    println!("Running MCTS for {} milliseconds", duration.as_millis());

    let root = mcts.run(duration, 12);

    println!(
        "Root node game state:\n{}",
        visualize_game_state(&root.lock().unwrap().game_state)
    );
    println!("Root node visits: {}", root.lock().unwrap().visits);

    if let Some((_, best_child)) = root
        .lock()
        .unwrap()
        .children
        .iter()
        .max_by_key(|(_, child)| child.lock().unwrap().visits)
    {
        let best_child_ref = best_child.lock().unwrap();
        println!(
            "Best child game state:\n{}",
            visualize_game_state(&best_child_ref.game_state)
        );
        println!("Best child visits: {}", best_child_ref.visits);
    }

    let our_snake_id = &info.you.id;
    if let Some(our_move) = mcts.get_best_move_for_snake(our_snake_id) {
        let chosen_move = match our_move {
            Direction::Up => "up",
            Direction::Down => "down",
            Direction::Left => "left",
            Direction::Right => "right",
        };

        HttpResponse::Ok().json(MoveResponse {
            r#move: chosen_move.to_string(),
            shout: Some(format!("Moving {} using MCTS", chosen_move)),
        })
    } else {
        let moves = vec!["up", "down", "left", "right"];
        let chosen_move = moves.choose(&mut rand::thread_rng()).unwrap();

        HttpResponse::Ok().json(MoveResponse {
            r#move: chosen_move.to_string(),
            shout: Some("No valid moves! Moving randomly!".to_string()),
        })
    }
}

async fn end(info: web::Json<BattlesnakeRequest>) -> impl Responder {
    println!("Game ended: {}", info.game.id);
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    println!("Starting server on port: {}", port);

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/start", web::post().to(start))
            .route("/move", web::post().to(r#move))
            .route("/end", web::post().to(end))
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
