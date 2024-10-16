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

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
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
        let width = self.board.width;
        let height = self.board.height;

        // Helper function to convert (x, y) to index
        fn coord_to_index(x: usize, y: usize, width: usize) -> usize {
            y * width + x
        }

        // Add snakes
        for snake in &self.board.snakes {
            let body: Vec<usize> = snake
                .body
                .iter()
                .map(|coord| coord_to_index(coord.x, coord.y, width))
                .collect();
            game_state.add_snake(snake.id.clone(), body, snake.health);
        }

        // Add food
        for food in &self.board.food {
            let index = coord_to_index(food.x, food.y, width);
            game_state.add_food(index);
        }

        // Add hazards
        for hazard in &self.board.hazards {
            let index = coord_to_index(hazard.x, hazard.y, width);
            game_state.add_hazard(index);
        }

        game_state
    }
}
