use std::collections::VecDeque;

use crate::game_state::{Direction, GameState};

/// Calculates which snake controls each position on the board.
///
/// The control is determined by simulating how each snake can expand its territory.
/// Each snake tries to reach empty positions as quickly as possible, considering when
/// positions become unoccupied. The snake that can reach a position first controls it.
/// If two snakes can reach a position at the same time, the snake with the lower index
/// controls it.
///
/// # Parameters
/// - `game_state`: The current state of the game.
///
/// # Returns
/// A vector of length `board_size`, where each element is the index of the snake that
/// controls that position. If a position is unclaimed, the value is -1.
pub fn calculate_snake_control(game_state: &GameState) -> Vec<i8> {
    let width = game_state.width;
    let height = game_state.height;
    let board_size = width * height;

    // Initialize the control vector with -1 (no snake controls the position yet)
    let mut control = vec![-1; board_size];

    // Initialize the minimum depth (time) at which each position is reached
    let mut min_depth = vec![u32::MAX; board_size];

    // Queue for BFS (Breadth-First Search)
    let mut queue = VecDeque::new();

    // This vector will store the earliest time when each position becomes unoccupied
    let mut position_unoccupied_at = vec![0u32; board_size];

    // Precompute when each position becomes unoccupied for all snakes
    for snake in &game_state.snakes {
        // Length of the snake's body
        let body_len = snake.body.len() as u32;

        // Skip the snake if it's dead or has an invalid position
        if snake.health == 0 || snake.body.is_empty() || snake.body[0].index == usize::MAX {
            continue;
        }

        // Iterate over each segment of the snake's body
        for (i, body_part) in snake.body.iter().enumerate() {
            let pos = body_part.index;

            // Determine the time when this position becomes unoccupied
            let t = if i == snake.body.len() - 1 {
                // For the tail segment, it becomes unoccupied at time 0 (immediately)
                0
            } else {
                // For other segments, they become unoccupied after the snake moves
                i as u32 + 1 // Time steps start from 1
            };

            // Update the earliest time when this position becomes unoccupied
            if t > position_unoccupied_at[pos] {
                position_unoccupied_at[pos] = t;
            }
        }
    }

    // Initialize the BFS queue with the heads of all snakes
    for (i, snake) in game_state.snakes.iter().enumerate() {
        let head = snake.body[0].index;

        // Skip the snake if it's dead or has an invalid position
        if head == usize::MAX || snake.health == 0 {
            continue;
        }

        // Add the head position to the queue with depth 0
        queue.push_back((head, i as i8, 0u32)); // (position, snake_id, depth)
        min_depth[head] = 0; // The head position is reached at time 0
        control[head] = i as i8; // The snake controls its head position
    }

    // Directions to move on the board: Up, Down, Left, Right
    let directions = [-(width as isize), width as isize, -1, 1];

    // Perform BFS to expand each snake's control territory
    while let Some((pos, snake_id, depth)) = queue.pop_front() {
        let next_depth = depth + 1; // Time increases by 1 with each move

        // Try moving in all four directions
        for &offset in &directions {
            let new_pos = pos as isize + offset;

            // Check if the new position is within the board boundaries
            if new_pos < 0 || new_pos >= board_size as isize {
                continue; // Skip positions outside the board
            }

            // Check for horizontal wrapping (moving left or right across edges)
            if (offset == -1 && pos % width == 0) || (offset == 1 && (pos + 1) % width == 0) {
                continue; // Skip wrapping around the edges
            }

            let new_pos = new_pos as usize;

            // Check if the position is occupied at the time we reach it
            if position_unoccupied_at[new_pos] > next_depth {
                continue; // Position is still occupied by a snake segment
            }

            // Update control if we found a shorter path to this position
            if next_depth < min_depth[new_pos] {
                min_depth[new_pos] = next_depth;
                control[new_pos] = snake_id;
                // Add the new position to the queue to continue expanding
                queue.push_back((new_pos, snake_id, next_depth));
            }
            // If two snakes reach the position at the same time
            else if next_depth == min_depth[new_pos] && snake_id < control[new_pos] {
                control[new_pos] = snake_id;
                // No need to enqueue again since depth is the same
            }
        }
    }

    // Return the control vector indicating which snake controls each position
    control
}

/// Calculates the percentage of the board controlled by each snake.
///
/// # Parameters
/// - `game_state`: The current state of the game.
///
/// # Returns
/// A vector where each element is the percentage of the board controlled by the corresponding snake.
pub fn calculate_control_percentages(game_state: &GameState) -> Vec<f32> {
    let control = calculate_snake_control(game_state);
    let board_size = game_state.width * game_state.height;

    // Initialize counts for each snake
    let mut counts = vec![0; game_state.snakes.len()];

    // Count how many positions each snake controls
    for &c in &control {
        if c >= 0 {
            counts[c as usize] += 1;
        }
    }

    // Calculate the percentage for each snake
    counts
        .iter()
        .map(|&count| (count as f32 / board_size as f32))
        .collect()
}

/// Calculates the control percentage for a specific snake after making a move.
///
/// # Parameters
/// - `game_state`: The current state of the game.
/// - `snake_index`: The index of the snake making the move.
/// - `direction`: The direction in which the snake moves.
///
/// # Returns
/// The percentage of the board controlled by the snake after the move.
pub fn calculate_move_control(
    game_state: &GameState,
    snake_index: usize,
    direction: Direction,
) -> f32 {
    if snake_index >= game_state.snakes.len() {
        return 0.0; // Invalid snake index
    }

    // Clone the game state to simulate the move
    let mut new_game_state = game_state.clone();

    // Move the snake in the specified direction
    new_game_state.move_snake(snake_index, direction);

    // Resolve any collisions that occur after the move
    new_game_state.resolve_collisions();

    // Get the ID of the snake
    let snake_id = &game_state.snakes[snake_index].id;

    // Find the index of the snake in the new game state
    let new_snake_index = new_game_state.snakes.iter().position(|s| &s.id == snake_id);

    if let Some(new_snake_index) = new_snake_index {
        // Calculate the control after the move
        let control = calculate_snake_control(&new_game_state);

        // Count the number of positions controlled by the snake
        let snake_control = control
            .iter()
            .filter(|&&c| c == new_snake_index as i8)
            .count();

        // Calculate the percentage of the board controlled by the snake
        (snake_control as f32 / (game_state.width * game_state.height) as f32) * 100.0
    } else {
        // The snake died after the move
        0.0
    }
}
