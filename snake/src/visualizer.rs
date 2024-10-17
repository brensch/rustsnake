// File: src/visualizer.rs

use crate::game_state::{GameState, Position, Snake};

pub fn visualize_game_state(game_state: &GameState) -> String {
    let mut grid = vec!['.'; game_state.width * game_state.height];

    // Place food
    for food in &game_state.food {
        if food.index < grid.len() {
            grid[food.index] = '*';
        }
    }

    // Place hazards
    for hazard in &game_state.hazards {
        if hazard.index < grid.len() {
            grid[hazard.index] = '!';
        }
    }

    // Place snakes
    for (i, snake) in game_state.snakes.iter().enumerate() {
        if snake.health == 0 {
            continue;
        }
        let snake_char = (b'a' + i as u8) as char;
        let head_char = snake_char.to_ascii_uppercase();

        for (j, &Position { index }) in snake.body.iter().enumerate() {
            // Skip if the snake's position is out of bounds (i.e., usize::MAX)
            if index != usize::MAX && index < grid.len() {
                grid[index] = if j == 0 { head_char } else { snake_char };
            }
        }
    }

    // Convert grid to string with newlines
    grid.chunks(game_state.width)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn json_to_game_state(json: &serde_json::Value) -> GameState {
    let width = json["width"].as_u64().unwrap() as usize;
    let height = json["height"].as_u64().unwrap() as usize;
    let mut game = GameState::new(width, height);

    for snake_json in json["snakes"].as_array().unwrap() {
        let body: Vec<Position> = snake_json["body"]
            .as_array()
            .unwrap()
            .iter()
            .map(|index| Position {
                index: index.as_u64().unwrap() as usize,
            })
            .collect();

        game.snakes.push(Snake {
            id: snake_json["id"].as_str().unwrap().to_string(),
            body: body.into(),
            health: snake_json["health"].as_u64().unwrap() as u8,
        });
    }

    game.food = json["food"]
        .as_array()
        .unwrap()
        .iter()
        .map(|index| Position {
            index: index.as_u64().unwrap() as usize,
        })
        .collect();

    game.hazards = json["hazards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|index| Position {
            index: index.as_u64().unwrap() as usize,
        })
        .collect();

    game
}
