// File: src/main.rs

use battlesnake::game_state::{GameState, Snake, Position, Direction};
use battlesnake::visualizer::visualize_game_state;

fn main() {
    // Create a new game state
    let mut game = GameState::new(11, 11);

    // Add a snake
    let snake = Snake {
        id: "player1".to_string(),
        body: vec![Position { index: 60 }, Position { index: 61 }, Position { index: 62 }].into(),
        health: 100,
    };
    game.snakes.push(snake);

    // Add some food
    game.food.push(Position { index: 22 });
    game.food.push(Position { index: 55 });

    // Add a hazard
    game.hazards.push(Position { index: 33 });

    // Visualize initial state
    println!("Initial state:\n{}", visualize_game_state(&game));

    // Move the snake
    game.move_snake(0, Direction::Up);

    // Visualize state after move
    println!("\nState after move:\n{}", visualize_game_state(&game));
}