use serde::Serialize;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Position {
    pub index: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Snake {
    pub id: String,
    pub body: VecDeque<Position>,
    pub health: u8,
}

impl Snake {
    pub fn head(&self) -> Position {
        self.body
            .front()
            .cloned()
            .unwrap_or(Position { index: usize::MAX })
    }

    pub fn length(&self) -> usize {
        self.body.len()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GameState {
    pub width: usize,
    pub height: usize,
    pub snakes: Vec<Snake>,
    pub food: Vec<Position>,
    pub hazards: Vec<Position>,
}

impl GameState {
    pub fn new(width: usize, height: usize) -> Self {
        GameState {
            width,
            height,
            snakes: Vec::new(),
            food: Vec::new(),
            hazards: Vec::new(),
        }
    }

    pub fn move_snake(&mut self, snake_index: usize, direction: Direction) {
        if snake_index >= self.snakes.len() {
            return;
        }

        let snake = &mut self.snakes[snake_index];
        if snake.health == 0 {
            // Dead snakes do not move
            return;
        }

        let width = self.width;
        let board_size = width * self.height;

        let head_index = snake.head().index;

        // If the snake is already out of bounds, no need to move it
        if head_index == usize::MAX {
            return;
        }

        let new_index = match direction {
            Direction::Up => {
                if head_index >= width {
                    head_index - width
                } else {
                    usize::MAX // Moved out of bounds
                }
            }
            Direction::Down => {
                if head_index + width < board_size {
                    head_index + width
                } else {
                    usize::MAX // Moved out of bounds
                }
            }
            Direction::Left => {
                if head_index % width != 0 {
                    head_index - 1
                } else {
                    usize::MAX // Moved out of bounds
                }
            }
            Direction::Right => {
                if head_index % width != width - 1 {
                    head_index + 1
                } else {
                    usize::MAX // Moved out of bounds
                }
            }
        };

        // Update snake's head position and health
        snake.body.push_front(Position { index: new_index });
        snake.body.pop_back();
        snake.health = snake.health.saturating_sub(1);
    }

    pub fn resolve_collisions(&mut self) {
        let mut eaten_food = HashSet::new();
        let mut snakes_to_kill = HashSet::new();

        // Check for out-of-bounds, dead snakes, and health depletion
        for (i, snake) in self.snakes.iter().enumerate() {
            if snake.health == 0 {
                continue; // Already dead
            }

            let head = snake.head();
            if head.index == usize::MAX {
                // Snake moved out of bounds
                snakes_to_kill.insert(i);
            }
        }

        // Food consumption and hazard damage
        for (i, snake) in self.snakes.iter_mut().enumerate() {
            if snake.health == 0 {
                continue; // Skip dead snakes
            }

            let head = snake.head();

            // Food consumption
            if self.food.contains(&head) {
                eaten_food.insert(head);
                snake.health = 100; // Reset health when food is eaten
                                    // Grow the snake
                if let Some(&tail) = snake.body.back() {
                    snake.body.push_back(tail);
                }
            }

            // Hazard damage
            if self.hazards.contains(&head) {
                snake.health = snake.health.saturating_sub(15);
                if snake.health == 0 {
                    snakes_to_kill.insert(i);
                }
            }
        }

        // Remove eaten food
        self.food.retain(|food_pos| !eaten_food.contains(food_pos));

        // Build a map of head positions to snake indices
        let mut head_positions: HashMap<usize, Vec<usize>> = HashMap::new();
        for (i, snake) in self.snakes.iter().enumerate() {
            if snake.health == 0 {
                continue;
            }
            let head_index = snake.head().index;
            head_positions.entry(head_index).or_default().push(i);
        }

        // Handle head-on collisions
        for snakes_at_position in head_positions.values() {
            if snakes_at_position.len() > 1 {
                self.handle_head_collision(snakes_at_position, &mut snakes_to_kill);
            }
        }

        // Handle collisions with bodies and self-collisions
        let mut body_positions: HashMap<usize, usize> = HashMap::new();
        for (i, snake) in self.snakes.iter().enumerate() {
            if snake.health == 0 {
                continue;
            }
            for pos in snake.body.iter().skip(1) {
                body_positions.insert(pos.index, i);
            }
        }

        for (i, snake) in self.snakes.iter().enumerate() {
            if snake.health == 0 {
                continue; // Skip dead snakes
            }
            let head = snake.head();

            // Self-collision
            if snake.body.iter().skip(1).any(|&p| p == head) {
                // Snake collides with its own body
                snakes_to_kill.insert(i);
                continue;
            }

            // Collision with other snakes' bodies
            if let Some(&other_snake_index) = body_positions.get(&head.index) {
                if other_snake_index != i {
                    // Snake collides with another snake's body
                    snakes_to_kill.insert(i);
                }
            }
        }

        // Update snakes' health after all computations
        for &i in &snakes_to_kill {
            self.snakes[i].health = 0;
        }
    }

    fn handle_head_collision(
        &self,
        snakes_at_position: &[usize],
        snakes_to_kill: &mut HashSet<usize>,
    ) {
        // Determine the maximum length among these snakes
        let lengths: Vec<usize> = snakes_at_position
            .iter()
            .map(|&i| self.snakes[i].length())
            .collect();
        let max_length = *lengths.iter().max().unwrap();
        let all_same_length = lengths.iter().all(|&l| l == max_length);

        for &i in snakes_at_position {
            if all_same_length {
                // All snakes have the same length, all die
                snakes_to_kill.insert(i);
            } else if self.snakes[i].length() < max_length {
                // Snake is shorter than the longest snake, dies
                snakes_to_kill.insert(i);
            }
        }
    }

    pub fn add_snake(&mut self, id: String, body: Vec<usize>, health: u8) {
        let snake_body: VecDeque<Position> =
            body.into_iter().map(|index| Position { index }).collect();
        let snake = Snake {
            id,
            body: snake_body,
            health,
        };
        self.snakes.push(snake);
    }

    pub fn add_food(&mut self, index: usize) {
        self.food.push(Position { index });
    }

    pub fn add_hazard(&mut self, index: usize) {
        self.hazards.push(Position { index });
    }

    pub fn get_safe_moves(&self, snake_index: usize) -> Vec<Direction> {
        if snake_index >= self.snakes.len() {
            return Vec::new();
        }

        let snake = &self.snakes[snake_index];
        if snake.health == 0 {
            return Vec::new(); // Dead snakes have no safe moves
        }

        let head_index = snake.head().index;
        let width = self.width;
        let board_size = width * self.height;
        let mut safe_moves = Vec::new();

        // If the snake is already out of bounds, it has no safe moves
        if head_index == usize::MAX {
            return safe_moves;
        }

        // Get the snake's neck position if it exists (second position in the body)
        let neck_index = snake.body.get(1).map(|pos| pos.index);

        for &direction in &[
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ] {
            let new_index = match direction {
                Direction::Up => {
                    if head_index >= width {
                        head_index - width
                    } else {
                        usize::MAX // Out of bounds
                    }
                }
                Direction::Down => {
                    if head_index + width < board_size {
                        head_index + width
                    } else {
                        usize::MAX // Out of bounds
                    }
                }
                Direction::Left => {
                    if head_index % width != 0 {
                        head_index - 1
                    } else {
                        usize::MAX // Out of bounds
                    }
                }
                Direction::Right => {
                    if head_index % width != width - 1 {
                        head_index + 1
                    } else {
                        usize::MAX // Out of bounds
                    }
                }
            };

            // If the move is out of bounds, skip it
            if new_index == usize::MAX {
                continue;
            }

            // Check if the new position is the snake's own neck
            if neck_index == Some(new_index) {
                continue; // Skip the neck to avoid moving into it
            }

            // No need to check for other snake collisions
            safe_moves.push(direction);
        }

        safe_moves
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
