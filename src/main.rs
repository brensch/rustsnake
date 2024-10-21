// File: src/main.rs

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rand::seq::SliceRandom;
use serde_json::json;
use std::env;

mod battlesnake_api;
mod game_state;
mod heuristic;
mod search; // Include the MCTS module
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

    // Create an MCTS instance
    let mut mcts = MCTS::new(game_state.clone());

    // Run the MCTS for a specified duration (e.g., 400 milliseconds)
    let duration = std::time::Duration::from_millis(400);
    let num_threads = num_cpus::get();
    println!("Number of threads: {}", num_threads);

    mcts.run(duration, num_threads);

    // In your move handler after running MCTS
    println!(
        "Root node game state:\n{}",
        visualize_game_state(&mcts.root.lock().unwrap().game_state)
    );
    println!("Root node visits: {}", mcts.root.lock().unwrap().visits);

    if let Some(best_child) = mcts
        .root
        .lock()
        .unwrap()
        .children
        .iter()
        .max_by_key(|child| child.lock().unwrap().visits)
    {
        let best_child_lock = best_child.lock().unwrap();
        println!(
            "Best child game state:\n{}",
            visualize_game_state(&best_child_lock.game_state)
        );
        println!("Best child visits: {}", best_child_lock.visits);
        println!("Best child moves: {:?}", best_child_lock.moves);
    }

    // Get the best move for our snake
    let our_snake_id = &info.you.id;
    if let Some(our_move) = mcts.get_best_move_for_snake(our_snake_id) {
        let chosen_move = match our_move {
            // our board is upside down so flip up and down.
            Direction::Up => "down",
            Direction::Down => "up",
            Direction::Left => "left",
            Direction::Right => "right",
        };

        HttpResponse::Ok().json(MoveResponse {
            r#move: chosen_move.to_string(),
            shout: Some(format!("Moving {} using MCTS", chosen_move)),
        })
    } else {
        // No valid move found; choose a random direction
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
    // Get the port from the environment variable or default to 8080
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    println!("Starting server on port: {}", port);

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/start", web::post().to(start))
            .route("/move", web::post().to(r#move))
            .route("/end", web::post().to(end))
    })
    .bind(format!("0.0.0.0:{}", port))? // Bind to the selected port
    .run()
    .await
}
