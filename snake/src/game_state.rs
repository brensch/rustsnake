// File: src/game_state.rs

use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct Snake {
    pub id: String,
    pub body: VecDeque<Position>,
    pub health: u8,
}

impl Snake {
    pub fn head(&self) -> Position {
        self.body.front().cloned().unwrap_or(Position { index: 0 })
    }

    pub fn length(&self) -> usize {
        self.body.len()
    }
}

#[derive(Debug, Clone)]
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

    pub fn index_to_coord(&self, index: usize) -> (usize, usize) {
        (index % self.width, index / self.width)
    }

    pub fn coord_to_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn is_within_bounds(&self, position: Position) -> bool {
        position.index < self.width * self.height
    }

    pub fn move_snake(&mut self, snake_index: usize, direction: Direction) {
        if snake_index >= self.snakes.len() {
            return;
        }

        let (width, height) = (self.width, self.height);
        let (head_x, head_y) = self.index_to_coord(self.snakes[snake_index].head().index);

        let new_head_index = match direction {
            Direction::Up => self.coord_to_index(head_x, (head_y + 1) % height),
            Direction::Down => self.coord_to_index(head_x, (head_y + height - 1) % height),
            Direction::Left => self.coord_to_index((head_x + width - 1) % width, head_y),
            Direction::Right => self.coord_to_index((head_x + 1) % width, head_y),
        };

        let new_head = Position {
            index: new_head_index,
        };

        // Update snake
        let snake = &mut self.snakes[snake_index];
        snake.body.push_front(new_head);
        snake.body.pop_back();
        snake.health = snake.health.saturating_sub(1);
    }

    pub fn resolve_collisions(&mut self) {
        let mut eaten_food = Vec::new();
        let mut dead_snakes = Vec::new();

        // Check for food consumption and hazard damage
        for (i, snake) in self.snakes.iter_mut().enumerate() {
            let head = snake.head();

            // Food consumption
            if let Some(food_index) = self.food.iter().position(|&f| f == head) {
                eaten_food.push(food_index);
                snake.body.push_back(*snake.body.back().unwrap());
                snake.health = 100; // Reset health when food is eaten
            }

            // Hazard damage
            if self.hazards.contains(&head) {
                snake.health = snake.health.saturating_sub(15);
            }

            // Check for death
            if snake.health == 0 {
                dead_snakes.push(i);
            }
        }

        // Remove eaten food
        for index in eaten_food.into_iter().rev() {
            self.food.swap_remove(index);
        }

        // Check for collisions
        for i in 0..self.snakes.len() {
            let head = self.snakes[i].head();
            for j in 0..self.snakes.len() {
                if i == j {
                    // Self collision (excluding head)
                    if self.snakes[i].body.iter().skip(1).any(|&p| p == head) {
                        dead_snakes.push(i);
                        break;
                    }
                } else {
                    // Collision with other snake
                    if self.snakes[j].body.contains(&head) {
                        if self.snakes[i].length() <= self.snakes[j].length() {
                            dead_snakes.push(i);
                        }
                        break;
                    }
                }
            }
        }

        // Remove dead snakes
        dead_snakes.sort_unstable();
        dead_snakes.dedup();
        for index in dead_snakes.into_iter().rev() {
            self.snakes.swap_remove(index);
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
        let snake = &self.snakes[snake_index];
        let (head_x, head_y) = self.index_to_coord(snake.head().index);
        let mut safe_moves = Vec::new();

        for &direction in &[
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ] {
            let new_position = match direction {
                Direction::Up => self.coord_to_index(head_x, (head_y + 1) % self.height),
                Direction::Down => {
                    self.coord_to_index(head_x, (head_y + self.height - 1) % self.height)
                }
                Direction::Left => {
                    self.coord_to_index((head_x + self.width - 1) % self.width, head_y)
                }
                Direction::Right => self.coord_to_index((head_x + 1) % self.width, head_y),
            };

            if self.is_safe_move(
                Position {
                    index: new_position,
                },
                snake_index,
            ) {
                safe_moves.push(direction);
            }
        }

        safe_moves
    }

    fn is_safe_move(&self, position: Position, snake_index: usize) -> bool {
        // Check if the position is within bounds
        if !self.is_within_bounds(position) {
            return false;
        }

        // Check if the position collides with the snake's own neck
        let snake = &self.snakes[snake_index];
        if snake.body.len() > 1 && position == snake.body[1] {
            return false;
        }

        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
