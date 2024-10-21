use crate::game_state::{Direction, GameState};
use crate::heuristic::calculate_control_percentages;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time::{Duration, Instant};

pub struct Node {
    pub game_state: GameState,
    pub total_score: Vec<f32>,
    pub visits: u32,
    pub children: HashMap<Direction, Arc<Mutex<Node>>>,
    pub move_made: Option<Direction>,
    pub parent: Option<Weak<Mutex<Node>>>,
    pub current_player: usize,
    pub num_snakes: usize,
    pub heuristic: Option<Vec<f32>>, // Stores the heuristic at the leaf node
    pub is_terminal: bool,           // Indicates whether this node is terminal
}

pub struct MCTS {
    pub root: Arc<Mutex<Node>>,
    exploration_constant: f32,
}

impl MCTS {
    pub fn new(initial_state: GameState) -> Self {
        let number_of_snakes = initial_state.snakes.len();
        let is_terminal = MCTSWorker::is_terminal(&initial_state);
        MCTS {
            root: Arc::new(Mutex::new(Node {
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

    pub fn run(&mut self, duration: Duration, num_threads: usize) -> Arc<Mutex<Node>> {
        let start_time = Instant::now();
        let root = Arc::clone(&self.root);
        let exploration_constant = self.exploration_constant;

        let handles: Vec<_> = (0..num_threads)
            .map(|_| {
                let root_clone = Arc::clone(&root);

                thread::spawn(move || {
                    let worker = MCTSWorker {
                        exploration_constant,
                    };
                    while Instant::now().duration_since(start_time) < duration {
                        worker.tree_policy(&root_clone);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        root
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

struct MCTSWorker {
    exploration_constant: f32,
}

impl MCTSWorker {
    fn tree_policy(&self, node: &Arc<Mutex<Node>>) {
        let mut current = Arc::clone(node);
        loop {
            let expand_result = {
                let node_ref = current.lock().unwrap();
                node_ref.children.is_empty()
            };

            if expand_result {
                self.expand(&current);
                break;
            } else {
                let node_is_terminal = {
                    let node_ref = current.lock().unwrap();
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

    fn expand(&self, node: &Arc<Mutex<Node>>) {
        let mut node_ref = node.lock().unwrap();
        if node_ref.is_terminal {
            return; // Do not expand terminal nodes
        }

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
                    parent: Some(Arc::downgrade(node)),
                    current_player: next_player,
                    num_snakes,
                    heuristic: None,
                    is_terminal,
                };

                // Use a unique key for the move, including None
                let direction_key = move_option.unwrap_or(Direction::Up);
                node_ref
                    .children
                    .insert(direction_key, Arc::new(Mutex::new(new_node)));
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
                parent: Some(Arc::downgrade(node)),
                current_player: next_player,
                num_snakes,
                heuristic: None,
                is_terminal,
            };

            // Use a dummy direction as key
            node_ref
                .children
                .insert(Direction::Up, Arc::new(Mutex::new(new_node)));
        }
    }

    fn select_best_move(&self, node: &Arc<Mutex<Node>>) -> Option<Arc<Mutex<Node>>> {
        let node_ref = node.lock().unwrap();
        let current_player = node_ref.current_player;

        if node_ref.children.is_empty() {
            return None;
        }

        node_ref
            .children
            .values()
            .max_by(|a, b| {
                let a_ref = a.lock().unwrap();
                let b_ref = b.lock().unwrap();
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

    fn back_propagate(&self, node: &Arc<Mutex<Node>>) {
        let heuristic = {
            let node_ref = node.lock().unwrap();
            if node_ref.is_terminal {
                // Assign scores based on game outcome
                let mut scores = vec![0.0; node_ref.num_snakes];
                let alive_snakes: Vec<_> = node_ref
                    .game_state
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
                calculate_control_percentages(&node_ref.game_state)
            }
        };

        // At the leaf node, store the heuristic
        {
            let mut node_ref = node.lock().unwrap();
            node_ref.heuristic = Some(heuristic.clone());
        }

        let mut current = Arc::clone(node);
        loop {
            let mut node_ref = current.lock().unwrap();
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
}
