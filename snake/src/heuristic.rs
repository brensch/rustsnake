use std::collections::VecDeque;

use crate::game_state::{Direction, GameState};

pub fn calculate_snake_control(game_state: &GameState) -> Vec<i8> {
    let width = game_state.width;
    let height = game_state.height;
    let board_size = width * height;
    let mut control = vec![-1; board_size];
    let mut queue = VecDeque::new();
    let mut visited = vec![false; board_size];

    // Initialize the queue with snake heads
    for (i, snake) in game_state.snakes.iter().enumerate() {
        let head = snake.body[0].index;

        // Skip if the snake is out of bounds
        if head == usize::MAX {
            continue;
        }

        queue.push_back((head, i as i8, snake.body.len() as i32));
        visited[head] = true;
        control[head] = i as i8;
    }

    while let Some((pos, snake_id, remaining_length)) = queue.pop_front() {
        if remaining_length == 0 {
            continue;
        }

        let mut neighbors = Vec::new();

        // Calculate neighboring positions without wrapping
        // Up
        if pos >= width {
            neighbors.push(pos - width);
        }
        // Down
        if pos + width < board_size {
            neighbors.push(pos + width);
        }
        // Left
        if pos % width != 0 {
            neighbors.push(pos - 1);
        }
        // Right
        if pos % width != width - 1 {
            neighbors.push(pos + 1);
        }

        for new_pos in neighbors {
            if !visited[new_pos] {
                visited[new_pos] = true;
                control[new_pos] = snake_id;
                queue.push_back((new_pos, snake_id, remaining_length - 1));
            }
        }
    }

    control
}

pub fn calculate_control_percentages(game_state: &GameState) -> Vec<f32> {
    let control = calculate_snake_control(game_state);
    let board_size = game_state.width * game_state.height;
    let mut counts = vec![0; game_state.snakes.len()];

    for &c in &control {
        if c >= 0 {
            counts[c as usize] += 1;
        }
    }

    counts
        .iter()
        .map(|&count| (count as f32 / board_size as f32) * 100.0)
        .collect()
}

pub fn calculate_move_control(
    game_state: &GameState,
    snake_index: usize,
    direction: Direction,
) -> f32 {
    if snake_index >= game_state.snakes.len() {
        return 0.0;
    }

    // Clone the game state and move the snake
    let mut new_game_state = game_state.clone();
    new_game_state.move_snake(snake_index, direction);
    new_game_state.resolve_collisions();

    // Check if the snake is still alive
    let snake_id = &game_state.snakes[snake_index].id;
    let new_snake_index = new_game_state.snakes.iter().position(|s| &s.id == snake_id);

    if let Some(new_snake_index) = new_snake_index {
        // Calculate the control after the move
        let control = calculate_snake_control(&new_game_state);
        let snake_control = control
            .iter()
            .filter(|&&c| c == new_snake_index as i8)
            .count();

        (snake_control as f32 / (game_state.width * game_state.height) as f32) * 100.0
    } else {
        // Snake died after the move
        0.0
    }
}
