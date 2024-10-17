use std::collections::VecDeque;

use crate::game_state::{Direction, GameState};

pub fn calculate_snake_control(game_state: &GameState) -> Vec<i8> {
    let width = game_state.width;
    let height = game_state.height;
    let board_size = width * height;
    let mut control = vec![-1; board_size];
    let mut min_depth = vec![u32::MAX; board_size];
    let mut queue = VecDeque::new();

    // Precompute when each position becomes unoccupied
    let mut position_unoccupied_at = vec![0u32; board_size];

    for snake in &game_state.snakes {
        let body_len = snake.body.len() as u32;

        // Skip if the snake is out of bounds or dead
        if snake.health == 0 || snake.body.is_empty() || snake.body[0].index == usize::MAX {
            continue;
        }

        for (i, body_part) in snake.body.iter().enumerate() {
            let pos = body_part.index;
            let t = i as u32 + 1; // Time when the position becomes unoccupied
            if t > position_unoccupied_at[pos] {
                position_unoccupied_at[pos] = t;
            }
        }
    }

    // Initialize the queue with snake heads
    for (i, snake) in game_state.snakes.iter().enumerate() {
        let head = snake.body[0].index;

        // Skip if the snake is out of bounds or dead
        if head == usize::MAX || snake.health == 0 {
            continue;
        }

        queue.push_back((head, i as i8, 0u32)); // position, snake_id, depth
        min_depth[head] = 0;
        control[head] = i as i8;
    }

    while let Some((pos, snake_id, depth)) = queue.pop_front() {
        let next_depth = depth + 1;

        // Directions: Up (-width), Down (+width), Left (-1), Right (+1)
        let directions = [-(width as isize), width as isize, -1, 1];

        for &offset in &directions {
            let new_pos = pos as isize + offset;

            // Check if new position is within bounds
            if new_pos < 0 || new_pos >= board_size as isize {
                continue;
            }

            // Check for horizontal wrapping
            if (offset == -1 && pos % width == 0) || (offset == 1 && (pos + 1) % width == 0) {
                continue;
            }

            let new_pos = new_pos as usize;

            // Check if the position is unoccupied at the time we reach it
            if position_unoccupied_at[new_pos] > next_depth {
                continue; // Position is occupied
            }

            // Update control if we found a shorter path or a snake with a lower index
            if next_depth < min_depth[new_pos] {
                min_depth[new_pos] = next_depth;
                control[new_pos] = snake_id;
                queue.push_back((new_pos, snake_id, next_depth));
            } else if next_depth == min_depth[new_pos] && snake_id < control[new_pos] {
                control[new_pos] = snake_id;
                // No need to enqueue again since depth is the same
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
