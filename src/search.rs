use crate::game_state::{Direction, GameState};
use crate::heuristic;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Node {
    pub game_state: GameState,
    pub value: Vec<f32>,       // Average value per player
    pub total_value: Vec<f32>, // Accumulated values for each player
    pub visits: u32,
    pub children: HashMap<Direction, Arc<Mutex<Node>>>, // Map from moves to child nodes
    pub move_made: Option<Direction>,                   // Move made to reach this node
    pub parent: Weak<Mutex<Node>>,
    pub current_player: usize,       // Index of the current player
    pub initial_heuristic: Vec<f32>, // Initial heuristic value of the node
}

pub struct MCTS {
    pub root: Arc<Mutex<Node>>,
    exploration_constant: f32,
}

impl MCTS {
    pub fn new(initial_state: GameState) -> Self {
        let number_of_players = initial_state.snakes.len();
        let initial_heuristic = heuristic::calculate_control_percentages(&initial_state);
        MCTS {
            root: Arc::new(Mutex::new(Node {
                game_state: initial_state,
                value: initial_heuristic.clone(),
                total_value: vec![0.0; number_of_players],
                visits: 0,
                children: HashMap::new(),
                move_made: None,
                parent: Weak::new(),
                current_player: 0,
                initial_heuristic,
            })),
            exploration_constant: 1.414,
        }
    }

    pub fn run(&mut self, duration: Duration) -> Arc<Mutex<Node>> {
        let start_time = Instant::now();
        let root = Arc::clone(&self.root);

        while Instant::now().duration_since(start_time) < duration {
            if let Err(e) = Self::tree_policy(&root, self.exploration_constant) {
                eprintln!("Error in tree policy: {:?}", e);
            }
        }

        root
    }

    fn tree_policy(node: &Arc<Mutex<Node>>, exploration_constant: f32) -> Result<(), String> {
        let mut current = Arc::clone(node);
        loop {
            let expand_result = {
                let node = current.lock().unwrap();
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
                break;
            } else {
                let node_is_terminal = {
                    let node = current.lock().unwrap();
                    Self::is_terminal(&node.game_state)
                };

                if node_is_terminal {
                    break;
                }

                let selected_child = Self::select_best_move(&current, exploration_constant);
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
        let mut node_lock = node.lock().unwrap();
        let current_player = node_lock.current_player;
        let num_snakes = node_lock.game_state.snakes.len();

        if node_lock.game_state.snakes[current_player].health > 0 {
            let safe_moves = node_lock.game_state.get_safe_moves(current_player);
            let moves = if safe_moves.is_empty() {
                vec![None]
            } else {
                safe_moves.into_iter().map(Some).collect()
            };

            for &move_option in &moves {
                let mut new_state = node_lock.game_state.clone();
                if let Some(direction) = move_option {
                    new_state.move_snake(current_player, direction);
                }

                let next_player = (current_player + 1) % num_snakes;
                let should_resolve = next_player == 0;

                if should_resolve {
                    new_state.resolve_collisions();
                }

                let initial_heuristic = heuristic::calculate_control_percentages(&new_state);
                let new_node = Node {
                    game_state: new_state,
                    value: initial_heuristic.clone(),
                    total_value: vec![0.0; num_snakes],
                    visits: 0,
                    children: HashMap::new(),
                    move_made: move_option,
                    parent: Arc::downgrade(node),
                    current_player: next_player,
                    initial_heuristic,
                };

                if let Some(direction) = move_option {
                    node_lock
                        .children
                        .insert(direction, Arc::new(Mutex::new(new_node)));
                }
            }
        } else {
            // If the current snake is dead, create a single child node with no move
            let mut new_state = node_lock.game_state.clone();
            let next_player = (current_player + 1) % num_snakes;
            let should_resolve = next_player == 0;

            if should_resolve {
                new_state.resolve_collisions();
            }

            let initial_heuristic = heuristic::calculate_control_percentages(&new_state);
            let new_node = Node {
                game_state: new_state,
                value: initial_heuristic.clone(),
                total_value: vec![0.0; num_snakes],
                visits: 0,
                children: HashMap::new(),
                move_made: None,
                parent: Arc::downgrade(node),
                current_player: next_player,
                initial_heuristic,
            };

            node_lock
                .children
                .insert(Direction::Up, Arc::new(Mutex::new(new_node))); // Use Up as a dummy direction
        }
    }

    fn select_best_move(
        node: &Arc<Mutex<Node>>,
        exploration_constant: f32,
    ) -> Option<Arc<Mutex<Node>>> {
        let node_lock = node.lock().unwrap();

        if node_lock.children.is_empty() {
            return None;
        }

        node_lock
            .children
            .values()
            .max_by(|a, b| {
                let a_lock = a.lock().unwrap();
                let b_lock = b.lock().unwrap();
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
        let exploitation = node.value[node.current_player];
        let exploration = ((2.0 * parent_visits.ln()) / node.visits as f32).sqrt();
        exploitation + exploration_constant * exploration
    }

    fn back_propagate(node: &Arc<Mutex<Node>>) {
        let mut current = Arc::clone(node);
        loop {
            let mut node = current.lock().unwrap();
            node.visits += 1;

            // Update total_value and value
            for i in 0..node.value.len() {
                node.total_value[i] += node.initial_heuristic[i];
                node.value[i] = node.total_value[i] / node.visits as f32;
            }

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
        let root = self.root.lock().unwrap();

        if !root.children.is_empty() {
            let best_child = root
                .children
                .iter()
                .max_by_key(|(_, child)| child.lock().unwrap().visits)
                .map(|(direction, _)| *direction);

            return best_child;
        }

        None
    }
}
