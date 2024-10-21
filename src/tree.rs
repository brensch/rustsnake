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
use std::sync::atomic::Ordering;
use std::sync::{Arc, Weak};
use uuid::Uuid;

#[derive(Serialize)]
pub struct Snake {
    pub id: String,
    pub body: Vec<usize>,
    pub health: u8,
    pub head: usize,
}

#[derive(Serialize)]
pub struct Board {
    pub height: usize,
    pub width: usize,
    pub food: Vec<usize>,
    pub hazards: Vec<usize>,
    pub snakes: Vec<Snake>,
}

#[derive(Serialize)]
pub struct TreeNode {
    pub id: String,
    pub visits: u32,
    pub ucb: f32,
    pub isMostVisited: bool,
    pub children: Vec<TreeNode>,
    pub body: String,
    pub board: Board,
}

impl TreeNode {
    fn from_node(node: &Arc<Node>, exploration_constant: f32, is_root: bool) -> Self {
        // Since we're using atomics, we need to load the values
        let visits = node.visits.load(Ordering::Relaxed);

        let total_score_clone: Vec<u32> = node
            .total_score
            .iter()
            .map(|score| score.load(Ordering::Relaxed))
            .collect();

        let heuristics_clone = node.heuristic.clone();
        let parent_weak = node.parent.clone();
        let id = format!("Node_{:p}", Arc::as_ptr(node));
        let body = visualize_game_state(&node.game_state);
        let game_state = node.game_state.clone();
        let terminal = node.is_terminal;

        let ucb = calculate_ucb_value(&node, parent_weak.as_ref(), exploration_constant);

        let board = game_state_to_board(&game_state);

        let snake_control = calculate_snake_control(&game_state);
        let control_visualization =
            visualize_control(&snake_control, game_state.width, game_state.height);

        let total_scores = game_state
            .snakes
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let score = total_score_clone.get(i).cloned().unwrap_or(0) as f32 / 1000.0;
                format!("Player {}: total Score: {:.2}", i + 1, score)
            })
            .collect::<Vec<String>>()
            .join("\n");

        let heuristics = game_state
            .snakes
            .iter()
            .enumerate()
            .map(|(i, _)| {
                // If heuristics_clone is Some, use the value; otherwise, return a default score
                let score = heuristics_clone
                    .as_ref() // Access the reference to Option
                    .map(|heuristics| heuristics.get(i).cloned().unwrap_or(-69.0)) // Get the i-th score if exists
                    .unwrap_or(0.0); // Default to 0.0 if heuristics_clone is None

                format!("Player {}: heuristic Score: {:.2}", i + 1, score)
            })
            .collect::<Vec<String>>()
            .join("\n");

        let body_with_extra_text = format!(
            "{}\nVisits: {}\nUCB: {:.2}\nTotal Scores:\n{}\nHeuristics:\n{}\nControl Layout:\n{}\nTerminal:{}",
            body, visits, ucb, total_scores, heuristics, control_visualization, terminal,
        );

        TreeNode {
            id,
            visits,
            ucb,
            isMostVisited: is_root,
            children: Vec::new(),
            body: body_with_extra_text,
            board,
        }
    }
}

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
                    head: s.head().index,
                }
            })
            .collect(),
    }
}

pub fn generate_most_visited_path_with_alternatives_html_tree(
    root_node: &Arc<Node>,
) -> Result<(), std::io::Error> {
    println!("starting");
    let tree_node = generate_tree_data(root_node);

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S%.6f").to_string();
    let uuid = Uuid::new_v4().to_string();
    let file_name = format!("{}_{}", timestamp, uuid);
    let file_location = format!("visualiser/tree-data/{}.json", file_name);

    let path = Path::new(&file_location);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = File::create(&file_location)?;

    let json_data = serde_json::to_string(&tree_node)?;
    file.write_all(json_data.as_bytes())?;

    println!(
        "Generated move tree: http://localhost:5173/trees/{}",
        file_name
    );

    Ok(())
}

fn generate_tree_data(root_node: &Arc<Node>) -> TreeNode {
    println!("getting data");

    let root_tree_node = TreeNode::from_node(root_node, 1.414, true);
    println!("got data");

    let mut root_tree_node = root_tree_node;
    traverse_and_build_tree(root_node, &mut root_tree_node);

    root_tree_node
}

fn traverse_and_build_tree(node: &Arc<Node>, tree_node: &mut TreeNode) {
    let children_nodes: Vec<_> = node
        .children
        .iter()
        .map(|entry| entry.value().clone())
        .collect();

    let mut sorted_children = children_nodes;
    sorted_children.sort_by(|a, b| {
        let a_visits = a.visits.load(Ordering::Relaxed);
        let b_visits = b.visits.load(Ordering::Relaxed);
        b_visits.cmp(&a_visits)
    });

    for (i, child_node) in sorted_children.iter().enumerate() {
        let mut child_tree_node = TreeNode::from_node(child_node, 1.414, false);

        if i == 0 {
            child_tree_node.isMostVisited = true;
        }

        traverse_and_build_tree(child_node, &mut child_tree_node);
        tree_node.children.push(child_tree_node);
    }
}

fn calculate_ucb_value(node: &Node, parent: Option<&Weak<Node>>, exploration_constant: f32) -> f32 {
    // Load node's visits atomically
    let node_visits = node.visits.load(Ordering::Relaxed) as f32;

    if node_visits == 0.0 {
        return f32::INFINITY;
    }

    // Load parent's visits atomically
    let parent_visits = parent
        .and_then(|weak| weak.upgrade())
        .map(|arc| arc.visits.load(Ordering::Relaxed) as f32)
        .unwrap_or(1.0);

    // Load the total score atomically and convert it to f32 for calculation
    let total_score = node.total_score[node.current_player].load(Ordering::Relaxed) as f32;

    // Adjust total_score if you scaled it during backpropagation (e.g., divided by 1000)
    let adjusted_total_score = total_score / 1000.0;

    // Calculate the exploitation term
    let exploitation = adjusted_total_score / node_visits;

    // Calculate the exploration term using the exploration constant and parent visits
    let exploration = exploration_constant * ((parent_visits.ln()) / node_visits).sqrt();

    // Return the combined UCB value
    exploitation + exploration
}
