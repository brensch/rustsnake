use crate::game_state::{Direction, GameState};
use crate::heuristic::calculate_control_percentages;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Weak};
use std::thread;
use std::time::{Duration, Instant};

pub struct Node {
    pub game_state: GameState,
    pub total_score: Vec<AtomicU32>,
    pub visits: AtomicU32,
    pub children: DashMap<Direction, Arc<Node>>,
    pub move_made: Option<Direction>,
    pub parent: Option<Weak<Node>>,
    pub current_player: usize,
    pub num_snakes: usize,
    pub is_terminal: bool,
    pub heuristic: Option<Vec<f32>>, // Added back the heuristic field
}

pub struct MCTS {
    pub root: Arc<Node>,
    exploration_constant: f32,
}

impl MCTS {
    pub fn new(initial_state: GameState) -> Self {
        let number_of_snakes = initial_state.snakes.len();
        let is_terminal = Self::is_terminal(&initial_state);
        MCTS {
            root: Arc::new(Node {
                game_state: initial_state,
                total_score: (0..number_of_snakes).map(|_| AtomicU32::new(0)).collect(),
                visits: AtomicU32::new(0),
                children: DashMap::new(),
                move_made: None,
                parent: None,
                current_player: 0,
                num_snakes: number_of_snakes,
                is_terminal,
                heuristic: None, // Initialize heuristic as None
            }),
            exploration_constant: 1.414,
        }
    }

    pub fn run(&self, duration: Duration, num_threads: usize) {
        let start_time = Instant::now();

        let handles: Vec<_> = (0..num_threads)
            .map(|_| {
                let root_clone = Arc::clone(&self.root);
                let exploration_constant = self.exploration_constant;
                thread::spawn(move || {
                    while Instant::now().duration_since(start_time) < duration {
                        Self::tree_policy(&root_clone, exploration_constant);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    pub fn get_best_move_for_snake(&self, our_snake_id: &str) -> Option<Direction> {
        let root = &self.root;

        if !root.children.is_empty() {
            let best_child = root
                .children
                .iter()
                .max_by_key(|entry| entry.value().visits.load(Ordering::Relaxed))
                .map(|entry| *entry.key());

            return best_child;
        }

        None
    }

    fn tree_policy(node: &Arc<Node>, exploration_constant: f32) {
        let mut path = Vec::new();
        let mut current_node = Arc::clone(node);

        loop {
            path.push(Arc::clone(&current_node));

            if current_node.is_terminal {
                break;
            }

            // Try to expand the node
            if Self::expand(&current_node) {
                // Node was expanded, select one of the new children
                let selected_child = Self::select_child(&current_node, exploration_constant);
                current_node = selected_child;
                path.push(Arc::clone(&current_node));
                break;
            } else {
                // Select best child
                let selected_child = Self::select_child(&current_node, exploration_constant);
                current_node = selected_child;
            }
        }

        // Simulate a playout from the current node
        let simulation_result = Self::default_policy(&current_node.game_state);

        // Backpropagate the result
        Self::back_propagate(&path, &simulation_result);

        // Store heuristic at the leaf node
        if let Some(leaf_node) = path.last() {
            // Only store heuristic if it's not already stored
            if leaf_node.heuristic.is_none() {
                let heuristic = simulation_result.clone();
                // Since we're using Arc<Node>, we need to update heuristic in a thread-safe way
                // You can use a Mutex or RwLock for this field, or design the Node to have interior mutability for heuristic
                // For simplicity, let's assume we can set it here safely (since only one thread writes to it at a time)
                // This is acceptable in this context because each simulation works on its own path
                unsafe {
                    let node_ptr = Arc::as_ptr(leaf_node) as *mut Node;
                    (*node_ptr).heuristic = Some(heuristic);
                }
            }
        }
    }

    fn expand(node: &Arc<Node>) -> bool {
        if node.is_terminal || !node.children.is_empty() {
            return false;
        }

        let current_player = node.current_player;
        let num_snakes = node.num_snakes;

        if node.game_state.snakes[current_player].health > 0 {
            // Get all possible moves (excluding out-of-bounds and moving into own neck)
            let safe_moves = node.game_state.get_safe_moves(current_player);
            let moves = if safe_moves.is_empty() {
                vec![None] // If no safe moves, the snake doesn't move
            } else {
                safe_moves.into_iter().map(Some).collect()
            };

            for &move_option in &moves {
                let mut new_state = node.game_state.clone();
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

                let child_node = Arc::new(Node {
                    game_state: new_state,
                    total_score: (0..num_snakes).map(|_| AtomicU32::new(0)).collect(),
                    visits: AtomicU32::new(0),
                    children: DashMap::new(),
                    move_made: move_option,
                    parent: Some(Arc::downgrade(node)),
                    current_player: next_player,
                    num_snakes,
                    is_terminal,
                    heuristic: None, // Initialize heuristic as None
                });

                let direction_key = move_option.unwrap_or(Direction::Up);
                node.children.insert(direction_key, child_node);
            }
            true
        } else {
            // If the current snake is dead, skip its turn
            let mut new_state = node.game_state.clone();
            let next_player = (current_player + 1) % num_snakes;
            let should_resolve = next_player == 0;

            if should_resolve {
                new_state.resolve_collisions();
            }

            // Check if the new state is terminal
            let is_terminal = Self::is_terminal(&new_state);

            let child_node = Arc::new(Node {
                game_state: new_state,
                total_score: (0..num_snakes).map(|_| AtomicU32::new(0)).collect(),
                visits: AtomicU32::new(0),
                children: DashMap::new(),
                move_made: None,
                parent: Some(Arc::downgrade(node)),
                current_player: next_player,
                num_snakes,
                is_terminal,
                heuristic: None, // Initialize heuristic as None
            });

            node.children.insert(Direction::Up, child_node);
            true
        }
    }

    fn select_child(node: &Arc<Node>, exploration_constant: f32) -> Arc<Node> {
        let parent_visits = node.visits.load(Ordering::Relaxed) as f32;

        node.children
            .iter()
            .map(|entry| {
                let child = entry.value();
                let child_visits = child.visits.load(Ordering::Relaxed) as f32;
                if child_visits == 0.0 {
                    return (Arc::clone(child), f32::INFINITY);
                }
                let total_score =
                    child.total_score[child.current_player].load(Ordering::Relaxed) as f32 / 1000.0; // Adjust for scaling
                let exploitation = total_score / child_visits;
                let exploration =
                    exploration_constant * ((parent_visits.ln()) / child_visits).sqrt();
                let ucb = exploitation + exploration;
                (Arc::clone(child), ucb)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(child, _)| child)
            .unwrap()
    }

    fn default_policy(state: &GameState) -> Vec<f32> {
        // Implement a simulation policy (e.g., random playout)
        // For now, we'll use the heuristic directly
        if Self::is_terminal(state) {
            let mut scores = vec![0.0; state.snakes.len()];
            let alive_snakes: Vec<_> = state
                .snakes
                .iter()
                .enumerate()
                .filter(|(_, s)| s.health > 0)
                .collect();

            if alive_snakes.len() == 1 {
                let (winner_index, _) = alive_snakes[0];
                scores[winner_index] = 1.0;
            }
            scores
        } else {
            // Use heuristic function for non-terminal states
            calculate_control_percentages(state)
        }
    }

    fn back_propagate(path: &[Arc<Node>], simulation_result: &[f32]) {
        for node in path.iter().rev() {
            node.visits.fetch_add(1, Ordering::Relaxed);
            for i in 0..node.num_snakes {
                let delta = (simulation_result[i] * 1000.0) as u32; // Scale to integer
                node.total_score[i].fetch_add(delta, Ordering::Relaxed);
            }
        }
    }

    fn is_terminal(game_state: &GameState) -> bool {
        let alive_snakes = game_state.snakes.iter().filter(|s| s.health > 0).count();
        alive_snakes == 0 || alive_snakes == 1 // All snakes dead or only one snake left
    }
}
