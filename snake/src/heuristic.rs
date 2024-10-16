use std::collections::VecDeque;

use crate::game_state::GameState;

pub fn calculate_snake_control(game_state: &GameState) -> Vec<i8> {
    let board_size = game_state.width * game_state.height;
    let mut control = vec![-1; board_size];
    let mut queue = VecDeque::new();
    let mut visited = vec![false; board_size];

    // Initialize the queue with snake heads
    for (i, snake) in game_state.snakes.iter().enumerate() {
        let head = snake.body[0].index;
        queue.push_back((head, i as i8, snake.body.len() as i32));
        visited[head] = true;
        control[head] = i as i8;
    }

    let directions = [1, -1, game_state.width as i32, -(game_state.width as i32)];

    while let Some((pos, snake_id, remaining_length)) = queue.pop_front() {
        if remaining_length == 0 {
            continue;
        }

        for &dir in &directions {
            let new_pos = pos as i32 + dir;
            if new_pos < 0 || new_pos >= board_size as i32 {
                continue;
            }
            if (pos % game_state.width == 0 && dir == -1) || 
               (pos % game_state.width == game_state.width - 1 && dir == 1) {
                continue;
            }

            let new_pos = new_pos as usize;
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

    counts.iter()
        .map(|&count| (count as f32 / board_size as f32) * 100.0)
        .collect()
}