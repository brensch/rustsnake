use crate::game_state::{Direction, GameState};
use crate::heuristic;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Node {
    game_state: GameState,
    value: Vec<f32>,
    visits: u32,
    children: Vec<Arc<Mutex<Node>>>,
    moves: Vec<Option<Direction>>,
    parent: Weak<Mutex<Node>>,
}

pub struct MCTS {
    root: Arc<Mutex<Node>>,
    exploration_constant: f32,
}

impl MCTS {
    pub fn new(initial_state: GameState) -> Self {
        let num_snakes = initial_state.snakes.len();
        MCTS {
            root: Arc::new(Mutex::new(Node {
                game_state: initial_state,
                value: vec![0.0; num_snakes],
                visits: 0,
                children: Vec::new(),
                moves: vec![None; num_snakes],
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
                let node = current.lock().unwrap();
                if node.children.is_empty() {
                    if node.visits == 0 {
                        true
                    } else if Self::is_terminal(&node.game_state) {
                        false
                    } else {
                        true
                    }
                } else {
                    false
                }
            };

            if expand_result {
                Self::expand(&current);
                break;
            }

            let selected_child = Self::select_child(&current, exploration_constant);
            match selected_child {
                Some(child) => current = child,
                None => break,
            }
        }
        Self::back_propagate(&current);
        Ok(())
    }

    fn expand(node: &Arc<Mutex<Node>>) {
        let mut node_lock = node.lock().unwrap();
        let num_snakes = node_lock.game_state.snakes.len();
        for snake_index in 0..num_snakes {
            let safe_moves = node_lock.game_state.get_safe_moves(snake_index);
            for &move_direction in &safe_moves {
                let mut new_moves = node_lock.moves.clone();
                new_moves[snake_index] = Some(move_direction);

                if new_moves.iter().all(|m| m.is_some()) {
                    let mut new_state = node_lock.game_state.clone();
                    for (i, &m) in new_moves.iter().enumerate() {
                        if let Some(direction) = m {
                            new_state.move_snake(i, direction);
                        }
                    }
                    new_state.resolve_collisions();

                    let new_node = Node {
                        game_state: new_state,
                        value: vec![0.0; num_snakes],
                        visits: 0,
                        children: Vec::new(),
                        moves: new_moves,
                        parent: Arc::downgrade(node),
                    };
                    node_lock.children.push(Arc::new(Mutex::new(new_node)));
                } else {
                    let new_node = Node {
                        game_state: node_lock.game_state.clone(),
                        value: vec![0.0; num_snakes],
                        visits: 0,
                        children: Vec::new(),
                        moves: new_moves,
                        parent: Arc::downgrade(node),
                    };
                    node_lock.children.push(Arc::new(Mutex::new(new_node)));
                }
            }
        }
    }

    fn select_child(
        node: &Arc<Mutex<Node>>,
        exploration_constant: f32,
    ) -> Option<Arc<Mutex<Node>>> {
        let node_lock = node.lock().unwrap();
        if node_lock.children.is_empty() {
            return None;
        }
        node_lock
            .children
            .iter()
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
        let exploitation =
            node.value.iter().sum::<f32>() / (node.value.len() as f32 * node.visits as f32);
        let exploration = ((2.0 * parent_visits.ln()) / node.visits as f32).sqrt();
        exploitation + exploration_constant * exploration
    }

    fn back_propagate(node: &Arc<Mutex<Node>>) {
        let mut current = Arc::clone(node);
        loop {
            let mut node = current.lock().unwrap();
            node.visits += 1;
            let control_percentages = heuristic::calculate_control_percentages(&node.game_state);
            for (i, &percentage) in control_percentages.iter().enumerate() {
                node.value[i] += percentage;
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
        game_state.snakes.len() <= 1
    }

    pub fn get_best_moves(&self) -> Vec<Direction> {
        let root = self.root.lock().unwrap();
        root.children
            .iter()
            .max_by_key(|child| child.lock().unwrap().visits)
            .map(|child| {
                child
                    .lock()
                    .unwrap()
                    .moves
                    .iter()
                    .filter_map(|&m| m)
                    .collect()
            })
            .unwrap_or_else(|| {
                root.game_state
                    .snakes
                    .iter()
                    .map(|_| {
                        *[
                            Direction::Up,
                            Direction::Down,
                            Direction::Left,
                            Direction::Right,
                        ]
                        .choose(&mut thread_rng())
                        .unwrap()
                    })
                    .collect()
            })
    }
}
