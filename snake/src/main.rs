// File: src/main.rs

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rand::seq::SliceRandom;
use serde_json::json;

mod battlesnake_api;
mod game_state;
mod heuristic;
mod visualizer;

use crate::battlesnake_api::{BattlesnakeRequest, MoveResponse};
use crate::game_state::Direction;
use crate::heuristic::{calculate_control_percentages, calculate_move_control};
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
    let mut game_state = info.to_game_state();
    let control_percentages = calculate_control_percentages(&game_state);

    println!("Turn: {}", info.turn);
    println!("Game state:\n{}", visualize_game_state(&game_state));
    println!("Control percentages: {:?}", control_percentages);

    // Find the index of our snake
    let our_snake_index = game_state
        .snakes
        .iter()
        .position(|s| s.id == info.you.id)
        .unwrap();

    // Get safe moves
    let safe_moves = game_state.get_safe_moves(our_snake_index);

    if safe_moves.is_empty() {
        // If there are no safe moves, choose a random direction
        let moves = vec!["up", "down", "left", "right"];
        let chosen_move = moves.choose(&mut rand::thread_rng()).unwrap();

        HttpResponse::Ok().json(MoveResponse {
            r#move: chosen_move.to_string(),
            shout: Some("No safe moves! Moving randomly!".to_string()),
        })
    } else {
        // Calculate control percentages for each safe move
        let move_controls: Vec<(Direction, f32)> = safe_moves
            .iter()
            .map(|&direction| {
                (
                    direction,
                    calculate_move_control(&game_state, our_snake_index, direction),
                )
            })
            .collect();

        // Find the move(s) with the highest control percentage
        let max_control = move_controls
            .iter()
            .map(|&(_, control)| control)
            .fold(f32::NEG_INFINITY, f32::max);
        let best_moves: Vec<&Direction> = move_controls
            .iter()
            .filter(|&(_, control)| *control == max_control)
            .map(|(direction, _)| direction)
            .collect();

        // Choose a random move from the best moves
        let chosen_direction = best_moves.choose(&mut rand::thread_rng()).unwrap();
        let chosen_move = match chosen_direction {
            Direction::Up => "up",
            Direction::Down => "down",
            Direction::Left => "left",
            Direction::Right => "right",
        };

        HttpResponse::Ok().json(MoveResponse {
            r#move: chosen_move.to_string(),
            shout: Some(format!(
                "Moving {} for max control: {:.2}%",
                chosen_move, max_control
            )),
        })
    }
}

async fn end(info: web::Json<BattlesnakeRequest>) -> impl Responder {
    println!("Game ended: {}", info.game.id);
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/start", web::post().to(start))
            .route("/move", web::post().to(r#move))
            .route("/end", web::post().to(end))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
