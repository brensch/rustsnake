// File: src/battlesnake_api.rs

use crate::game_state::{GameState, Position, Snake};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct BattlesnakeRequest {
    pub game: Game,
    pub turn: u32,
    pub board: Board,
    pub you: Battlesnake,
}

#[derive(Deserialize, Debug)]
pub struct Game {
    pub id: String,
    pub ruleset: Ruleset,
    pub timeout: u32,
}

#[derive(Deserialize, Debug)]
pub struct Ruleset {
    pub name: String,
    pub version: String,
}

#[derive(Deserialize, Debug)]
pub struct Board {
    pub height: usize,
    pub width: usize,
    pub food: Vec<Coord>,
    pub hazards: Vec<Coord>,
    pub snakes: Vec<Battlesnake>,
}

#[derive(Deserialize, Debug)]
pub struct Battlesnake {
    pub id: String,
    pub name: String,
    pub health: u8,
    pub body: Vec<Coord>,
    pub head: Coord,
    pub length: usize,
}

#[derive(Deserialize, Debug)]
pub struct Coord {
    pub x: usize,
    pub y: usize,
}

#[derive(Serialize)]
pub struct MoveResponse {
    pub r#move: String,
    pub shout: Option<String>,
}

impl BattlesnakeRequest {
    pub fn to_game_state(&self) -> GameState {
        let mut game_state = GameState::new(self.board.width, self.board.height);

        // Add snakes
        for snake in &self.board.snakes {
            let body: Vec<usize> = snake
                .body
                .iter()
                .map(|coord| game_state.coord_to_index(coord.x, coord.y))
                .collect();
            game_state.add_snake(snake.id.clone(), body, snake.health);
        }

        // Add food
        for food in &self.board.food {
            game_state.add_food(game_state.coord_to_index(food.x, food.y));
        }

        // Add hazards
        for hazard in &self.board.hazards {
            game_state.add_hazard(game_state.coord_to_index(hazard.x, hazard.y));
        }

        game_state
    }
}
