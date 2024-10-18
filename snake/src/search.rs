use crate::game_state::{Direction, GameState};
use crate::heuristic;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Node {
    pub game_state: GameState,
    pub value: Vec<f32>,
    pub total_value: Vec<f32>, // Accumulated values for each player
    pub visits: u32,
    pub children: Vec<Arc<Mutex<Node>>>,
    pub moves: Vec<Option<Direction>>, // Moves made to reach this node
    pub parent: Weak<Mutex<Node>>,
}

pub struct MCTS {
    pub root: Arc<Mutex<Node>>,
    exploration_constant: f32,
}

impl MCTS {
    pub fn new(initial_state: GameState) -> Self {
        let number_of_players = initial_state.snakes.len();
        MCTS {
            root: Arc::new(Mutex::new(Node {
                game_state: initial_state,
                value: vec![0.0; number_of_players],
                total_value: vec![0.0; number_of_players],
                visits: 0,
                children: Vec::new(),
                moves: Vec::new(), // Root node has no moves
                parent: Weak::new(),
            })),
            exploration_constant: 1.414,
        }
    }

    pub fn run(&mut self, duration: Duration, num_threads: usize) -> Arc<Mutex<Node>> {
        let start_time = Instant::now();
        let root = Arc::clone(&self.root);
        let exploration_constant = self.exploration_constant;

        let handles: Vec<_> = (0..num_threads)
            .map(|_| {
                let root_clone = Arc::clone(&root);
                let duration_clone = duration;
                let ec = exploration_constant;
                thread::spawn(move || {
                    while Instant::now().duration_since(start_time) < duration_clone {
                        if let Err(e) = Self::tree_policy(&root_clone, ec) {
                            eprintln!("Error in tree policy: {:?}", e);
                        }
                    }
                })
            })
            .collect();

        for handle in handles {
            if let Err(e) = handle.join() {
                eprintln!("Error joining thread: {:?}", e);
            }
        }

        root
    }

    fn tree_policy(node: &Arc<Mutex<Node>>, exploration_constant: f32) -> Result<(), String> {
        let mut current = Arc::clone(node);
        loop {
            let expand_result = {
                let node = current.lock().unwrap_or_else(|e| e.into_inner());
                if Self::is_terminal(&node.game_state) {
                    false
                } else if node.children.is_empty() {
                    true
                } else {
                    false
                }
            };

            if expand_result {
                Self::expand(&current);
                // Optionally select a child to continue the simulation
                let selected_child = {
                    let node = current.lock().unwrap_or_else(|e| e.into_inner());
                    node.children.first().cloned()
                };
                if let Some(child) = selected_child {
                    current = child;
                }
                break;
            } else {
                let node_is_terminal = {
                    let node = current.lock().unwrap_or_else(|e| e.into_inner());
                    Self::is_terminal(&node.game_state)
                };

                if node_is_terminal {
                    // Node is terminal; cannot proceed further
                    break;
                }

                let selected_child = Self::select_child(&current, exploration_constant);
                match selected_child {
                    Some(child) => current = child,
                    None => break,
                }
            }
        }
        Self::back_propagate(&current);
        Ok(())
    }

    fn expand(node: &Arc<Mutex<Node>>) {
        let mut node_lock = node.lock().unwrap_or_else(|e| e.into_inner());
        let num_snakes = node_lock.game_state.snakes.len();

        let mut snakes_safe_moves = Vec::new();
        for snake_index in 0..num_snakes {
            let snake = &node_lock.game_state.snakes[snake_index];
            if snake.health > 0 {
                let safe_moves = node_lock.game_state.get_safe_moves(snake_index);
                let moves_with_option: Vec<Option<Direction>> = if safe_moves.is_empty() {
                    vec![None] // Snake has no safe moves
                } else {
                    safe_moves.into_iter().map(Some).collect()
                };
                snakes_safe_moves.push(moves_with_option);
            } else {
                // Dead snake: only possible move is None
                snakes_safe_moves.push(vec![None]);
            }
        }

        let move_combinations = cartesian_product(&snakes_safe_moves);

        for moves in move_combinations {
            let mut new_state = node_lock.game_state.clone();
            for (i, move_option) in moves.iter().enumerate() {
                if let Some(direction) = move_option {
                    new_state.move_snake(i, *direction);
                }
            }
            new_state.resolve_collisions();

            // Do not remove dead snakes to keep indices consistent
            // new_state.snakes.retain(|s| s.health > 0);

            let number_of_players = node_lock.game_state.snakes.len();

            let new_node = Node {
                game_state: new_state,
                value: vec![0.0; number_of_players],
                total_value: vec![0.0; number_of_players],
                visits: 0,
                children: Vec::new(),
                moves: moves.clone(),
                parent: Arc::downgrade(node),
            };
            node_lock.children.push(Arc::new(Mutex::new(new_node)));
        }
    }

    fn select_child(
        node: &Arc<Mutex<Node>>,
        exploration_constant: f32,
    ) -> Option<Arc<Mutex<Node>>> {
        let node_lock = node.lock().unwrap_or_else(|e| e.into_inner());
        if node_lock.children.is_empty() {
            return None;
        }
        node_lock
            .children
            .iter()
            .max_by(|a, b| {
                let a_lock = a.lock().unwrap_or_else(|e| e.into_inner());
                let b_lock = b.lock().unwrap_or_else(|e| e.into_inner());
                let a_ucb = Self::ucb_value(&a_lock, node_lock.visits as f32, exploration_constant);
                let b_ucb = Self::ucb_value(&b_lock, node_lock.visits as f32, exploration_constant);
                a_ucb
                    .partial_cmp(&b_ucb)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }

    fn ucb_value(node: &Node, parent_visits: f32, exploration_constant: f32) -> f32 {
        if node.visits == 0 {
            return f32::INFINITY;
        }
        let average_value = node.value.iter().sum::<f32>() / node.value.len() as f32;
        let exploration = ((2.0 * parent_visits.ln()) / node.visits as f32).sqrt();
        average_value + exploration_constant * exploration
    }

    fn back_propagate(node: &Arc<Mutex<Node>>) {
        let mut current = Arc::clone(node);
        let control_percentages = {
            let node_lock = current.lock().unwrap_or_else(|e| e.into_inner());
            heuristic::calculate_control_percentages(&node_lock.game_state)
        };

        loop {
            let mut node = current.lock().unwrap_or_else(|e| e.into_inner());
            node.visits += 1;

            // Accumulate control percentages
            for (i, val) in control_percentages.iter().enumerate() {
                node.total_value[i] += val;
            }

            // Update node.value (average value per player)
            node.value = node
                .total_value
                .iter()
                .map(|&v| v / node.visits as f32)
                .collect();

            match node.parent.upgrade() {
                Some(parent) => {
                    drop(node);
                    current = parent;
                }
                None => break,
            }
        }
    }

    fn is_terminal(game_state: &GameState) -> bool {
        let alive_snakes = game_state.snakes.iter().filter(|s| s.health > 0).count();
        alive_snakes <= 1
    }

    pub fn get_best_move_for_snake(&self, our_snake_id: &str) -> Option<Direction> {
        let root = self.root.lock().unwrap_or_else(|e| e.into_inner());

        if !root.children.is_empty() {
            let best_child = root
                .children
                .iter()
                .max_by_key(|child| child.lock().unwrap_or_else(|e| e.into_inner()).visits);

            if let Some(child) = best_child {
                let child_lock = child.lock().unwrap_or_else(|e| e.into_inner());

                // Find our snake in the parent game state
                let our_snake_index = root
                    .game_state
                    .snakes
                    .iter()
                    .position(|s| s.id == our_snake_id);

                if let Some(index) = our_snake_index {
                    // Check if our snake is still alive
                    if let Some(direction) = child_lock.moves.get(index).and_then(|&dir| dir) {
                        return Some(direction);
                    }
                }
            }
        }

        // Fallback to a default move
        None
    }
}

// Helper function to compute the Cartesian product of a list of lists
fn cartesian_product<T: Clone>(lists: &[Vec<T>]) -> Vec<Vec<T>> {
    let mut result: Vec<Vec<T>> = vec![vec![]];
    for pool in lists {
        if pool.is_empty() {
            // If any pool is empty, the Cartesian product is empty
            return vec![];
        }
        let mut temp = Vec::new();
        for x in &result {
            for y in pool {
                let mut x = x.clone();
                x.push(y.clone());
                temp.push(x);
            }
        }
        result = temp;
    }
    result
}
