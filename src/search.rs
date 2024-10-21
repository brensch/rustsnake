use crate::game_state::{Direction, GameState};
use crate::heuristic::{calculate_control_percentages, calculate_snake_control};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::time::{Duration, Instant};

pub struct Node {
    pub game_state: GameState,
    pub total_score: Vec<f32>,
    pub visits: u32,
    pub children: HashMap<Direction, Rc<RefCell<Node>>>,
    pub move_made: Option<Direction>,
    pub parent: Option<Weak<RefCell<Node>>>,
    pub current_player: usize,
    pub num_snakes: usize,
    pub heuristic: Option<Vec<f32>>, // New field to store the original heuristic
    pub is_terminal: bool,           // New field to indicate terminal state
}

pub struct MCTS {
    pub root: Rc<RefCell<Node>>,
    exploration_constant: f32,
}

impl MCTS {
    pub fn new(initial_state: GameState) -> Self {
        let number_of_snakes = initial_state.snakes.len();
        let is_terminal = Self::is_terminal(&initial_state);
        MCTS {
            root: Rc::new(RefCell::new(Node {
                game_state: initial_state,
                total_score: vec![0.0; number_of_snakes],
                visits: 0,
                children: HashMap::new(),
                move_made: None,
                parent: None,
                current_player: 0,
                num_snakes: number_of_snakes,
                heuristic: None,
                is_terminal,
            })),
            exploration_constant: 1.414,
        }
    }

    pub fn run(&mut self, duration: Duration) -> Rc<RefCell<Node>> {
        let start_time = Instant::now();
        let root = Rc::clone(&self.root);

        while Instant::now().duration_since(start_time) < duration {
            self.tree_policy(&root);
        }

        root
    }

    fn tree_policy(&self, node: &Rc<RefCell<Node>>) {
        let mut current = Rc::clone(node);
        loop {
            let expand_result = {
                let node_ref = current.borrow();
                if node_ref.is_terminal {
                    false
                } else if node_ref.children.is_empty() {
                    true
                } else {
                    false
                }
            };

            if expand_result {
                self.expand(&current);
                break;
            } else {
                let node_is_terminal = {
                    let node_ref = current.borrow();
                    node_ref.is_terminal
                };

                if node_is_terminal {
                    break;
                }

                let selected_child = self.select_best_move(&current);
                match selected_child {
                    Some(child) => current = child,
                    None => break,
                }
            }
        }
        self.back_propagate(&current);
    }

    fn expand(&self, node: &Rc<RefCell<Node>>) {
        let mut node_ref = node.borrow_mut();
        let current_player = node_ref.current_player;
        let num_snakes = node_ref.num_snakes;

        if node_ref.game_state.snakes[current_player].health > 0 {
            // Get all possible moves (excluding out-of-bounds and moving into own neck)
            let safe_moves = node_ref.game_state.get_safe_moves(current_player);
            let moves = if safe_moves.is_empty() {
                vec![None] // If no safe moves, the snake doesn't move
            } else {
                safe_moves.into_iter().map(Some).collect()
            };

            for &move_option in &moves {
                let mut new_state = node_ref.game_state.clone();
                if let Some(direction) = move_option {
                    new_state.move_snake(current_player, direction);
                }

                let next_player = (current_player + 1) % num_snakes;
                let should_resolve = next_player == 0;

                if should_resolve {
                    new_state.resolve_collisions();
                }

                // Check if the new state is terminal
                let is_terminal = Self::is_terminal(&new_state);

                let new_node = Node {
                    game_state: new_state,
                    total_score: vec![0.0; num_snakes],
                    visits: 0,
                    children: HashMap::new(),
                    move_made: move_option,
                    parent: Some(Rc::downgrade(node)),
                    current_player: next_player,
                    num_snakes,
                    heuristic: None,
                    is_terminal,
                };

                // Use a unique key for the move, including None
                let direction_key = move_option.unwrap_or(Direction::Up);
                node_ref
                    .children
                    .insert(direction_key, Rc::new(RefCell::new(new_node)));
            }
        } else {
            // If the current snake is dead, skip its turn
            let mut new_state = node_ref.game_state.clone();
            let next_player = (current_player + 1) % num_snakes;
            let should_resolve = next_player == 0;

            if should_resolve {
                new_state.resolve_collisions();
            }

            // Check if the new state is terminal
            let is_terminal = Self::is_terminal(&new_state);

            let new_node = Node {
                game_state: new_state,
                total_score: vec![0.0; num_snakes],
                visits: 0,
                children: HashMap::new(),
                move_made: None,
                parent: Some(Rc::downgrade(node)),
                current_player: next_player,
                num_snakes,
                heuristic: None,
                is_terminal,
            };

            // Use a dummy direction as key
            node_ref
                .children
                .insert(Direction::Up, Rc::new(RefCell::new(new_node)));
        }
    }

    fn select_best_move(&self, node: &Rc<RefCell<Node>>) -> Option<Rc<RefCell<Node>>> {
        let node_ref = node.borrow();
        let current_player = node_ref.current_player;

        if node_ref.children.is_empty() {
            return None;
        }

        node_ref
            .children
            .values()
            .filter(|child| !child.borrow().is_terminal) // Skip terminal nodes
            .max_by(|a, b| {
                let a_ref = a.borrow();
                let b_ref = b.borrow();
                let a_ucb = self.ucb_value(&a_ref, node_ref.visits, current_player);
                let b_ucb = self.ucb_value(&b_ref, node_ref.visits, current_player);
                a_ucb
                    .partial_cmp(&b_ucb)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }

    fn ucb_value(&self, node: &Node, parent_visits: u32, player_index: usize) -> f32 {
        if node.visits == 0 {
            return f32::INFINITY;
        }

        let exploitation = node.total_score[player_index] / node.visits as f32;
        let exploration =
            self.exploration_constant * ((parent_visits as f32).ln() / node.visits as f32).sqrt();

        exploitation + exploration
    }

    fn back_propagate(&self, node: &Rc<RefCell<Node>>) {
        let mut current = Rc::clone(node);

        // Calculate the heuristic score for the current game state and convert to Vec<f32>
        let heuristic = {
            let node_ref = current.borrow();
            calculate_control_percentages(&node_ref.game_state)
        };

        // At the leaf node, store the heuristic
        {
            let mut node_ref = current.borrow_mut();
            node_ref.heuristic = Some(heuristic.clone());
        }

        loop {
            let mut node_ref = current.borrow_mut();
            node_ref.visits += 1;

            // Update total_score for each snake
            for i in 0..node_ref.num_snakes {
                if i < heuristic.len() {
                    node_ref.total_score[i] += heuristic[i];
                }
            }

            match node_ref.parent.as_ref().and_then(|w| w.upgrade()) {
                Some(parent) => {
                    drop(node_ref);
                    current = parent;
                }
                None => break,
            }
        }
    }

    fn is_terminal(game_state: &GameState) -> bool {
        let alive_snakes = game_state.snakes.iter().filter(|s| s.health > 0).count();
        alive_snakes == 0 || alive_snakes == 1 // All snakes dead or only one snake left
    }

    pub fn get_best_move_for_snake(&self, our_snake_id: &str) -> Option<Direction> {
        let root = self.root.borrow();

        if !root.children.is_empty() {
            let best_child = root
                .children
                .iter()
                .max_by_key(|(_, child)| child.borrow().visits)
                .map(|(direction, _)| *direction);

            return best_child;
        }

        None
    }
}
