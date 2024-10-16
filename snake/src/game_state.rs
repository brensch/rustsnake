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

    pub fn index_to_coord(&self, index: usize) -> (isize, isize) {
        let x = (index % self.width) as isize;
        let y = (index / self.width) as isize;
        (x, y)
    }

    pub fn coord_to_index(&self, x: isize, y: isize) -> usize {
        (y as usize) * self.width + (x as usize)
    }

    pub fn is_within_bounds(&self, position: Position) -> bool {
        let (x, y) = self.index_to_coord(position.index);
        x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize
    }

    pub fn move_snake(&mut self, snake_index: usize, direction: Direction) {
        if snake_index >= self.snakes.len() {
            return;
        }

        let (head_x, head_y) = self.index_to_coord(self.snakes[snake_index].head().index);

        let (new_x, new_y) = match direction {
            Direction::Up => (head_x, head_y + 1),
            Direction::Down => (head_x, head_y.wrapping_sub(1)),
            Direction::Left => (head_x.wrapping_sub(1), head_y),
            Direction::Right => (head_x + 1, head_y),
        };

        // Create a new head position without wrapping
        let new_position = Position {
            index: self.coord_to_index(new_x, new_y),
        };

        // Update snake
        let snake = &mut self.snakes[snake_index];
        snake.body.push_front(new_position);
        snake.body.pop_back();
        snake.health = snake.health.saturating_sub(1);
    }

    pub fn resolve_collisions(&mut self) {
        let mut eaten_food = Vec::new();
        let mut dead_snakes = Vec::new();

        // First pass: check for food consumption, hazard damage, and out-of-bounds
        for (i, snake) in self.snakes.iter().enumerate() {
            let head = snake.head();

            // Check if the snake is out of bounds
            if !self.is_within_bounds(head) {
                dead_snakes.push(i);
                continue; // Skip the rest of the checks for this snake
            }

            // Food consumption
            if let Some(food_index) = self.food.iter().position(|&f| f == head) {
                eaten_food.push(food_index);
            }

            // Hazard damage
            if self.hazards.contains(&head) {
                dead_snakes.push(i); // If snake head is in hazard, mark snake as dead
            }

            // Check for health depletion
            if snake.health == 0 {
                dead_snakes.push(i);
            }
        }

        // Remove eaten food
        for index in eaten_food.into_iter().rev() {
            self.food.swap_remove(index);
        }

        // Second pass: check for collisions between snakes
        for i in 0..self.snakes.len() {
            let head = self.snakes[i].head();
            for j in 0..self.snakes.len() {
                if i != j && self.snakes[j].body.contains(&head) {
                    if self.snakes[i].length() <= self.snakes[j].length() {
                        dead_snakes.push(i);
                    }
                    break;
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
            let (new_x, new_y) = match direction {
                Direction::Up => (head_x, head_y + 1),
                Direction::Down => (head_x, head_y.wrapping_sub(1)),
                Direction::Left => (head_x.wrapping_sub(1), head_y),
                Direction::Right => (head_x + 1, head_y),
            };

            let new_position = Position {
                index: self.coord_to_index(new_x, new_y),
            };

            if self.is_safe_move(new_position, snake_index) {
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

        let snake = &self.snakes[snake_index];

        // Check if the position collides with the snake's own neck
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
