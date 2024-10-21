use crate::game_state::{GameState, Position, Snake as GameStateSnake};
use crate::heuristic::calculate_snake_control;
use crate::search::Node;
use crate::visualizer::{visualize_control, visualize_game_state};
use chrono::Utc;
use serde::Serialize;
use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex, Weak};
use uuid::Uuid;

#[derive(Serialize)]
pub struct Snake {
    pub id: String,
    pub body: Vec<usize>, // Using Vec<usize> for snake body positions
    pub health: u8,
    pub head: usize,
}

#[derive(Serialize)]
pub struct Board {
    pub height: usize,
    pub width: usize,
    pub food: Vec<usize>,    // Positions of food on the board
    pub hazards: Vec<usize>, // Positions of hazards
    pub snakes: Vec<Snake>,  // Snakes on the board
}

#[derive(Serialize)]
pub struct TreeNode {
    pub id: String,
    pub visits: u32,
    pub avg_score: f32,
    pub ucb: f32,
    pub isMostVisited: bool,
    pub children: Vec<TreeNode>,
    pub body: String,
    pub board: Board, // Add board representation here
}

impl TreeNode {
    fn from_node(node: &Arc<Mutex<Node>>, exploration_constant: f32, is_root: bool) -> Self {
        // Acquire lock only to copy necessary data
        let (visits, avg_scores, value_clone, parent_weak, id, body, game_state) = {
            let node_lock = node.lock().unwrap_or_else(|e| e.into_inner());
            (
                node_lock.visits,
                if node_lock.visits > 0 {
                    node_lock
                        .total_value
                        .iter()
                        .map(|v| *v / node_lock.visits as f32)
                        .collect()
                } else {
                    vec![0.0; node_lock.value.len()]
                },
                node_lock.value.clone(),
                node_lock.parent.clone(),
                format!("Node_{:p}", &*node_lock as *const _),
                visualize_game_state(&node_lock.game_state), // Get the initial body text
                node_lock.game_state.clone(),
            )
        }; // Lock is released here

        let avg_score = avg_scores.iter().copied().sum::<f32>() / avg_scores.len() as f32;
        let ucb_values =
            calculate_ucb_values(visits, &value_clone, parent_weak, exploration_constant);
        let ucb = ucb_values.iter().copied().sum::<f32>() / ucb_values.len() as f32;

        // Create a Board object from the game_state
        let board = game_state_to_board(&game_state);

        // Calculate snake control and visualize it
        let snake_control = calculate_snake_control(&game_state);
        let control_visualization =
            visualize_control(&snake_control, game_state.width, game_state.height);

        // Add individual player scores, safely accessing avg_scores
        let player_scores = game_state
            .snakes
            .iter()
            .enumerate()
            .map(|(i, snake)| {
                let score = avg_scores.get(i).cloned().unwrap_or(0.0); // Fallback to 0.0 if index is out of bounds
                format!("Player {}: avg Score: {:.2}", i + 1, score)
            })
            .collect::<Vec<String>>()
            .join("\n");

        let instant_scores = game_state
            .snakes
            .iter()
            .enumerate()
            .map(|(i, snake)| {
                let score = value_clone.get(i).cloned().unwrap_or(0.0); // Fallback to 0.0 if index is out of bounds
                format!("Player {}: instant Score: {:.2}", i + 1, score)
            })
            .collect::<Vec<String>>()
            .join("\n");

        // Add extra text to the body, including the heuristic layout and player scores
        let body_with_extra_text = format!(
            "{}\nVisits: {}\nAvg Score: {:.2}\nUCB: {:.2}\n{}\n\nPlayer Scores:\n{}\n\nControl Layout:\n{}",
            body,      // Existing body text from visualize_game_state
            visits,    // Add visits count
            avg_score, // Add average score
            ucb,       // Add UCB value
            // if is_root { "Root Node" } else { "" },  // Optional text for root node
            player_scores, // Add the scores for each player
            instant_scores,
            control_visualization // Add the visualized control layout
        );

        TreeNode {
            id,
            visits,
            avg_score,
            ucb,
            isMostVisited: is_root, // Root node should be marked as most visited
            children: Vec::new(),
            body: body_with_extra_text, // Use the updated body with extra text
            board,
        }
    }
}

// Convert GameState to Board for visualization
fn game_state_to_board(game_state: &GameState) -> Board {
    Board {
        height: game_state.height,
        width: game_state.width,
        food: game_state.food.iter().map(|f| f.index).collect(),
        hazards: game_state.hazards.iter().map(|h| h.index).collect(),
        snakes: game_state
            .snakes
            .iter()
            .map(|s| {
                let body: Vec<usize> = s.body.iter().map(|p| p.index).collect();
                Snake {
                    id: s.id.clone(),
                    body,
                    health: s.health,
                    head: s.head().index, // Using the head method to get head position
                }
            })
            .collect(),
    }
}

pub fn generate_most_visited_path_with_alternatives_html_tree(
    root_node: &Arc<Mutex<Node>>,
) -> Result<(), std::io::Error> {
    println!("starting");
    let tree_node = generate_tree_data(root_node);

    // Serialize to JSON
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S%.6f").to_string();
    let uuid = Uuid::new_v4().to_string();
    let file_name = format!("{}_{}", timestamp, uuid);
    let file_location = format!("visualiser/tree-data/{}.json", file_name);

    // Ensure the directory exists
    let path = Path::new(&file_location);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Create the output file
    let mut file = File::create(&file_location)?;

    // Serialize tree_node to JSON and write to file
    let json_data = serde_json::to_string(&tree_node)?;
    file.write_all(json_data.as_bytes())?;

    println!(
        "Generated move tree: http://localhost:5173/trees/{}",
        file_name
    );

    Ok(())
}

fn generate_tree_data(root_node: &Arc<Mutex<Node>>) -> TreeNode {
    println!("getting lock");

    let mut root_tree_node = TreeNode::from_node(root_node, 1.414, true); // Root node, set is_most_visited to true
    println!("got lock");

    traverse_and_build_tree(root_node, &mut root_tree_node);

    root_tree_node
}

fn traverse_and_build_tree(node: &Arc<Mutex<Node>>, tree_node: &mut TreeNode) {
    // Acquire lock only to clone children
    let children_nodes: Vec<_> = {
        let node_lock = node.lock().unwrap_or_else(|e| e.into_inner());
        node_lock.children.values().cloned().collect()
    }; // Lock is released here

    // Sort children by visits descending
    let mut sorted_children = children_nodes;
    sorted_children.sort_by(|a, b| {
        let a_visits = {
            let a_lock = a.lock().unwrap_or_else(|e| e.into_inner());
            a_lock.visits
        };
        let b_visits = {
            let b_lock = b.lock().unwrap_or_else(|e| e.into_inner());
            b_lock.visits
        };
        b_visits.cmp(&a_visits)
    });

    for (i, child_node) in sorted_children.iter().enumerate() {
        let mut child_tree_node = TreeNode::from_node(child_node, 1.414, false); // Not root, so is_most_visited = false

        if i == 0 {
            child_tree_node.isMostVisited = true; // Mark the most visited child
        }

        traverse_and_build_tree(child_node, &mut child_tree_node);
        tree_node.children.push(child_tree_node);
    }
}

fn calculate_ucb_values(
    node_visits: u32,
    node_value: &Vec<f32>,
    parent: Weak<Mutex<Node>>,
    exploration_constant: f32,
) -> Vec<f32> {
    let node_visits_f32 = node_visits as f32;
    let ln_parent_visits = {
        if let Some(parent_arc) = parent.upgrade() {
            let parent_visits = {
                let p_lock = parent_arc.lock().unwrap_or_else(|e| e.into_inner());
                p_lock.visits as f32
            };
            if parent_visits > 0.0 {
                parent_visits.ln()
            } else {
                0.0
            }
        } else {
            0.0
        }
    };

    node_value
        .iter()
        .map(|&v| {
            if node_visits_f32 == 0.0 {
                f32::INFINITY
            } else {
                let avg_value = v / node_visits_f32;
                let exploration = (2.0 * ln_parent_visits / node_visits_f32).sqrt();
                avg_value + exploration_constant * exploration
            }
        })
        .collect()
}
