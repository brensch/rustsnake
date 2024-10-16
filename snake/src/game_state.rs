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
        self.body
            .front()
            .cloned()
            .unwrap_or(Position { index: usize::MAX })
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

    pub fn move_snake(&mut self, snake_index: usize, direction: Direction) {
        if snake_index >= self.snakes.len() {
            return;
        }

        let width = self.width;
        let height = self.height;
        let board_size = width * height;

        let head_index = self.snakes[snake_index].head().index;

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

        // Update snake
        let snake = &mut self.snakes[snake_index];
        snake.body.push_front(Position { index: new_index });
        snake.body.pop_back();
        snake.health = snake.health.saturating_sub(1);
    }

    pub fn resolve_collisions(&mut self) {
        let mut eaten_food = Vec::new();
        let mut dead_snakes_flags = vec![false; self.snakes.len()];

        // First pass: check for out-of-bounds and health depletion
        for (i, snake) in self.snakes.iter().enumerate() {
            let head = snake.head();

            // Out-of-bounds check
            if head.index == usize::MAX {
                dead_snakes_flags[i] = true;
                continue;
            }

            // Health depletion
            if snake.health == 0 {
                dead_snakes_flags[i] = true;
                continue;
            }
        }

        // Food consumption and hazard damage
        for (i, snake) in self.snakes.iter_mut().enumerate() {
            if dead_snakes_flags[i] {
                continue; // Skip dead snakes
            }

            let head = snake.head();

            // Food consumption
            if let Some(food_index) = self.food.iter().position(|&f| f == head) {
                eaten_food.push(food_index);
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
                    dead_snakes_flags[i] = true;
                }
            }
        }

        // Remove eaten food
        for index in eaten_food.into_iter().rev() {
            self.food.swap_remove(index);
        }

        // Check for collisions
        for i in 0..self.snakes.len() {
            if dead_snakes_flags[i] {
                continue; // Skip dead snakes
            }

            let head = self.snakes[i].head();

            // Self-collision
            if self.snakes[i].body.iter().skip(1).any(|&p| p == head) {
                dead_snakes_flags[i] = true;
                continue;
            }

            // Collision with other snakes
            for j in 0..self.snakes.len() {
                if i == j || dead_snakes_flags[j] {
                    continue;
                }
                if self.snakes[j].body.contains(&head) {
                    if self.snakes[i].length() <= self.snakes[j].length() {
                        dead_snakes_flags[i] = true;
                    }
                    break;
                }
            }
        }

        // Remove dead snakes using retain
        let mut index = 0;
        self.snakes.retain(|_| {
            let keep = !dead_snakes_flags[index];
            index += 1;
            keep
        });
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
        let head_index = snake.head().index;
        let width = self.width;
        let height = self.height;
        let board_size = width * height;
        let mut safe_moves = Vec::new();

        // If the snake is already out of bounds, it has no safe moves
        if head_index == usize::MAX {
            return safe_moves;
        }

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
                        usize::MAX
                    }
                }
                Direction::Down => {
                    if head_index + width < board_size {
                        head_index + width
                    } else {
                        usize::MAX
                    }
                }
                Direction::Left => {
                    if head_index % width != 0 {
                        head_index - 1
                    } else {
                        usize::MAX
                    }
                }
                Direction::Right => {
                    if head_index % width != width - 1 {
                        head_index + 1
                    } else {
                        usize::MAX
                    }
                }
            };

            if new_index != usize::MAX {
                let new_position = Position { index: new_index };
                if self.is_safe_move(new_position, snake_index) {
                    safe_moves.push(direction);
                }
            }
        }

        safe_moves
    }

    fn is_safe_move(&self, position: Position, snake_index: usize) -> bool {
        // Position is already guaranteed to be within bounds

        let snake = &self.snakes[snake_index];

        // Check for self-collision with neck
        if snake.body.len() > 1 && position == snake.body[1] {
            return false;
        }

        // Check for collisions with other snakes
        for (i, other_snake) in self.snakes.iter().enumerate() {
            if i == snake_index {
                continue;
            }
            if other_snake.body.contains(&position) {
                return false;
            }
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
